
use std::path::PathBuf;

use dioxus::prelude::*;
//use reqwest;
//use rfd;

use crate::backend::*;
use crate::backend::persistent_state::exe_dir;


//################################################################################
//## Buttons
//################################################################################

#[component]
pub fn DownloadResultButton(class: String, download_url: String, destination: Option<PathBuf>, comm_with_backend: Signal<FrontendCommChannel>) -> Element {

    rsx! {
        button {
            class: format!("download-result-button primary-button {class}"),
            onclick: move |_| {

                let curdir = match exe_dir() {
                    Some(pth) => pth.clone(),
                    None => PathBuf::new()
                };

                let destination = match destination.clone() {
                    Some(dest) => Some(dest),
                    None => {
                        rfd::FileDialog::new()
                            .set_directory(curdir)
                            .pick_file()
                    }
                };

                if let Some(ref dest) = destination {
                    println!("User chose destination: {}", &dest.to_string_lossy());
                } else {
                    println!("User chose no destination");
                }

                match destination {
                    Some(dest) => {
                        comm_with_backend.read().send(BackendRequest::DownloadContent(download_url.clone(), dest)).ok();
                    },
                    None => {}
                }

            },
            "Download"
        }
    }
}

#[component]
pub fn MoveResultButton(class: String, source: PathBuf, destination: Option<PathBuf>, comm_with_backend: Signal<FrontendCommChannel>) -> Element {

    rsx! {
        button {
            class: format!("download-result-button primary-button {class}"),
            onclick: move |_| {

                let curdir = match exe_dir() {
                    Some(pth) => pth.clone(),
                    None => PathBuf::new()
                };

                let destination = match destination.clone() {
                    Some(dest) => Some(dest),
                    None => {
                        rfd::FileDialog::new()
                            .set_directory(curdir)
                            .pick_file()
                    }
                };

                match destination {
                    Some(dest) => {
                        comm_with_backend.read().send(BackendRequest::MoveContent(source.clone(), dest)).ok();
                    },
                    None => {}
                }

            },
            "Download"
        }
    }
}





//################################################################################
//## Injectable Widgets
//################################################################################

#[component]
pub fn IFrame(title: String, source: String) -> Element {

    // TODO: validation and security checks
    //
    //
    //

    rsx! {
        iframe {
            class: "iframe frame",
            title: title,
            src: source
        }
    }
}













