

use std::time::Duration;
use std::path::{Path, PathBuf};

use dioxus::prelude::*;

use crate::{BackendRequest, BackendResponse, FrontendCommChannel};




#[component]
pub fn RemoteFilesystemWidget() -> Element {
    rsx! {

    }
}

//################################################################################
//## UI elements
//################################################################################

#[allow(unused)]
#[derive(Clone, Copy, PartialEq)]
pub enum FileWidgetMarking {
    Unmarked,
    Marked,
    Checked,
    ToBeDeleted,
}

#[component]
pub fn DirectoryWidget<F: 'static + PartialEq>(class: String, id: String, name: F,
                                               state: FileWidgetMarking,
                                               clicked_widgets: Signal<Vec<String>>,
                                               current_directory: Signal<PathBuf>,
                                               next_directory: Signal<Option<PathBuf>>,
                                               previous_directories: Signal<Vec<PathBuf>>,
                                               forward_directories: Signal<Vec<PathBuf>>) -> Element
    where F: Fn(String) -> String
{

    let mut hovered = use_signal(|| false);

    let id_clone = id.clone();
    let id_clone2 = id.clone();

    rsx! {
        div {
            class: match state {
                FileWidgetMarking::Unmarked    => format!("{class} directory-widget unmarked container"),
                FileWidgetMarking::Marked      => format!("{class} directory-widget marked container"),
                FileWidgetMarking::Checked     => format!("{class} directory-widget checked container"),
                FileWidgetMarking::ToBeDeleted => format!("{class} directory-widget to-be-deleted container"),
            },
            onmouseenter: move |_| {
                hovered.set(true);
            },
            onmouseleave: move |_| {
                hovered.set(false);
            },
            div {
                //class: format!("{class} directory-widget icon-container"),
                class: match (state, hovered()) {
                    (FileWidgetMarking::Unmarked, _)        => format!("{class} directory-widget icon-container unmarked"),
                    (FileWidgetMarking::Marked, false)      => format!("{class} directory-widget icon-container marked"),
                    (FileWidgetMarking::Marked, true)       => format!("{class} directory-widget icon-container marked shrunk"),
                    (FileWidgetMarking::Checked, false)     => format!("{class} directory-widget icon-container checked"),
                    (FileWidgetMarking::Checked, true)      => format!("{class} directory-widget icon-container checked shrunk"),
                    (FileWidgetMarking::ToBeDeleted, false) => format!("{class} directory-widget icon-container to-be-deleted"),
                    (FileWidgetMarking::ToBeDeleted, true)  => format!("{class} directory-widget icon-container to-be-deleted shrunk"),
                },
                onclick: move |_| {
                    let nm_clone = id.clone();
                    clicked_widgets.write().push(nm_clone)
                },
                div {
                    class: format!("{class} directory-widget marker-container"),
                    match state {
                        FileWidgetMarking::Unmarked    => rsx! {UnmarkedIcon { class: class.clone() }},
                        FileWidgetMarking::Marked      => rsx! {MarkedIcon { class: class.clone() }},
                        FileWidgetMarking::Checked     => rsx! {CheckedIcon { class: class.clone() }},
                        FileWidgetMarking::ToBeDeleted => rsx! {ToBeDeletedIcon { class: class.clone() }},
                    }

                }
                svg {
                    width: "16",
                    height: "16",
                    fill: "currentColor",
                    class: format!("{class} directory-widget medium-icon bi bi-folder-fill"),
                    view_box: "0 0 16 16",
                    path {
                        d: "M9.828 3h3.982a2 2 0 0 1 1.992 2.181l-.637 7A2 2 0 0 1 13.174 14H2.825a2 2 0 0 1-1.991-1.819l-.637-7a2 2 0 0 1 .342-1.31L.5 3a2 2 0 0 1 2-2h3.672a2 2 0 0 1 1.414.586l.828.828A2 2 0 0 0 9.828 3m-8.322.12q.322-.119.684-.12h5.396l-.707-.707A1 1 0 0 0 6.172 2H2.5a1 1 0 0 0-1 .981z"
                    }
                }
                h4 {
                    class: format!("{class} directory-widget name"),
                    {name(id_clone)}
                }
            }
            div {
                //class: format!("{class} directory-widget descend-icon-area"),
                class: match (state, hovered()) {
                    //(FileWidgetMarking::Unmarked,_) => format!("{class} directory-widget descend-icon-area hidden"),
                    (_, false)                      => format!("{class} directory-widget descend-icon-area hidden"),
                    (_, true)                       => format!("{class} directory-widget descend-icon-area"),
                },
                onclick: move |_| {
                    match next_directory() {
                        Some(_) => {
                            println!("Next directory is already set; No action taken");
                        },
                        None => {
                            let nm_clone = id_clone2.clone();
                            println!("Appending to path: {}", &nm_clone);
                            let mut cur_dir = current_directory().clone();
                            previous_directories.write().push(cur_dir.clone());
                            forward_directories.write().clear();
                            cur_dir.push(nm_clone);
                            next_directory.set(Some(cur_dir));
                            println!("Changed current directory to: {:?}", current_directory());
                        }
                    }
                },
                svg {
                    width: "16",
                    height: "16",
                    fill: "currentColor",
                    class: "bi bi-box-arrow-in-down",
                    view_box: "0 0 16 16",
                    path {
                        fill_rule: "evenodd",
                        d: "M3.5 6a.5.5 0 0 0-.5.5v8a.5.5 0 0 0 .5.5h9a.5.5 0 0 0 .5-.5v-8a.5.5 0 0 0-.5-.5h-2a.5.5 0 0 1 0-1h2A1.5 1.5 0 0 1 14 6.5v8a1.5 1.5 0 0 1-1.5 1.5h-9A1.5 1.5 0 0 1 2 14.5v-8A1.5 1.5 0 0 1 3.5 5h2a.5.5 0 0 1 0 1z"
                    }
                    path {
                        fill_rule: "evenodd",
                        d: "M7.646 11.854a.5.5 0 0 0 .708 0l3-3a.5.5 0 0 0-.708-.708L8.5 10.293V1.5a.5.5 0 0 0-1 0v8.793L5.354 8.146a.5.5 0 1 0-.708.708z"
                    }
                }
            }
        }
    }
}


#[component]
pub fn FileWidget<F: 'static + PartialEq>(class: String, id: String, name: F, state: FileWidgetMarking, clicked_widgets: Signal<Vec<String>>) -> Element
    where F: Fn(String) -> String
{

    let id_clone = id.clone();

    rsx! {
        div {
            class: match state {
                FileWidgetMarking::Unmarked    => format!("container {class} file-widget unmarked"),
                FileWidgetMarking::Marked      => format!("container {class} file-widget marked"),
                FileWidgetMarking::Checked     => format!("container {class} file-widget checked"),
                FileWidgetMarking::ToBeDeleted => format!("container {class} file-widget to-be-deleted"),
            },
            onclick: move |_| {
                let nm_clone = id.clone();
                clicked_widgets.write().push(nm_clone)
            },
            div {
                class: "file-widget marker-container",
                match state {
                    FileWidgetMarking::Unmarked    => rsx! {UnmarkedIcon { class: class.clone() }},
                    FileWidgetMarking::Marked      => rsx! {MarkedIcon { class: class.clone() }},
                    FileWidgetMarking::Checked     => rsx! {CheckedIcon { class: class.clone() }},
                    FileWidgetMarking::ToBeDeleted => rsx! {ToBeDeletedIcon { class: class.clone() }},
                }
            }
            svg {
                width: "16",
                height: "16",
                fill: "currentColor",
                class: format!("{class} medium-icon bi bi-file-earmark-fill"),
                view_box: "0 0 16 16",
                path {
                    d: "M4 0h5.293A1 1 0 0 1 10 .293L13.707 4a1 1 0 0 1 .293.707V14a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V2a2 2 0 0 1 2-2m5.5 1.5v2a1 1 0 0 0 1 1h2z"
                }
            }
            h4 {
                class: format!("name {class}"),
                {name(id_clone)}
            }
        }
    }
}

#[component]
pub fn UnmarkedIcon(class: String) -> Element {
    return rsx! {};
}

#[component]
pub fn MarkedIcon(class: String) -> Element {
    return rsx! {};
}

#[component]
pub fn CheckedIcon(class: String) -> Element {
    rsx! {
        svg {
            width: "16",
            height: "16",
            fill: "currentColor",
            class: format!("checked-icon bi bi-check {class}"),
            view_box: "0 0 16 16",
            path {
                d: "M10.97 4.97a.75.75 0 0 1 1.07 1.05l-3.99 4.99a.75.75 0 0 1-1.08.02L4.324 8.384a.75.75 0 1 1 1.06-1.06l2.094 2.093 3.473-4.425z"
            }
        }
    }
}

#[component]
pub fn ToBeDeletedIcon(class: String) -> Element {
    rsx! {
        svg {
            width: "16",
            height: "16",
            fill: "currentColor",
            class: format!("to-be-deleted-icon bi bi-x {class}"),
            view_box: "0 0 16 16",
            path {
                d: "M4.646 4.646a.5.5 0 0 1 .708 0L8 7.293l2.646-2.647a.5.5 0 0 1 .708.708L8.707 8l2.647 2.646a.5.5 0 0 1-.708.708L8 8.707l-2.646 2.647a.5.5 0 0 1-.708-.708L7.293 8 4.646 5.354a.5.5 0 0 1 0-.708"
            }
        }
    }
}


//################################################################################
//## taken from dioxus example; will be replaced
//################################################################################

#[allow(unused)]
struct LinuxFileTree {
    path_stack: Vec<String>,
    path_names: Vec<String>,
    err: Option<String>,
}

#[allow(unused)]
impl LinuxFileTree {
    fn new() -> Self {
        let files = Self {
            path_stack: vec!["./".to_string()],
            path_names: vec![],
            err: None,
        };

        //files.reload_path_list();

        files
    }

    fn reload_path_list(&mut self) {
        let cur_path = self.path_stack.last().unwrap();
        let paths = match std::fs::read_dir(cur_path) {
            Ok(e) => e,
            Err(err) => {
                let err = format!("An error occurred: {err:?}");
                self.err = Some(err);
                self.path_stack.pop();
                return;
            }
        };
        let collected = paths.collect::<Vec<_>>();

        // clear the current state
        self.clear_err();
        self.path_names.clear();

        for path in collected {
            self.path_names
                .push(path.unwrap().path().display().to_string());
        }
    }

    fn go_up(&mut self) {
        if self.path_stack.len() > 1 {
            self.path_stack.pop();
        }
        //self.reload_path_list();
    }

    fn enter_dir(&mut self, dir_id: usize) {
        let path = &self.path_names[dir_id];
        self.path_stack.push(path.clone());
        //self.reload_path_list();
    }

    fn current(&self) -> &str {
        self.path_stack.last().unwrap()
    }
    fn clear_err(&mut self) {
        self.err = None;
    }
}

//################################################################################
//## replacement
//################################################################################

#[allow(unused)]
pub struct GenericLinuxFileTree<T: FileTreeInspector> {
    path_stack: Vec<String>,
    path_names: Vec<String>,
    err: Option<String>,
    inspector: T,
}

#[allow(unused)]
impl<T: FileTreeInspector> GenericLinuxFileTree<T> {
    pub fn new(inspector: T) -> Self {
        let files = Self {
            path_stack: vec!["./".to_string()],
            path_names: vec![],
            err: None,
            inspector
        };

        //files.reload_path_list();

        files
    }

    pub fn new_at(initial_path: String, inspector: T) -> Self {
        let files = Self {
            path_stack: vec![initial_path],
            path_names: vec![],
            err: None,
            inspector
        };

        //files.reload_path_list().await;

        files
    }

    pub async fn inspect<P: AsRef<Path>>(&self, dir: P) ->Result<Vec<String>,()> {
        return self.inspector.inspect(dir).await;
    }

    pub async fn reload_path_list(&mut self) {
        let cur_path = self.path_stack.last().unwrap();
        let paths = match self.inspect(cur_path).await {
            Ok(e) => e,
            Err(err) => {
                let err = format!("An error occurred: {err:?}");
                self.err = Some(err);
                self.path_stack.pop();
                return;
            }
        };
        let collected = paths;//.collect::<Vec<_>>();

        // clear the current state
        self.clear_err();
        self.path_names.clear();

        for path in collected {
            self.path_names
                .push(path);
        }
    }

    pub fn go_up(&mut self) {
        if self.path_stack.len() > 1 {
            self.path_stack.pop();
        }
        //self.reload_path_list().await;
    }

    pub fn enter_dir(&mut self, dir_id: usize) {
        let path = &self.path_names[dir_id];
        self.path_stack.push(path.clone());
        //self.reload_path_list().await;
    }

    pub fn current(&self) -> &str {
        self.path_stack.last().unwrap()
    }
    pub fn clear_err(&mut self) {
        self.err = None;
    }
}

#[allow(unused)]
pub trait FileTreeInspector {
    // TODO: Should String be wrapped into a new type
    async fn inspect<P: AsRef<Path>>(&self, dir: P) -> Result<Vec<String>,()>;
}

pub struct WSLinspector {
    comm_with_backend: FrontendCommChannel,
    timeout: Duration,
}

#[allow(unused)]
impl WSLinspector {
    pub fn new(comm_with_backend: FrontendCommChannel) -> Self {
        return WSLinspector { comm_with_backend, timeout: Duration::from_millis(500) };
    }

    pub fn new_with_timeout(comm_with_backend: FrontendCommChannel, timeout: Duration) -> Self {
        return WSLinspector { comm_with_backend, timeout };
    }
}

impl FileTreeInspector for WSLinspector {
    async fn inspect<P: AsRef<Path>>(&self, dir: P) -> Result<Vec<String>,()> {
        self.comm_with_backend.send(BackendRequest::InspectFilesystem(FilesystemData::LocalWSL, dir.as_ref().to_path_buf())).ok();

        let begin = tokio::time::Instant::now();
        loop {
            if begin.elapsed() > self.timeout { return Err(()); }
            let now = tokio::time::Instant::now();
            let dt = std::time::Duration::from_millis(100);
            tokio::time::sleep_until(now + dt).await;

            match self.comm_with_backend.try_receive() {
                Ok(BackendResponse::FileList(file_list)) => {
                    return Ok(file_list);
                },
                Err(_) => {},
                Ok(other) => {
                    self.comm_with_backend.reinsert_message(other).ok();
                }
            }
        }
    }
}

#[allow(unused)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilesystemData {
    Local,
    LocalWSL,
    Remote(RemoteFileSystemData),
    RemoteSSHFS(RemoteSSHfsData),
    HelixSSH(HelixSSHData),
    SDSHD(SDS_SSHfsData),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteFileSystemData {
    pub address: String
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteSSHfsData { }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelixSSHData { }

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SDS_SSHfsData { }

//################################################################################
//## svg candidates - DISPLAY FORMAT
//################################################################################


//svg {
//    width: "16",
//    height: "16",
//    fill: "currentColor",
//    class: "bi bi-grid-fill",
//    view_box: "0 0 16 16",
//    path {
//        d: "M1 2.5A1.5 1.5 0 0 1 2.5 1h3A1.5 1.5 0 0 1 7 2.5v3A1.5 1.5 0 0 1 5.5 7h-3A1.5 1.5 0 0 1 1 5.5zm8 0A1.5 1.5 0 0 1 10.5 1h3A1.5 1.5 0 0 1 15 2.5v3A1.5 1.5 0 0 1 13.5 7h-3A1.5 1.5 0 0 1 9 5.5zm-8 8A1.5 1.5 0 0 1 2.5 9h3A1.5 1.5 0 0 1 7 10.5v3A1.5 1.5 0 0 1 5.5 15h-3A1.5 1.5 0 0 1 1 13.5zm8 0A1.5 1.5 0 0 1 10.5 9h3a1.5 1.5 0 0 1 1.5 1.5v3a1.5 1.5 0 0 1-1.5 1.5h-3A1.5 1.5 0 0 1 9 13.5z"
//    }
//}

//svg {
//    width: "16",
//    height: "16",
//    fill: "currentColor",
//    class: "bi bi-grid-3x2-gap-fill",
//    view_box: "0 0 16 16",
//    path {
//        d: "M1 4a1 1 0 0 1 1-1h2a1 1 0 0 1 1 1v2a1 1 0 0 1-1 1H2a1 1 0 0 1-1-1zm5 0a1 1 0 0 1 1-1h2a1 1 0 0 1 1 1v2a1 1 0 0 1-1 1H7a1 1 0 0 1-1-1zm5 0a1 1 0 0 1 1-1h2a1 1 0 0 1 1 1v2a1 1 0 0 1-1 1h-2a1 1 0 0 1-1-1zM1 9a1 1 0 0 1 1-1h2a1 1 0 0 1 1 1v2a1 1 0 0 1-1 1H2a1 1 0 0 1-1-1zm5 0a1 1 0 0 1 1-1h2a1 1 0 0 1 1 1v2a1 1 0 0 1-1 1H7a1 1 0 0 1-1-1zm5 0a1 1 0 0 1 1-1h2a1 1 0 0 1 1 1v2a1 1 0 0 1-1 1h-2a1 1 0 0 1-1-1z"
//    }
//}

//svg {
//    width: "16",
//    height: "16",
//    fill: "currentColor",
//    class: "bi bi-grip-horizontal",
//    view_box: "0 0 16 16",
//    path {
//        d: "M2 8a1 1 0 1 1 0 2 1 1 0 0 1 0-2m0-3a1 1 0 1 1 0 2 1 1 0 0 1 0-2m3 3a1 1 0 1 1 0 2 1 1 0 0 1 0-2m0-3a1 1 0 1 1 0 2 1 1 0 0 1 0-2m3 3a1 1 0 1 1 0 2 1 1 0 0 1 0-2m0-3a1 1 0 1 1 0 2 1 1 0 0 1 0-2m3 3a1 1 0 1 1 0 2 1 1 0 0 1 0-2m0-3a1 1 0 1 1 0 2 1 1 0 0 1 0-2m3 3a1 1 0 1 1 0 2 1 1 0 0 1 0-2m0-3a1 1 0 1 1 0 2 1 1 0 0 1 0-2"
//    }
//}


// for displaying filenames only
//svg {
//    width: "16",
//    height: "16",
//    fill: "currentColor",
//    class: "bi bi-list-task",
//    view_box: "0 0 16 16",
//    path {
//        fill-rule: "evenodd",
//        d: "M2 2.5a.5.5 0 0 0-.5.5v1a.5.5 0 0 0 .5.5h1a.5.5 0 0 0 .5-.5V3a.5.5 0 0 0-.5-.5zM3 3H2v1h1z"
//    },
//    path {
//        d: "M5 3.5a.5.5 0 0 1 .5-.5h9a.5.5 0 0 1 0 1h-9a.5.5 0 0 1-.5-.5M5.5 7a.5.5 0 0 0 0 1h9a.5.5 0 0 0 0-1zm0 4a.5.5 0 0 0 0 1h9a.5.5 0 0 0 0-1z"
//    },
//    path {
//        fill-rule: "evenodd",
//        d: "M1.5 7a.5.5 0 0 1 .5-.5h1a.5.5 0 0 1 .5.5v1a.5.5 0 0 1-.5.5H2a.5.5 0 0 1-.5-.5zM2 7h1v1H2zm0 3.5a.5.5 0 0 0-.5.5v1a.5.5 0 0 0 .5.5h1a.5.5 0 0 0 .5-.5v-1a.5.5 0 0 0-.5-.5zm1 .5H2v1h1z"
//    }
//}


//################################################################################
//## svg candidates - FILES
//################################################################################


//file-check-fill

//svg {
//    width: "16",
//    height: "16",
//    fill: "currentColor",
//    class: "bi bi-file-earmark-fill",
//    view_box: "0 0 16 16",
//    path {
//        d: "M4 0h5.293A1 1 0 0 1 10 .293L13.707 4a1 1 0 0 1 .293.707V14a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V2a2 2 0 0 1 2-2m5.5 1.5v2a1 1 0 0 0 1 1h2z"
//    }
//}
//

//svg {
//    width: "16",
//    height: "16",
//    fill: "currentColor",
//    class: "bi bi-file-earmark-check-fill",
//    view_box: "0 0 16 16",
//    path {
//        d: "M9.293 0H4a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h8a2 2 0 0 0 2-2V4.707A1 1 0 0 0 13.707 4L10 .293A1 1 0 0 0 9.293 0M9.5 3.5v-2l3 3h-2a1 1 0 0 1-1-1m1.354 4.354-3 3a.5.5 0 0 1-.708 0l-1.5-1.5a.5.5 0 1 1 .708-.708L7.5 9.793l2.646-2.647a.5.5 0 0 1 .708.708"
//    }
//}


//svg {
//    width: "16",
//    height: "16",
//    fill: "currentColor",
//    class: "bi bi-file-earmark-pdf-fill",
//    view_box: "0 0 16 16",
//    path {
//        d: "M5.523 12.424q.21-.124.459-.238a8 8 0 0 1-.45.606c-.28.337-.498.516-.635.572l-.035.012a.3.3 0 0 1-.026-.044c-.056-.11-.054-.216.04-.36.106-.165.319-.354.647-.548m2.455-1.647q-.178.037-.356.078a21 21 0 0 0 .5-1.05 12 12 0 0 0 .51.858q-.326.048-.654.114m2.525.939a4 4 0 0 1-.435-.41q.344.007.612.054c.317.057.466.147.518.209a.1.1 0 0 1 .026.064.44.44 0 0 1-.06.2.3.3 0 0 1-.094.124.1.1 0 0 1-.069.015c-.09-.003-.258-.066-.498-.256M8.278 6.97c-.04.244-.108.524-.2.829a5 5 0 0 1-.089-.346c-.076-.353-.087-.63-.046-.822.038-.177.11-.248.196-.283a.5.5 0 0 1 .145-.04c.013.03.028.092.032.198q.008.183-.038.465z"
//    },
//    path {
//        fill-rule: "evenodd",
//        d: "M4 0h5.293A1 1 0 0 1 10 .293L13.707 4a1 1 0 0 1 .293.707V14a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V2a2 2 0 0 1 2-2m5.5 1.5v2a1 1 0 0 0 1 1h2zM4.165 13.668c.09.18.23.343.438.419.207.075.412.04.58-.03.318-.13.635-.436.926-.786.333-.401.683-.927 1.021-1.51a11.7 11.7 0 0 1 1.997-.406c.3.383.61.713.91.95.28.22.603.403.934.417a.86.86 0 0 0 .51-.138c.155-.101.27-.247.354-.416.09-.181.145-.37.138-.563a.84.84 0 0 0-.2-.518c-.226-.27-.596-.4-.96-.465a5.8 5.8 0 0 0-1.335-.05 11 11 0 0 1-.98-1.686c.25-.66.437-1.284.52-1.794.036-.218.055-.426.048-.614a1.24 1.24 0 0 0-.127-.538.7.7 0 0 0-.477-.365c-.202-.043-.41 0-.601.077-.377.15-.576.47-.651.823-.073.34-.04.736.046 1.136.088.406.238.848.43 1.295a20 20 0 0 1-1.062 2.227 7.7 7.7 0 0 0-1.482.645c-.37.22-.699.48-.897.787-.21.326-.275.714-.08 1.103"
//    }
//}

//svg {
//    width: "16",
//    height: "16",
//    fill: "currentColor",
//    class: "bi bi-file-earmark-plus-fill",
//    view_box: "0 0 16 16",
//    path {
//        d: "M9.293 0H4a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h8a2 2 0 0 0 2-2V4.707A1 1 0 0 0 13.707 4L10 .293A1 1 0 0 0 9.293 0M9.5 3.5v-2l3 3h-2a1 1 0 0 1-1-1M8.5 7v1.5H10a.5.5 0 0 1 0 1H8.5V11a.5.5 0 0 1-1 0V9.5H6a.5.5 0 0 1 0-1h1.5V7a.5.5 0 0 1 1 0"
//    }
//}

//svg {
//    width: "16",
//    height: "16",
//    fill: "currentColor",
//    class: "bi bi-file-earmark-x-fill",
//    view_box: "0 0 16 16",
//    path {
//        d: "M9.293 0H4a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h8a2 2 0 0 0 2-2V4.707A1 1 0 0 0 13.707 4L10 .293A1 1 0 0 0 9.293 0M9.5 3.5v-2l3 3h-2a1 1 0 0 1-1-1M6.854 7.146 8 8.293l1.146-1.147a.5.5 0 1 1 .708.708L8.707 9l1.147 1.146a.5.5 0 0 1-.708.708L8 9.707l-1.146 1.147a.5.5 0 0 1-.708-.708L7.293 9 6.146 7.854a.5.5 0 1 1 .708-.708"
//    }
//}


//################################################################################
//## svg candidates - DIRECTORIES
//################################################################################


//svg {
//    width: "16",
//    height: "16",
//    fill: "currentColor",
//    class: "bi bi-folder-fill",
//    view_box: "0 0 16 16",
//    path {
//        d: "M9.828 3h3.982a2 2 0 0 1 1.992 2.181l-.637 7A2 2 0 0 1 13.174 14H2.825a2 2 0 0 1-1.991-1.819l-.637-7a2 2 0 0 1 .342-1.31L.5 3a2 2 0 0 1 2-2h3.672a2 2 0 0 1 1.414.586l.828.828A2 2 0 0 0 9.828 3m-8.322.12q.322-.119.684-.12h5.396l-.707-.707A1 1 0 0 0 6.172 2H2.5a1 1 0 0 0-1 .981z"
//    }
//}









































