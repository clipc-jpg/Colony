

use std::path::PathBuf;

use dioxus::prelude::*;

use crate::{backend::*, AppState};


#[component]
pub fn HomeButton(class: String, app_state: Signal<AppState>) -> Element {
    rsx! {
        div {
            class: format!("back-to-entry-page-button medium-icon icon-container {class}"),
            onclick: move |_| {
                app_state.set(AppState::EntryPage);
            },
            svg {
                width: "16",
                height: "16",
                fill: "currentColor",
                class: format!("back-to-entry-page-button medium-icon bi bi-house-fill {class}"),
                view_box: "0 0 16 16",
                path {
                    d: "M8.707 1.5a1 1 0 0 0-1.414 0L.646 8.146a.5.5 0 0 0 .708.708L8 2.207l6.646 6.647a.5.5 0 0 0 .708-.708L13 5.793V2.5a.5.5 0 0 0-.5-.5h-1a.5.5 0 0 0-.5.5v1.293z"
                }
                path {
                    d: "m8 3.293 6 6V13.5a1.5 1.5 0 0 1-1.5 1.5h-9A1.5 1.5 0 0 1 2 13.5V9.293z"
                }
            }
        }
    }
}


#[component]
pub fn LinkButton(class: String, url: String, app_state: Signal<AppState>) -> Element {
    rsx! {
        div {
            class: format!("to-link-button medium-icon icon-container {class}"),
            onclick: move |_| {
                app_state.set(AppState::IFrame(url.clone()));
            },
            svg {
                width: "16",
                height: "16",
                fill: "currentColor",
                class: format!("to-link-button medium-icon bi bi-wrench-adjustable {class}"),
                view_box: "0 0 16 16",
                path {
                    d: "M16 4.5a4.5 4.5 0 0 1-1.703 3.526L13 5l2.959-1.11q.04.3.041.61"
                }
                path {
                    d: "M11.5 9c.653 0 1.273-.139 1.833-.39L12 5.5 11 3l3.826-1.53A4.5 4.5 0 0 0 7.29 6.092l-6.116 5.096a2.583 2.583 0 1 0 3.638 3.638L9.908 8.71A4.5 4.5 0 0 0 11.5 9m-1.292-4.361-.596.893.809-.27a.25.25 0 0 1 .287.377l-.596.893.809-.27.158.475-1.5.5a.25.25 0 0 1-.287-.376l.596-.893-.809.27a.25.25 0 0 1-.287-.377l.596-.893-.809.27-.158-.475 1.5-.5a.25.25 0 0 1 .287.376M3 14a1 1 0 1 1 0-2 1 1 0 0 1 0 2"
                }
            }
        }
    }
}

#[component]
pub fn SelfConfigButton(class: String, container: PathBuf, url: String, app_state: Signal<AppState>) -> Element {
    rsx! {
        div {
            class: format!("to-link-button medium-icon icon-container {class}"),
            onclick: move |_| {
                app_state.set(AppState::ContainerSelfConfiguratorPage(container.clone(), url.clone()));
            },
            svg {
                width: "16",
                height: "16",
                fill: "currentColor",
                class: format!("to-link-button medium-icon bi bi-wrench-adjustable {class}"),
                view_box: "0 0 16 16",
                path {
                    d: "M16 4.5a4.5 4.5 0 0 1-1.703 3.526L13 5l2.959-1.11q.04.3.041.61"
                }
                path {
                    d: "M11.5 9c.653 0 1.273-.139 1.833-.39L12 5.5 11 3l3.826-1.53A4.5 4.5 0 0 0 7.29 6.092l-6.116 5.096a2.583 2.583 0 1 0 3.638 3.638L9.908 8.71A4.5 4.5 0 0 0 11.5 9m-1.292-4.361-.596.893.809-.27a.25.25 0 0 1 .287.377l-.596.893.809-.27.158.475-1.5.5a.25.25 0 0 1-.287-.376l.596-.893-.809.27a.25.25 0 0 1-.287-.377l.596-.893-.809.27-.158-.475 1.5-.5a.25.25 0 0 1 .287.376M3 14a1 1 0 1 1 0-2 1 1 0 0 1 0 2"
                }
            }
        }
    }
}

#[component]
pub fn HtmlWrapperButton(class: String, url: String, app_state: Signal<AppState>) -> Element {
    rsx! {
        div {
            class: format!("to-html-wrapper-button medium-icon icon-container {class}"),
            onclick: move |_| {
                //app_state.set(AppState::HTMLPage(url.clone()));
            },
            svg {
                width: "16",
                height: "16",
                fill: "currentColor",
                class: format!("to-html-wrapper-button medium-icon bi bi-brush-fill {class}"),
                view_box: "0 0 16 16",
                path {
                    d: "M15.825.12a.5.5 0 0 1 .132.584c-1.53 3.43-4.743 8.17-7.095 10.64a6.1 6.1 0 0 1-2.373 1.534c-.018.227-.06.538-.16.868-.201.659-.667 1.479-1.708 1.74a8.1 8.1 0 0 1-3.078.132 4 4 0 0 1-.562-.135 1.4 1.4 0 0 1-.466-.247.7.7 0 0 1-.204-.288.62.62 0 0 1 .004-.443c.095-.245.316-.38.461-.452.394-.197.625-.453.867-.826.095-.144.184-.297.287-.472l.117-.198c.151-.255.326-.54.546-.848.528-.739 1.201-.925 1.746-.896q.19.012.348.048c.062-.172.142-.38.238-.608.261-.619.658-1.419 1.187-2.069 2.176-2.67 6.18-6.206 9.117-8.104a.5.5 0 0 1 .596.04"
                }
            }
        }
    }
}


#[component]
pub fn SelfConfigDummyButton(class: String, app_state: Signal<AppState>) -> Element {
    rsx! {
        div {
            class: format!("self-config-dummy-button medium-icon icon-container {class}"),
            onclick: move |_| {
                app_state.set(AppState::TestPage);
            },
            svg {
                width: "16",
                height: "16",
                fill: "currentColor",
                class: "self-config-dummy-button medium-icon bi bi-card-checklist",
                view_box: "0 0 16 16",
                path {
                    d: "M14.5 3a.5.5 0 0 1 .5.5v9a.5.5 0 0 1-.5.5h-13a.5.5 0 0 1-.5-.5v-9a.5.5 0 0 1 .5-.5zm-13-1A1.5 1.5 0 0 0 0 3.5v9A1.5 1.5 0 0 0 1.5 14h13a1.5 1.5 0 0 0 1.5-1.5v-9A1.5 1.5 0 0 0 14.5 2z"
                }
                path {
                    d: "M7 5.5a.5.5 0 0 1 .5-.5h5a.5.5 0 0 1 0 1h-5a.5.5 0 0 1-.5-.5m-1.496-.854a.5.5 0 0 1 0 .708l-1.5 1.5a.5.5 0 0 1-.708 0l-.5-.5a.5.5 0 1 1 .708-.708l.146.147 1.146-1.147a.5.5 0 0 1 .708 0M7 9.5a.5.5 0 0 1 .5-.5h5a.5.5 0 0 1 0 1h-5a.5.5 0 0 1-.5-.5m-1.496-.854a.5.5 0 0 1 0 .708l-1.5 1.5a.5.5 0 0 1-.708 0l-.5-.5a.5.5 0 0 1 .708-.708l.146.147 1.146-1.147a.5.5 0 0 1 .708 0"
                }
            }
        }
    }
}

#[component]
pub fn TestButton(class: String, app_state: Signal<AppState>) -> Element {
    rsx! {
        div {
            class: format!("self-config-dummy-button medium-icon icon-container {class}"),
            onclick: move |_| {
                app_state.set(AppState::TestPage);
            },
            svg {
                width: "16",
                height: "16",
                fill: "currentColor",
                class: "self-config-dummy-button medium-icon bi bi-bug-fill",
                view_box: "0 0 16 16",
                path {
                    d: "M4.978.855a.5.5 0 1 0-.956.29l.41 1.352A5 5 0 0 0 3 6h10a5 5 0 0 0-1.432-3.503l.41-1.352a.5.5 0 1 0-.956-.29l-.291.956A5 5 0 0 0 8 1a5 5 0 0 0-2.731.811l-.29-.956z"
                }
                path {
                    d: "M13 6v1H8.5v8.975A5 5 0 0 0 13 11h.5a.5.5 0 0 1 .5.5v.5a.5.5 0 1 0 1 0v-.5a1.5 1.5 0 0 0-1.5-1.5H13V9h1.5a.5.5 0 0 0 0-1H13V7h.5A1.5 1.5 0 0 0 15 5.5V5a.5.5 0 0 0-1 0v.5a.5.5 0 0 1-.5.5zm-5.5 9.975V7H3V6h-.5a.5.5 0 0 1-.5-.5V5a.5.5 0 0 0-1 0v.5A1.5 1.5 0 0 0 2.5 7H3v1H1.5a.5.5 0 0 0 0 1H3v1h-.5A1.5 1.5 0 0 0 1 11.5v.5a.5.5 0 1 0 1 0v-.5a.5.5 0 0 1 .5-.5H3a5 5 0 0 0 4.5 4.975"
                }
            }
        }
    }
}

#[component]
pub fn ProjectOverviewPageButton(class: String, app_state: Signal<AppState>) -> Element {
    rsx! {
        div {
            class: format!("project-page-dummy-button medium-icon icon-container {class}"),
            onclick: move |_| {
                app_state.set(AppState::ProjectOverviewPage);
            },
            svg {
                width: "16",
                height: "16",
                fill: "currentColor",
                class: "project-page-dummy-button medium-icon bi bi-calendar-plus-fill",
                view_box: "0 0 16 16",
                path {
                    d: "M4 .5a.5.5 0 0 0-1 0V1H2a2 2 0 0 0-2 2v1h16V3a2 2 0 0 0-2-2h-1V.5a.5.5 0 0 0-1 0V1H4zM16 14V5H0v9a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2M8.5 8.5V10H10a.5.5 0 0 1 0 1H8.5v1.5a.5.5 0 0 1-1 0V11H6a.5.5 0 0 1 0-1h1.5V8.5a.5.5 0 0 1 1 0"
                }
            }
        }
    }
}

#[component]
pub fn ProjectPageButton(class: String, app_state: Signal<AppState>) -> Element {
    rsx! {
        div {
            class: format!("project-page-dummy-button medium-icon icon-container {class}"),
            onclick: move |_| {
                app_state.set(AppState::LabBookPage);
            },
            svg {
                width: "16",
                height: "16",
                fill: "currentColor",
                class: "project-page-dummy-button medium-icon bi bi-calendar-plus-fill",
                view_box: "0 0 16 16",
                path {
                    d: "M4 .5a.5.5 0 0 0-1 0V1H2a2 2 0 0 0-2 2v1h16V3a2 2 0 0 0-2-2h-1V.5a.5.5 0 0 0-1 0V1H4zM16 14V5H0v9a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2M8.5 8.5V10H10a.5.5 0 0 1 0 1H8.5v1.5a.5.5 0 0 1-1 0V11H6a.5.5 0 0 1 0-1h1.5V8.5a.5.5 0 0 1 1 0"
                }
            }
        }
    }
}


#[component]
pub fn StartLocalServerButton(class: String, followup_page: Signal<AppState>, comm_partner_container: Signal<PathBuf>, comm_with_backend: Signal<FrontendCommChannel>) -> Element {
    rsx! {
        div {
            class: format!("self-config-dummy-button medium-icon icon-container {class}"),
            onclick: move |_| {
                comm_with_backend.read().send(BackendRequest::StartLocalWebServer(followup_page.read().clone(), comm_partner_container.read().clone())).ok();
            },
            svg {
                width: "16",
                height: "16",
                fill: "currentColor",
                class: "self-config-dummy-button medium-icon bi bi-airplane-fill",
                view_box: "0 0 16 16",
                path {
                    d: "M6.428 1.151C6.708.591 7.213 0 8 0s1.292.592 1.572 1.151C9.861 1.73 10 2.431 10 3v3.691l5.17 2.585a1.5 1.5 0 0 1 .83 1.342V12a.5.5 0 0 1-.582.493l-5.507-.918-.375 2.253 1.318 1.318A.5.5 0 0 1 10.5 16h-5a.5.5 0 0 1-.354-.854l1.319-1.318-.376-2.253-5.507.918A.5.5 0 0 1 0 12v-1.382a1.5 1.5 0 0 1 .83-1.342L6 6.691V3c0-.568.14-1.271.428-1.849"
                }
            }
        }
    }
}



#[component]
pub fn RequestFilePathButton(class: String, data: Signal<PathBuf>, starting_dir: Signal<Option<PathBuf>>) -> Element {
    rsx! {
        div {
            class: format!("request-file-button container {class}"),
            onclick: move |_| async move {
                    let path = match starting_dir() {
                        Some(ptb) => ptb.clone(),
                        None => std::env::current_dir().unwrap()
                    };

                    //TODO using map here is not idiomatic and may be compiled away in release mode
                    rfd::FileDialog::new()
                        .set_directory(&path)
                        .add_filter("All files", &["*"])
                        .pick_file()
                        .map(|res| data.set(res));
            },
            svg {
                width: "16",
                height: "16",
                fill: "currentColor",
                class: "request-file-button svg bi bi-file-fill",
                view_box: "0 0 16 16",
                path {
                    fill_rule: "evenodd",
                    d: "M4 0h8a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V2a2 2 0 0 1 2-2"
                }
            }
        }
    }
}

#[component]
pub fn RequestDirectoryPathButton(class: String, data: Signal<PathBuf>, starting_dir: Signal<Option<PathBuf>>) -> Element {
    rsx! {
        div {
            class: format!("request-file-button container {class}"),
            onclick: move |_| async move {
                    let path = match starting_dir() {
                        Some(ptb) => ptb.clone(),
                        None => std::env::current_dir().unwrap()
                    };

                    //TODO using map here is not idiomatic and may be compiled away in release mode
                    rfd::FileDialog::new()
                        .set_directory(&path)
                        .pick_folder()
                        .map(|res| data.set(res));
            },
            svg {
                width: "16",
                height: "16",
                fill: "currentColor",
                class: "request-file-button svg bi bi-file-fill",
                view_box: "0 0 16 16",
                path {
                    fill_rule: "evenodd",
                    d: "M4 0h8a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V2a2 2 0 0 1 2-2"
                }
            }
        }
    }
}

#[component]
pub fn RequestFilePathButton2(class: String, starting_dir: Signal<Option<PathBuf>>, result_handler: EventHandler<PathBuf>) -> Element {
    rsx! {
        div {
            class: format!("request-file-button container {class}"),
            onclick: move |_| async move {
                    let path = match starting_dir() {
                        Some(ptb) => ptb.clone(),
                        None => std::env::current_dir().unwrap()
                    };

                    //TODO using map here is not idiomatic and may be compiled away in release mode
                    rfd::FileDialog::new()
                        .set_directory(&path)
                        .add_filter("All files", &["*"])
                        .pick_file()
                        .map(|res| result_handler.call(res));
            },
            svg {
                width: "16",
                height: "16",
                fill: "currentColor",
                class: "request-file-button svg bi bi-file-fill",
                view_box: "0 0 16 16",
                path {
                    fill_rule: "evenodd",
                    d: "M4 0h8a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V2a2 2 0 0 1 2-2"
                }
            }
        }
    }
}

#[component]
pub fn RequestDirectoryPathButton2(class: String, starting_dir: Signal<Option<PathBuf>>, result_handler: EventHandler<PathBuf>) -> Element {
    rsx! {
        div {
            class: format!("request-file-button container {class}"),
            onclick: move |_| async move {
                    let path = match starting_dir() {
                        Some(ptb) => ptb.clone(),
                        None => std::env::current_dir().unwrap()
                    };

                    //TODO using map here is not idiomatic and may be compiled away in release mode
                    rfd::FileDialog::new()
                        .set_directory(&path)
                        .pick_folder()
                        .map(|res| result_handler.call(res));
            },
            svg {
                width: "16",
                height: "16",
                fill: "currentColor",
                class: "request-file-button svg bi bi-file-fill",
                view_box: "0 0 16 16",
                path {
                    fill_rule: "evenodd",
                    d: "M4 0h8a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V2a2 2 0 0 1 2-2"
                }
            }
        }
    }
}





