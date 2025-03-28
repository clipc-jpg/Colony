




use dioxus::prelude::*;

use crate::pages::AppState;
use crate::components::*;
use crate::backend::*;

use self::JobId;

#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub enum GenericJobState {
    JobQueued,
    JobStarted,
    JobCompleted,
    JobErrored
}

#[component]
pub fn GenericObserverPage(app_state: Signal<AppState>, comm_with_backend: Signal<FrontendCommChannel>, jobid: JobId) -> Element {
    // Signals with Signal dependencies usually need use_memo

    let mut backend_output = use_signal_sync(|| Vec::new() as Vec<String>);

    // have the backend install wsl in the background
    use_future(move || { let jobid = jobid.clone(); async move {
        loop {
            let now = tokio::time::Instant::now();
            let dt = std::time::Duration::from_millis(200);
            tokio::time::sleep_until(now + dt).await;

            match comm_with_backend.read().send(BackendRequest::SendJobOutput(jobid.clone(), backend_output().len())) {
                Ok(_response) => {},
                Err(_) => {continue;}
            }

            loop {
                let now = tokio::time::Instant::now();
                let dt = std::time::Duration::from_millis(200);
                tokio::time::sleep_until(now + dt).await;

                match comm_with_backend.read().try_receive() {
                    Ok(msg) => {
                        match msg {
                            BackendResponse::JobOutput(received_job_id, mut lines) if received_job_id == jobid => {
                                backend_output.write().append(&mut lines);
                            },
                            _ => {panic!("Need to echo unused messages back")}
                        }
                    },
                    Err(_) => {}
                }
            }
        }
    }});


    rsx! {
        div {
            class: "installation-background background",
            // background_image: BACKGROUND_IMG,
            h1 {
                class: "installation-header",
                //{current_process()}
            }
            div {
                class: "installation-spinning-icon rotating large-icon icon-container",
                //transform: icon_spin,
                LoadingIcon {class: "installation-spinning-icon large-icon unclickable".to_owned()}
            }
            div {
                class: "installation-page backend-output-container",
                    p {
                        class: "installation-page backend-output",
                        display: "block",
                        for elem in vec_to_html_multiline(backend_output()) {
                            {elem}
                        }
                    }

            }
        }
    }
}

















