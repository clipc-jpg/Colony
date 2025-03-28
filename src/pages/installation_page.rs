
use std::sync::{Arc,Mutex};

use dioxus::prelude::*;
use itertools::Itertools;

use crate::pages::AppState;
use crate::components::*;
use crate::backend::*;
//use crate::DNA_BACKGROUND1;

#[component]
pub fn InstallPage(app_state: Signal<AppState>, comm_with_backend: Signal<FrontendCommChannel>) -> Element {
    let mut install_state = use_signal(|| InstallationState::InstallationStarted);
    // Signals with Signal dependencies usually need use_memo
    let current_process = use_memo(move || {
        match install_state() {
            InstallationState::InstallationStarted => "Checking installations...".to_string(),
            InstallationState::InstallingWSL => "Checking WSL...".to_string(),
            InstallationState::ImportingDistribution => "Checking Custom WSL Distribution".to_string(),
            InstallationState::DistributionWasNotFound => "Please select a WSL Distribution manually".to_string(),
            InstallationState::DistributionNeedsToBeSelected => "Please select a WSL Distribution manually".to_string(),
            InstallationState::InstallingSingularity => "Installing singularity...".to_string(),
            InstallationState::InstallationEnded => "Completing Installation...".to_string(),
            InstallationState::InstallationFailed => "An Error occurred during installation.".to_string(),
        }
    });
    let mut current_progress = use_signal(|| 0 as i64);

    let mut pseudo_real_time = use_signal(||0 as i64);
    let icon_spin = use_memo(move || {
        let deg = pseudo_real_time * 15;
        format!("rotate({}deg)", deg)
    });

    let mut output_collection_sig = use_signal(|| Arc::new(Mutex::new(Vec::new() as Vec<String>)));
    let output_collection_elems = use_memo(move || to_html_multiline(output_collection_sig.read().lock().expect("Arc/Mutex failed").iter().join("\n")) );

    let backend_count = use_signal_sync(|| 0 as usize);

//    let dt1 = Duration::from_millis(10);
    //    let dt2 = Duration::from_millis(5);

    // progress time for user interactivity
    use_future(move || async move {
        loop {
            let now = tokio::time::Instant::now();
            let dt = std::time::Duration::from_millis(30);
            tokio::time::sleep_until(now + dt).await;
            *pseudo_real_time.write() += 1;
        }
    });

    // have the backend install wsl in the background
    use_future(move || async move {
        let output_collection = Arc::clone(&output_collection_sig());
        //comm_with_backend.read().send(BackendRequest::CheckWSL(output_collection,backend_count)).ok();
        comm_with_backend.read().send(BackendRequest::CheckWSL_v2(output_collection,backend_count)).ok();
        loop {
            let now = tokio::time::Instant::now();
            let dt = std::time::Duration::from_millis(200);
            tokio::time::sleep_until(now + dt).await;

            output_collection_sig.write();

            match comm_with_backend.read().try_receive() {
                Ok(msg) => {
                    match msg {
                        BackendResponse::InstallStepCompleted(InstallationState::InstallationStarted) => {
                            install_state.set(InstallationState::InstallationStarted);
                            current_progress.set(10);
                        },
                        BackendResponse::InstallStepCompleted(InstallationState::InstallingWSL) => {
                            install_state.set(InstallationState::InstallingWSL);
                            current_progress.set(20);
                        },
                        BackendResponse::InstallStepCompleted(InstallationState::ImportingDistribution) => {
                            install_state.set(InstallationState::ImportingDistribution);
                            current_progress.set(40);
                        },
                        BackendResponse::InstallStepCompleted(InstallationState::DistributionWasNotFound) => {
                            install_state.set(InstallationState::DistributionNeedsToBeSelected);
                            current_progress.set(40);
                        },
                        BackendResponse::InstallStepCompleted(InstallationState::DistributionNeedsToBeSelected) => {
                            install_state.set(InstallationState::ImportingDistribution);
                            current_progress.set(40);
                        },
                        BackendResponse::InstallStepCompleted(InstallationState::InstallingSingularity) => {
                            install_state.set(InstallationState::InstallingSingularity);
                            current_progress.set(80);
                        },
                        BackendResponse::InstallStepCompleted(InstallationState::InstallationEnded) => {
                            install_state.set(InstallationState::InstallationEnded);
                            current_progress.set(100);
                            let now = tokio::time::Instant::now();
                            tokio::time::sleep_until(now + dt).await;
                            app_state.set(AppState::EntryPage);
                            break;
                        },
                        BackendResponse::InstallStepCompleted(InstallationState::InstallationFailed) => {
                            install_state.set(InstallationState::InstallationFailed);
                            current_progress.set(100);
                            let now = tokio::time::Instant::now();
                            tokio::time::sleep_until(now + dt*15000).await;
                            app_state.set(AppState::EntryPage);
                            break;
                        }
                        _ => {panic!("Need to echo unused messages back")}
                    }
                },
                Err(_) => {}
            }
        }
    });

    rsx! {
        div {
            class: "installation-background background",
            // background_image: BACKGROUND_IMG,
            h1 {
                class: "installation-header",
                {current_process()}
            }
            div {
                class: "installation-spinning-icon rotating large-icon icon-container",
                transform: icon_spin,
                LoadingIcon {class: "installation-spinning-icon large-icon unclickable".to_owned()}
            }

            progress {
                id:    "installation-progress",
                value: current_progress(),
                max:   "100",
                {format!("{}%", current_progress())}
            }
            div {
                class: "installation-page backend-output-container",
                p {
                    class: "installation-page backend-output",
                    display: "block",

                    for elem in output_collection_elems() {
                        {elem}
                    }
                }
            }
        }


    }
}

















