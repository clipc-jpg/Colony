

use std::collections::HashMap;
use std::path::PathBuf;

use dioxus::prelude::*;
use itertools::Itertools;

use crate::pages::*;
use crate::components::*;
use crate::FrontendCommChannel;
use crate::backend::*;



#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SelectionValidity {
    Valid,
    Invalid
}

#[component]
pub fn RemoteDirTestPage(app_state: Signal<AppState>, comm_with_backend: Signal<FrontendCommChannel>) -> Element {

    let mut remote_dir_overlay_active = use_signal(|| false);

    rsx! {
        div {
            class: "file-picker background",
            // background_image: BACKGROUND_IMG,
            div {
                class: "file-picker logo-navbar-row",
                //"Logo-Navbar-Row"

                div {
                    class: "file-picker logo-row",
                    IMILogo { class: "navbar-logo" }
                    //p {"Logo-Row"}
                }
                div {
                    class: "file-picker large-navbar",
                    //p {"Navbar"}

                    button {
                        class: "file-picker primary-button",
                        onclick: move |_| {
                            remote_dir_overlay_active.toggle();
                        },
                        {"Choose Remote File"}
                    }
                    HomeButton {app_state: app_state, class: "file-picker"}
                }
            }
            //RemoteDirWidget { app_state, active: remote_dir_overlay_active }
            RemoteDirTestOverlay { app_state, active: remote_dir_overlay_active, comm_with_backend }
        }
    }
}

#[component]
pub fn RemoteDirTestOverlay(app_state: Signal<AppState>, active: Signal<bool>, comm_with_backend: Signal<FrontendCommChannel>) -> Element {
    rsx! {
        div {
            class: match active() {
                true => "file-picker overlay",
                false => "file-picker overlay hidden"
            },
            div {
                class: "file-picker file-picker-area",
                RemoteDirWidget { app_state, active, comm_with_backend }
            }
        }
    }
}

#[component]
pub fn RemoteDirWidget(app_state: Signal<AppState>, active: Signal<bool>, comm_with_backend: Signal<FrontendCommChannel>) -> Element {

    let chosen_layout = use_signal(|| FilePickerLayout::LargeTiles);


    let current_file_system = use_signal(|| FilesystemData::Local);
    let mut current_top_directory = use_signal(|| PathBuf::from("/"));

    let mut next_top_directory = use_signal(|| None as Option<PathBuf>);

    let previous_top_directories = use_signal(|| Vec::new() as Vec<PathBuf>);
    let forward_top_directories = use_signal(|| Vec::new() as Vec<PathBuf>);

    let mut files = use_signal(|| Vec::new() as Vec<String>);
    let mut directories_signal = use_signal(|| Vec::new() as Vec<String>);
    let mut error_popup_msg = use_signal(|| None as Option<String>);
    use_memo(move || {
        println!("Current top directory changed to: {:?}", current_top_directory());
        println!("Available directories to return to: {:?}", previous_top_directories());
        println!("Available directories to forward to: {:?}", forward_top_directories());
    });

    use_future(move || {
        async move {
            next_top_directory.set(Some(current_top_directory().clone()));
            loop {
                if let Some(nxt_top_directory) = next_top_directory() {
                    comm_with_backend.read().send(BackendRequest::ListDirectory(FilesystemData::LocalWSL, nxt_top_directory.clone())).ok();
                    next_top_directory.set(None);
                }

                let now = tokio::time::Instant::now();
                let dt = std::time::Duration::from_millis(200);
                tokio::time::sleep_until(now + dt).await;

                match comm_with_backend.read().try_receive() {
                    Ok(BackendResponse::ListDirectory(Ok(paths))) => {
                        //all_content.set(paths.read().iter().map(|pb| format!("{:?}", pb)).collect_vec())
                        println!("Received directory contents");
                        println!("Received files:");
                        for pth in paths.files.iter() { println!("\t{}", pth); }
                        println!("\nReceived directories:");
                        for pth in paths.directories.iter() { println!("\t{}", pth); }
                        println!("");
                        files.set(paths.files.iter().map(|pb| format!("{}", pb)).collect_vec());
                        directories_signal.set(paths.directories.iter().map(|pb| format!("{}", pb)).collect_vec());
                        current_top_directory.set(paths.root);
                    },
                    Ok(BackendResponse::ListDirectory(Err(_))) => {
                        let msg = "An Error occurred while opening the directory".to_string();
                        println!("{}", &msg);
                        error_popup_msg.set(Some(msg))
                    },
                    Err(_) => { continue; },
                    _ => { continue; }
                }
            }
        }
    });

    let mut selected_file_endings = use_signal(|| "".to_string());

    selected_file_endings.set("_file".to_string());

    let filtered_files = use_memo(move || {
        files().iter()
                .filter(|name| name.ends_with(&selected_file_endings()))
                .map(|s| s.clone())
                .collect_vec()
    });

    let other_files = use_memo(move || {
        files().iter()
                .filter(|name| !name.ends_with(&selected_file_endings()))
                .map(|s| s.clone())
                .collect_vec()
    });

    let mut file_markings = use_signal(move || {
        files().iter().chain(directories_signal().iter()).map(|nm| (nm.clone(), FileWidgetMarking::Unmarked))
                     .collect::<HashMap<_,_>>()
    });


    let directories = use_memo(move || {

        for pth in files().iter().chain(directories_signal().iter()) {
            file_markings.write().entry(pth.clone()).or_insert(FileWidgetMarking::Unmarked);
        }

        directories_signal()
    });



    //TODO: selection has to escape widget somehow
    //maybe send a backend request on exit

    let mut clicked_widgets = use_signal(|| Vec::new() as Vec<String>);

    let mut num_selections = use_signal(|| { 0 as usize });

    let is_selection_valid = use_memo(move || {
        match num_selections() {
            0 => SelectionValidity::Invalid,
            _ => SelectionValidity::Valid
        }
    });


    use_future(move || async move {
        loop {
            let now = tokio::time::Instant::now();
            let dt = std::time::Duration::from_millis(100);
            tokio::time::sleep_until(now + dt).await;


            //TODO: there is multiple use cases for the widget with different logic; needs to be implemented

            let cur_num_selections = num_selections();

            #[allow(warnings)]
            for widget_name in clicked_widgets().iter() {
                println!("{} has been clicked", widget_name);
                *file_markings.write().get_mut(widget_name).unwrap() = match file_markings().get(widget_name).unwrap() {
                    FileWidgetMarking::Unmarked    => {
                        *num_selections.write_silent() += 1;
                        FileWidgetMarking::Marked
                    },
                    FileWidgetMarking::Marked      => {
                        *num_selections.write_silent() -= 1;
                        FileWidgetMarking::Unmarked
                    },
                    FileWidgetMarking::Checked     => {
                        *num_selections.write_silent() -= 1;
                        FileWidgetMarking::Unmarked
                    },
                    FileWidgetMarking::ToBeDeleted => {
                        *num_selections.write_silent() -= 1;
                        FileWidgetMarking::Unmarked
                    },
                }
            }
            clicked_widgets.write().clear();
            if cur_num_selections != num_selections() {
                num_selections.write();
            }
        }
    });

    rsx! {
        div {
            class: "file-picker top-row",
            CurrentPositionWidget { current_file_system, current_top_directory, next_top_directory, previous_top_directories, forward_top_directories }
            ChooseLayoutIcon { layout_choice: chosen_layout }
        }
        div {
            class: "file-picker display-area-overflow-parent",
            div {
                class: "file-picker display-area",
                TopFileSection { files: filtered_files, file_markings, clicked_widgets }
                div { class: "file-picker separator" }
                DirectorySection { directories, file_markings, clicked_widgets, current_directory: current_top_directory, next_directory: next_top_directory, previous_top_directories, forward_top_directories }
                div { class: "file-picker separator" }
                OtherFileSection { files: other_files, file_markings, clicked_widgets }
            }
            div {
                class: "file-picker confirm-cancel-button-container",
                button {
                    class: match is_selection_valid() {
                        SelectionValidity::Invalid => format!("file-picker confirm-selection-button primary-button disabled-button"),
                        SelectionValidity::Valid => format!("file-picker confirm-selection-button primary-button")
                    },
                    onclick: move |_| {
                        match num_selections() {
                            0 => println!("Clicked, but disabled. Selected items: {}", num_selections()),
                            _ => {
                                println!("Clicked, and enabled. Selected items: {}", num_selections());
                                active.set(false);
                            }
                        }
                    },
                    {"Confirm Selection"}
                }
                button {
                    class: "file-picker cancel-selection-button primary-button",
                    onclick: move |_| {
                        // different functionality!
                        //for val in file_markings.write().values_mut() {
    //                        *val = FileWidgetMarking::Unmarked
    //                    }
    //                    num_selections.set(0);

                        for mark in file_markings.write().values_mut() {
                            *mark = FileWidgetMarking::Unmarked;
                        }
                        active.set(false);
                    },
                    {"Cancel Selection"}
                }
            }
        }
    }
}


//################################################################################
//## Top Row Components
//################################################################################


//TODO: if the filesystem gets changed, top directory has to change, too
// then, a request needs to be send to the backend
// the backend responds with the default top directory (or last visited directory, optionally with a context)
#[component]            //TODO: Avoid primitive obsession
pub fn CurrentPositionWidget(current_file_system: Signal<FilesystemData>,
                             current_top_directory: Signal<PathBuf>,
                             next_top_directory: Signal<Option<PathBuf>>,
                             previous_top_directories: Signal<Vec<PathBuf>>,
                             forward_top_directories: Signal<Vec<PathBuf>>) -> Element {


    let mut current_directory_hovered = use_signal(|| false);
    // needs to be moved outside, such that a new overlay can be created
    let mut filesystem_combobox_opened = use_signal(|| false);
    let filesystem_button_rotation = use_memo(move || {
        if filesystem_combobox_opened() { "rotate(0deg)" } else { "rotate(180deg)" }
    });

    let return_button_active = use_memo(move || !previous_top_directories().is_empty());
    let forward_button_active = use_memo(move || !forward_top_directories().is_empty());
    let ascend_button_active =  use_memo(move || {
        let cur_top_dir = current_top_directory();
        let mut cmps = cur_top_dir.components();
        let cmp1 = cmps.next();
        let cmp2 = cmps.next();
        cmp2.is_some() || cmp1.is_some_and(|cmp| cmp != std::path::Component::RootDir)
        //current_top_directory().components().count()>1 ||
//        current_top_directory().components().nth(0).is_some_and(|cmp| cmp != std::path::Component::RootDir)
    });

    let fake_filesystems = use_signal(|| {
        vec![
             FilesystemData::Remote( RemoteFileSystemData { address: "srdtcfzhvgublinjmkjhgfdxsytexrdhtcfjgvzbhnjmkl".to_string() } ),
             FilesystemData::Local,
             FilesystemData::LocalWSL,
             FilesystemData::Local,
             FilesystemData::LocalWSL,
             FilesystemData::Local,
             FilesystemData::LocalWSL,
             FilesystemData::Local,
             FilesystemData::LocalWSL,
             FilesystemData::Local,
             FilesystemData::LocalWSL,
             FilesystemData::Local,
             FilesystemData::LocalWSL,
             FilesystemData::Local,
             FilesystemData::LocalWSL,
             FilesystemData::Local,
             FilesystemData::LocalWSL,
             FilesystemData::Local,
             FilesystemData::LocalWSL,
             FilesystemData::Local,
             FilesystemData::LocalWSL,
             FilesystemData::Local,
             FilesystemData::LocalWSL,
             FilesystemData::Local,
             FilesystemData::LocalWSL,
             FilesystemData::Remote( RemoteFileSystemData { address: "srdtcfzhvgublinjmkjhgfdxsytexrdhtcfjgvzbhnjmkl".to_string() } )
            ]
    });

    rsx! {
        div {
            class: "file-picker current-position-container",

            div {
                class: "file-picker current-position-filesystem-container",
                h4 {
                    class: "file-picker current-position-filesystem-title",
                    "Current Filesystem:"
                }
                div {
                    //class: "file-picker file-system-combobox-anchor-container",
                    class: match filesystem_combobox_opened() {
                        true =>  "file-picker file-system-combobox-anchor-container",
                        false =>  "file-picker file-system-combobox-anchor-container hidden",
                    },
                    FilesystemCombobox { current_file_system, registered_file_systems: fake_filesystems, opened: filesystem_combobox_opened }
                }
                div {
                    //class: "file-picker current-filesystem-name-container",
                    class: match filesystem_combobox_opened() {
                        true =>  "file-picker current-filesystem-name-container hidden",
                        false =>  "file-picker current-filesystem-name-container",
                    },
                    FileSystemName { filesystem: current_file_system() }
                }
                div {
                    class: "file-picker choose-filesystem-icon-container",
                    transform: filesystem_button_rotation(),
                    svg {
                        onclick: move |_| {
                            //println!("Clicked!");
                            filesystem_combobox_opened.toggle();
                        },
                        fill: "currentColor",
                        class: "file-picker choose-filesystem-icon bi bi-triangle-fill",
                        view_box: "0 0 16 16",
                        path {
                            fill_rule: "evenodd",
                            d: "M7.022 1.566a1.13 1.13 0 0 1 1.96 0l6.857 11.667c.457.778-.092 1.767-.98 1.767H1.144c-.889 0-1.437-.99-.98-1.767z"
                        }
                    }
                }
            }
            div {
                class: "file-picker current-position-widget-container",
                div {
                    //class: "file-picker current-position-go-back-button-container icon-background small-icon",
                    class: match return_button_active() {
                        true => "file-picker current-position-go-back-button-container icon-background small-icon",
                        false => "file-picker current-position-go-back-button-container icon-background small-icon disabled",
                    },
                    svg {
                        width: "16",
                        height: "16",
                        fill: "currentColor",
                        class: "file-picker current-position-go-back-button small-icon bi bi-arrow-left",
                        onclick: move |_| {
                            if return_button_active() {
                                if let Some(prev_dir) = previous_top_directories.write().pop() {
                                    match next_top_directory() {
                                        Some(_) => {},
                                        None => {
                                            forward_top_directories.write().push(current_top_directory().clone());
                                            next_top_directory.set(Some(prev_dir));
                                        }
                                    }
                                }
                            }
                        },
                        view_box: "0 0 16 16",
                        path {
                            fill_rule: "evenodd",
                            d: "M15 8a.5.5 0 0 0-.5-.5H2.707l3.147-3.146a.5.5 0 1 0-.708-.708l-4 4a.5.5 0 0 0 0 .708l4 4a.5.5 0 0 0 .708-.708L2.707 8.5H14.5A.5.5 0 0 0 15 8"
                        }
                    }
                }
                div {
                    //class: "file-picker current-position-go-forward-button-container icon-background small-icon",
                    class: match forward_button_active() {
                        true => "file-picker current-position-go-back-button-container icon-background small-icon",
                        false => "file-picker current-position-go-back-button-container icon-background small-icon disabled",
                    },
                    svg {
                        width: "16",
                        height: "16",
                        fill: "currentColor",
                        class: "file-picker current-position-go-forward-button small-icon bi bi-arrow-right",
                        onclick: move |_| {
                            if forward_button_active() {
                                if let Some(forw_dir) = forward_top_directories.write().pop() {
                                    match next_top_directory() {
                                        Some(_) => {},
                                        None => {
                                            previous_top_directories.write().push(current_top_directory().clone());
                                            next_top_directory.set(Some(forw_dir));
                                        }
                                    }
                                }
                            }
                        },
                        view_box: "0 0 16 16",
                        path {
                            fill_rule: "evenodd",
                            d: "M1 8a.5.5 0 0 1 .5-.5h11.793l-3.147-3.146a.5.5 0 0 1 .708-.708l4 4a.5.5 0 0 1 0 .708l-4 4a.5.5 0 0 1-.708-.708L13.293 8.5H1.5A.5.5 0 0 1 1 8"
                        }
                    }
                }
                div {
                    //class: "file-picker current-position-ascend-directory-button-container icon-background small-icon",
                    class: match ascend_button_active() {
                        true => "file-picker current-position-go-back-button-container icon-background small-icon",
                        false => "file-picker current-position-go-back-button-container icon-background small-icon disabled",
                    },
                    svg {
                        width: "16",
                        height: "16",
                        fill: "currentColor",
                        class: "file-picker current-position-ascend-directory-button small-icon bi bi-arrow-up",
                        onclick: move |_| {
                            if ascend_button_active() {
                                let prev_dir = current_top_directory().clone();
                                let mut nxt_dir = prev_dir.clone();
                                if nxt_dir.pop() {
                                    match next_top_directory() {
                                        Some(_) => {},
                                        None => {
                                            previous_top_directories.write().push(prev_dir);
                                            forward_top_directories.write().clear();
                                            next_top_directory.set(Some(nxt_dir));
                                        }
                                    }
                                }
                            }
                        },
                        view_box: "0 0 16 16",
                        path {
                            fill_rule: "evenodd",
                            d: "M8 15a.5.5 0 0 0 .5-.5V2.707l3.146 3.147a.5.5 0 0 0 .708-.708l-4-4a.5.5 0 0 0-.708 0l-4 4a.5.5 0 1 0 .708.708L7.5 2.707V14.5a.5.5 0 0 0 .5.5"
                        }
                    }
                }
                div {
                    class: "file-picker current-position-reload-directory-button-container icon-background small-icon",
                    svg {
                        width: "16",
                        height: "16",
                        fill: "currentColor",
                        class: "file-picker current-position-reload-directory-button small-icon bi bi-arrow-clockwise",
                        onclick: move |_| {
                            match next_top_directory() {
                                Some(_) => {},
                                None => {
                                    next_top_directory.set(Some(current_top_directory().clone()));
                                }
                            }
                        },
                        view_box: "0 0 16 16",
                        path {
                            fill_rule: "evenodd",
                            d: "M8 3a5 5 0 1 0 4.546 2.914.5.5 0 0 1 .908-.417A6 6 0 1 1 8 2z"
                        }
                        path {
                            d: "M8 4.466V.534a.25.25 0 0 1 .41-.192l2.36 1.966c.12.1.12.284 0 .384L8.41 4.658A.25.25 0 0 1 8 4.466"
                        }
                    }
                }
                h4 {
                   // class: "file-picker current-position-widget-directory",
                    class: match current_directory_hovered() {
                        true => "file-picker current-position-widget-directory",
                        false => "file-picker current-position-widget-directory clipped"
                    },
                    onmouseenter: move |_| current_directory_hovered.set(true),
                    onmouseleave: move |_| current_directory_hovered.set(false),

                    if current_directory_hovered() {
                        div {
                            class: "file-picker current-position-full-path-anchor-container",
                            div {
                                class: "file-picker current-position-full-path",
                                MultiLinePathString { path: current_top_directory().clone() }
                                //{current_top_directory().to_string_lossy().to_string()}
                            }
                        }
                    } else {
                        //div {
//                            class: "file-picker current-position-last-path-component",
//                            {current_top_directory().iter().last().unwrap_or_default().to_string_lossy().to_string()}
//                            //{current_top_directory().to_string_lossy().to_string()}
//                        }
                        {current_top_directory().iter().last().unwrap_or_default().to_string_lossy().to_string()}
                    }
                }
            }
        }
    }
}


#[component]
pub fn FilesystemCombobox(current_file_system: Signal<FilesystemData>, registered_file_systems: Signal<Vec<FilesystemData>>, opened: Signal<bool>) -> Element {
    rsx! {
        div {
            class: match opened() {
                true => {"file-picker current-position-popup container"},
                false => {"file-picker current-position-popup container hidden"}
            },
            {registered_file_systems().iter().enumerate().map(|(k,fs)| {
                rsx! {
                    div {
                        class: if current_file_system() == *fs {
                            "file-picker file-system-title selected"
                        } else {
                            "file-picker file-system-title"
                        },
                        onclick: move |_| {
                            current_file_system.set(registered_file_systems()[k].clone())
                        },
                        FileSystemName { filesystem: registered_file_systems()[k].clone() }
                    }
                }
            })}
        }
    }
}

#[component]
//pub fn FileSystemName(filesystem_index: usize, registered_file_systems: Signal<Vec<FilesystemData>>) -> Element {
pub fn FileSystemName(filesystem: FilesystemData) -> Element {
    rsx! {
        match &filesystem {
            FilesystemData::Local => { "Local" },
            FilesystemData::LocalWSL => { "WSL" },
            FilesystemData::Remote(_remote_file_system_data) => {"Remote via command line"},
            FilesystemData::RemoteSSHFS(_remote_sshfs_data) => {"Remote mounted via SSHFS"},
            FilesystemData::HelixSSH(_helix_ssh_data) => {"BWForClusterHelix via command line"},
            FilesystemData::SDSHD(_sds_sshfs_data) => {"SDS HD mounted via SSHFS"},
        }
    }
}



#[derive(Debug, Clone, Copy)]
pub enum FilePickerLayout {
    LargeTiles,
    List,
    Widget
}

#[component]
pub fn ChooseLayoutIcon(layout_choice: Signal<FilePickerLayout>) -> Element {
    rsx! {
        div {
            class: "file-picker choose-layout-icon",
            onclick: move |_| {
                layout_choice.set(
                    match layout_choice() {
                        FilePickerLayout::LargeTiles => FilePickerLayout::List,
                        FilePickerLayout::List => FilePickerLayout::Widget,
                        FilePickerLayout::Widget => FilePickerLayout::LargeTiles
                    }
                );
            },
            match layout_choice() {
                FilePickerLayout::LargeTiles => rsx! {
                    "Large" wbr{} "Tiles"
                    svg {
                        width: "16",
                        height: "16",
                        fill: "currentColor",
                        class: "file-picker choose-layout-icon medium-icon bi bi-layout-three-columns",
                        view_box: "0 0 16 16",
                        path {
                            d: "M0 1.5A1.5 1.5 0 0 1 1.5 0h13A1.5 1.5 0 0 1 16 1.5v13a1.5 1.5 0 0 1-1.5 1.5h-13A1.5 1.5 0 0 1 0 14.5zM1.5 1a.5.5 0 0 0-.5.5v13a.5.5 0 0 0 .5.5H5V1zM10 15V1H6v14zm1 0h3.5a.5.5 0 0 0 .5-.5v-13a.5.5 0 0 0-.5-.5H11z"
                        }
                    }
                },
                FilePickerLayout::List => rsx! {
                    "List"
                    svg {
                        width: "16",
                        height: "16",
                        fill: "currentColor",
                        class: "file-picker choose-layout-icon medium-icon bi bi-layout-text-window",
                        view_box: "0 0 16 16",
                        path {
                            d: "M3 6.5a.5.5 0 0 1 .5-.5h5a.5.5 0 0 1 0 1h-5a.5.5 0 0 1-.5-.5m0 3a.5.5 0 0 1 .5-.5h5a.5.5 0 0 1 0 1h-5a.5.5 0 0 1-.5-.5m.5 2.5a.5.5 0 0 0 0 1h5a.5.5 0 0 0 0-1z"
                        }
                        path {
                            d: "M2 0a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V2a2 2 0 0 0-2-2zm12 1a1 1 0 0 1 1 1v1H1V2a1 1 0 0 1 1-1zm1 3v10a1 1 0 0 1-1 1h-2V4zm-4 0v11H2a1 1 0 0 1-1-1V4z"
                        }
                    }
                },
                FilePickerLayout::Widget => rsx! {
                    "Widget-" wbr{} "Like"
                    svg {
                        width: "16",
                        height: "16",
                        fill: "currentColor",
                        class: "file-picker choose-layout-icon medium-icon bi bi-layout-sidebar",
                        view_box: "0 0 16 16",
                        path {
                            d: "M0 3a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H2a2 2 0 0 1-2-2zm5-1v12h9a1 1 0 0 0 1-1V3a1 1 0 0 0-1-1zM4 2H2a1 1 0 0 0-1 1v10a1 1 0 0 0 1 1h2z"
                        }
                    }
                }
            }
        }
    }
}


//################################################################################
//## File/Directory Sections
//################################################################################

#[component]
pub fn TopFileSection(files: Memo<Vec<String>>, file_markings: Signal<HashMap<String, FileWidgetMarking>>, clicked_widgets: Signal<Vec<String>>) -> Element {

    let file_naming: fn(String) -> String = |nm| format!("{}", nm);

    rsx! {
        div {
            class: "file-picker filtered-file-area",
            {files().iter().map(|nm| {

                rsx! {
                    FileWidget {
                        class: "file-picker".to_string(),
                        id: nm.clone(), name: file_naming,
                        state: #[allow(warnings)]  {
                            *file_markings.write_silent().entry(nm.clone()).or_insert(FileWidgetMarking::Unmarked)
                        },
                        clicked_widgets
                    }
                }
            })}
        }
    }
}

#[component]
pub fn DirectorySection(directories: Memo<Vec<String>>,
                        file_markings: Signal<HashMap<String, FileWidgetMarking>>,
                        clicked_widgets: Signal<Vec<String>>,
                        current_directory: Signal<PathBuf>,
                        next_directory: Signal<Option<PathBuf>>,
                        previous_top_directories: Signal<Vec<PathBuf>>,
                        forward_top_directories: Signal<Vec<PathBuf>>) -> Element {

    let dir_naming: fn(String) -> String = |nm| format!("{}", nm);
    rsx! {
        div {
            class: "file-picker directory-area",
            {directories().iter().map(|nm| {
                rsx! {
                    DirectoryWidget {
                        class: "file-picker".to_string(),
                        id: nm.clone(), name: dir_naming,
                        state: #[allow(warnings)]  {
                            *file_markings.write_silent().entry(nm.clone()).or_insert(FileWidgetMarking::Unmarked)
                        },
                        clicked_widgets,
                        current_directory,
                        next_directory,
                        previous_directories: previous_top_directories,
                        forward_directories: forward_top_directories
                    }
                }
            })}
        }
    }
}

#[component]
pub fn OtherFileSection(files: Memo<Vec<String>>, file_markings: Signal<HashMap<String, FileWidgetMarking>>, clicked_widgets: Signal<Vec<String>>) -> Element {

    let file_naming: fn(String) -> String = |nm| format!("{}", nm);
    rsx! {
        div {
            class: "file-picker other-file-area",
            {files().iter().map(|nm| {
                rsx! {
                    FileWidget {
                        class: "file-picker".to_string(),
                        id: nm.clone(), name: file_naming,
                        state: #[allow(warnings)]  {
                            *file_markings.write_silent().entry(nm.clone()).or_insert(FileWidgetMarking::Unmarked)
                        },
                        clicked_widgets
                    }
                }
            })}
        }
    }
}


#[component]
pub fn WidgetFileDirectorySection(files: Memo<Vec<String>>, directories: Memo<Vec<String>>,
                                  file_markings: Signal<HashMap<String, FileWidgetMarking>>,
                                  clicked_widgets: Signal<Vec<String>>,
                                  current_directory: Signal<PathBuf>,
                                  next_directory: Signal<Option<PathBuf>>,
                                  previous_top_directories: Signal<Vec<PathBuf>>,
                                  forward_top_directories: Signal<Vec<PathBuf>>) -> Element {

    let file_naming: fn(String) -> String = |nm| format!("{}", nm);
    let dir_naming: fn(String) -> String = |nm| format!("{}", nm);

    rsx! {
        div {
            class: "file-picker directory-area",
            {directories().iter().map(|nm| {
                rsx! {
                    DirectoryWidget {
                        class: "file-picker".to_string(),
                        id: nm.clone(), name: dir_naming,
                        state: #[allow(warnings)]  {
                            *file_markings.write_silent().entry(nm.clone()).or_insert(FileWidgetMarking::Unmarked)
                        },
                        clicked_widgets,
                        current_directory,
                        next_directory,
                        previous_directories: previous_top_directories,
                        forward_directories: forward_top_directories
                    }
                }
            })}
            {files().iter().map(|nm| {
                rsx! {
                    FileWidget {
                        class: "file-picker".to_string(),
                        id: nm.clone(), name: file_naming,
                        state: #[allow(warnings)]  {
                            *file_markings.write_silent().entry(nm.clone()).or_insert(FileWidgetMarking::Unmarked)
                        },
                        clicked_widgets
                    }
                }
            })}
        }
    }
}



