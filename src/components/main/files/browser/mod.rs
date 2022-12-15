use std::{collections::HashSet, time::Duration, path::{PathBuf, Path}};

use dioxus::prelude::*;
use dioxus_heroicons::{Icon, outline::Shape};

use crate::Storage;
use ui_kit::{file::File, folder::{State, Folder}, new_folder::NewFolder};
use warp::constellation::{item::{ItemType}};
mod lib;

#[derive(Props, PartialEq)]
pub struct Props {
    account: crate::Account,
    storage: Storage,
    show_new_folder: UseState<bool>,
    dir_paths: UseRef<Vec<PathBuf>>,
}

#[allow(non_snake_case)]
pub fn FileBrowser(cx: Scope<Props>) -> Element {

    let files = use_ref(&cx, HashSet::new);
    let files_sorted = use_state(&cx, Vec::new);
    let root_directory = cx.props.storage.root_directory();
    let current_directory = cx.props.storage.current_directory().unwrap_or(root_directory.clone());
    let update_current_dir = use_state(&cx, || ());
    let dir_paths = cx.props.dir_paths.clone();

    use_future(
        &cx,
        (files, files_sorted, &current_directory, &cx.props.storage.clone(), &cx.props.dir_paths.clone()),
        |(files, files_sorted, current_directory, files_storage, dir_paths)| async move {
            
            let current_dir_path = files_storage.get_path().clone();
            let dir_paths_vec = dir_paths.with(|vec| vec.clone());
            let dir_paths_len = dir_paths.read().len().clone();
            let final_dir_path = dir_paths.read().last().unwrap().clone();

            if !dir_paths_vec.contains(&current_dir_path) {
                dir_paths.write().insert(dir_paths_len, current_dir_path);
            } else {
                if final_dir_path != current_dir_path {
                    dir_paths.write().remove(dir_paths_len - 1);
                }
            } 

            loop {
                let files_updated: HashSet<_> = HashSet::from_iter(current_directory.get_items());
                if *files.read() != files_updated {
                    log::debug!("updating files list");
                    *files.write_silent() = files_updated.clone();
                    let mut total_files_list: Vec<_> = files_updated.iter().cloned().collect();
                    total_files_list.sort_by_key(|b| std::cmp::Reverse(b.modified()));
                    files_sorted.set(total_files_list);
                }
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        },
    );
    let root_dir_id = root_directory.id();
    let current_dir_items_len = current_directory.get_items().len();
    let current_dir_size = format_folder_size(current_directory.size());

    cx.render(rsx! {
        div {
            dir_paths.read().iter().map(|current_dir_path| {
                let directory = match root_directory.get_item_by_path(&current_dir_path.to_str().unwrap_or_default())
                .and_then(|item| item.get_directory()) {
                    Ok(dir) => {dir},
                    _ => root_directory.clone(),
                };
                let dir_name = directory.name().clone();
                let dir_id = directory.id().clone();
                
                if dir_id != root_dir_id {
                    rsx! (
                        h5 {
                            margin_left: "8px",
                            display: "inline-block",
                            ">"},
                        h5 {
                        class: "dir_paths_navigation",
                        margin_left: "8px",
                        display: "inline-block",
                        onclick: move |_| lib::go_back_dirs_with_loop(cx.clone(), dir_id),
                      "{dir_name}"
                    })
                } else {
                    rsx!(
                        div {
                            class: "dir_paths_navigation",
                            margin_left: "8px",
                            padding_top: "4px",
                            display: "inline-block",
                            onclick: move |_| lib::go_back_dirs_with_loop(cx.clone(), dir_id),
                            Icon {
                                icon: Shape::Home
                            }
                        }
                    )
                } 
            },
        ),
        }
        label {
            margin_left: "8px",
            "{current_dir_size} / {current_dir_items_len} item(s)"
            },
        div {
         id: "browser",
            (cx.props.show_new_folder).then(|| 
                rsx!(
                    
                div {
                    class: "item file",
                    NewFolder {
                        state: State::Primary,
                        storage: cx.props.storage.clone(),
                        show_new_folder: cx.props.show_new_folder.clone(),
                    }
                }
            )
            ),
            files_sorted.iter().filter(|item| item.item_type() == ItemType::DirectoryItem).map(|directory| {
                let key = directory.id();
                let (dir_items_len, dir_size) =  if let Ok(dir) = directory.get_directory() {
                    (dir.get_items().len(), dir.size())
                } else {
                    (0, 0)
                };
                    rsx!{
                         div {
                            key: "{key}-placeholder",
                        }
                        Folder {
                            key: "{key}"
                            name: directory.name(),
                            state: State::Primary,
                            id: key.to_string(),
                            size: dir_size,
                            children: dir_items_len,
                            storage: cx.props.storage.clone(),
                            update_current_dir: update_current_dir.clone(),
                        }}
               
            })
            files_sorted.iter().filter(|item| item.item_type() == ItemType::FileItem).map(|file| {
                let file_extension = std::path::Path::new(&file.name())
                    .extension()
                    .unwrap_or_else(|| std::ffi::OsStr::new(""))
                    .to_str()
                    .unwrap()
                    .to_string();

                let key = file.id();

                rsx!(
                    div {
                        onclick: move |_| {
                            if *cx.props.show_new_folder {
                                cx.props.show_new_folder.set(false);
                            }
                        },
                        File {
                            key: "{key}",
                            name: file.name(),
                            state: State::Secondary,
                            id: key.to_string(),
                            kind: file_extension,
                            size: file.size(),
                            thumbnail: file.thumbnail(),
                            storage: cx.props.storage.clone(),
                        } 
                    }
                   )
            })
        }
    })
}


fn format_folder_size(folder_size: usize) -> String {
    if folder_size == 0 {
        return String::from("0 bytes");
    }
    let base_1024: f64 = 1024.0;
    let size_f64: f64 = folder_size as f64;

    let i = (size_f64.log10() / base_1024.log10()).floor();
    let size_formatted = size_f64 / base_1024.powf(i);

    let file_size_suffix = ["bytes", "KB", "MB", "GB", "TB"][i as usize];
    let mut size_formatted_string = format!(
        "{size:.*} {size_suffix}",
        1,
        size = size_formatted,
        size_suffix = file_size_suffix
    );
    if size_formatted_string.contains(".0") {
        size_formatted_string = size_formatted_string.replace(".0", "");
    }
    size_formatted_string
}

