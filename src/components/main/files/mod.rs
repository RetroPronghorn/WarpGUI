use dioxus::prelude::*;

use crate::{
    components::main::files::{
        browser::FileBrowser, sidebar::Sidebar, toolbar::Toolbar, upload::Upload,
    },
    components::reusable::nav::Nav,
    STATE,
};

#[cfg(target_os = "windows")]
use utils::DRAG_FILE_EVENT;
#[cfg(target_os = "windows")]
use dioxus::desktop::wry::webview::FileDropEvent;

pub mod browser;
pub mod sidebar;
pub mod toolbar;
pub mod upload;

#[derive(Props, PartialEq)]
pub struct Props {
    account: crate::Account,
    storage: crate::Storage,
    messaging: crate::Messaging,
}

#[allow(non_snake_case)]
pub fn Files(cx: Scope<Props>) -> Element {
    let show_new_folder = use_state(&cx, || false);
    let show_upload = use_state(&cx, || false);

    let file_storage = cx.props.storage.clone();
    
    let current_dir = file_storage.current_directory().unwrap();   

    let parent_directory = use_ref(&cx, || current_dir.clone());
        
    let st = use_atom_ref(&cx, STATE).clone();
    let sidebar_visibility = match st.read().hide_sidebar {
        false => "mobile-sidebar-visible",
        true => "mobile-sidebar-hidden",
    };

    cx.render(rsx! {
        div {
            id: "files",
            onclick: move |_| {
                if **show_new_folder {
                    show_new_folder.set(false);
                }
            },
            onmouseover: |_| {
                // HACK(Windows): Block upload file if drop it anywhere on screen out
                // TODO(Temp): Temp solution to drag and drop work on Windows
                #[cfg(target_os = "windows")]
                {
                *DRAG_FILE_EVENT.write() = FileDropEvent::Cancelled;
                }
            },
            class: "{sidebar_visibility}",
            Sidebar { account: cx.props.account.clone(), messaging: cx.props.messaging.clone() },
            div {
                id: "content",
                rsx!(
                    div {
                        class: "flex-row top-container",
                        Toolbar {
                            on_new_folder: move |_| {
                                show_new_folder.set(true);
                                show_upload.set(false);
                            },
                            on_show_upload: move |_| {
                                show_upload.set(true);
                                show_new_folder.set(false);
                            }
                        },
                        Upload {
                            storage: cx.props.storage.clone(),
                            show: **show_upload,
                            on_hide: move |_| show_upload.set(false),
                        },
                    },
                    FileBrowser {
                        account: cx.props.account.clone(),
                        storage: cx.props.storage.clone(),
                        show_new_folder: show_new_folder.clone(),
                        parent_directory: parent_directory.clone(),
                    }
                    span {
                        class: "hidden-on-desktop mobile-nav",
                        Nav {
                            account: cx.props.account.clone(),
                            messaging: cx.props.messaging.clone(),
                        }
                    }
                ),
            },
        }
    })
}
