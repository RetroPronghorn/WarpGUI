use std::{fs, collections::HashMap};
use std::ffi::OsStr;
use std::sync::Arc;
use dioxus::prelude::*;
use libloading::{Library, Symbol};
use once_cell::sync::Lazy;
use warp::sync::RwLock;
use tracing::log::{info, error};

type ComponentFn = unsafe fn() -> Box<Component>;
type InfoFn = unsafe fn() -> Box<ExtensionInfo>;

type Extensions = HashMap<ExtensionType, Vec<Extension>>;

static EXTENSION_MANAGER: Lazy<ExtensionManager> = Lazy::new(
    || ExtensionManager::load_or_default()
);
static DEFAULT_PATH: Lazy<RwLock<PathBuf>> =
    Lazy::new(|| RwLock::new(dirs::home_dir().unwrap_or_default().join(".warp")));

#[derive(PartialEq, Eq, Hash, Clone, Debug, Copy)]
pub enum ExtensionType {
    SidebarWidget,
    ChatbarIcon,
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtensionInfo {
    pub name: String,
    pub author: String,
    pub description: String,
    pub location: ExtensionType,
}

#[allow(dead_code)]
pub struct Extension {
    lib: Arc<Library>,
    info: ExtensionInfo,
    component: Component,
}

#[derive(Default)]
#[allow(dead_code)]
pub struct ExtensionManager {
    extensions: Extensions,
    is_loaded: bool,
}

impl Default for ExtensionInfo {
    fn default() -> Self {
        Self {
            name: Default::default(),
            author: Default::default(),
            description: Default::default(),
            location: ExtensionType::SidebarWidget,
        }
    }
}

impl Default for ExtensionManager {
    fn default() -> Self {
        Self {
            extensions: HashMap::new(),
            is_loaded: false,
        }
    }
}

impl Extension {
    pub fn load<P: AsRef<OsStr>>(filename: P) -> Result<Self, anyhow::Error> {
        unsafe {
            let lib = Library::new(filename)?;
            let component: Symbol<ComponentFn> = lib.get(b"ret_rend")?;
            let info: Symbol<InfoFn> = lib.get(b"ret_info")?;

            Ok(Self {
                info: *info(),
                component: *component(),
                lib: Arc::new(lib),
            })
        }
    }
}

impl ExtensionManager {
    pub fn load_or_default() -> Self {
        match Self::load() {
            Ok(instance) => instance,
            Err(err) => {
                error!("Failed to initialize ExtensionManager: {}", err);
                Self::default()
            }
        }
    }

    fn load() -> Result<Self, anyhow::Error> {
        let extensions_path = DEFAULT_PATH.read().join("extensions");
        fs::create_dir_all(&extensions_path)?;
        let paths = fs::read_dir(&extensions_path).expect("Directory is empty");
        let mut extensions: Extensions = HashMap::new();

        for entry in paths {
            let path = entry?.path();
            let result = Extension::load(&path);
            match result {
                Ok(extension) => {
                    info!("Extension loaded {:?}", &extension.info);
                    let location = extension.info.location;
                    extensions.entry(location).or_default().push(extension);
                }
                Err(err) => {
                    error!("Failed to load extension {:?}: {}", &path, err)
                }
            }
        }

        Ok(Self {
            extensions,
            is_loaded: true,
        })
    }

    pub fn instance() -> &'static ExtensionManager {
        Lazy::force(&EXTENSION_MANAGER)
    }
}

#[allow(non_snake_case)]
pub fn get_renders<'src>(location: ExtensionType, enable: bool) -> Vec<LazyNodes<'src, 'src>> {
    if enable {
        let extensions = ExtensionManager::instance().extensions.get(&location);

        if let Some(items) = extensions {
            let nodes: Vec<LazyNodes> = items
                .iter()
                .map(|ext| {
                    let Ext = ext.component;
                    rsx!(div { Ext {} })
                })
                .collect();
            return nodes;
        }
    }
    vec![]
}
