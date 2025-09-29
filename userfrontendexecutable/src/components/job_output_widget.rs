

use std::path::PathBuf;

use dioxus::prelude::*;
use crate::backend::{JobId, FrontendCommChannel, vec_to_html_multiline};


#[component]
pub fn JobOutputWidget(class: String, job_id: JobId, working_directory: Memo<Option<PathBuf>>, displayed_text: Signal<Vec<String>>,comm_with_backend: Signal<FrontendCommChannel>) -> Element {

    // displayed text needs to be injected since the viewed job may change,
    // or a job may stop to exist
    // and then this a render pass might interact with a signal that has been dropped => causes crashes
    // "Do not use 'use_signal' and the like conditionally"
    let backend_output = displayed_text;

    let log_directory_message = use_memo(move || {

        if working_directory().is_some_and(|dir| dir.is_dir()) {
            let dir = working_directory().unwrap();
            format!("Logs will be written to {dir:?}")
        } else {
            format!("Warning: no valid direcotry to write logs to. Please adjust your selected working directory.")
        }
    });

    let _update_lines = use_future(move || async move {

        // initially, send all lines
        comm_with_backend.read().send(crate::BackendRequest::SendJobOutput(job_id, 0)).ok();

        loop {
            let now = tokio::time::Instant::now();
            let dt = std::time::Duration::from_millis(200);
            tokio::time::sleep_until(now + dt).await;

            //println!("Number of Lines of displayed output: {}", backend_output().len());

            //match comm_with_backend.read().try_receive() {
//                Ok(msg) => {
//                    match msg {
//                        BackendResponse::JobOutput(backend_job_id, output) if backend_job_id == job_id => {
//                            backend_output.with_mut(move |lines| {
//                                for new_line in output.into_iter() {
//                                    lines.push(new_line);
//                                }
//                            });
//                            comm_with_backend.read().send(crate::BackendRequest::SendJobOutput(job_id, backend_output().len())).ok();
//                        },
//                        other_msg => {
//                            // here single consumer abstraction fails
//                            println!("Received other message! {:?}", &other_msg);
//                            comm_with_backend.read().reinsert_message(other_msg).ok();
//                        }
//                    }
//                },
//                _ => {}
//            }
        }
    });

    rsx! {
        div {
            class: format!("{class} job-output-widget job-output-container"),
            p {
                class: format!("{class} job-output-widget job-output"),
                display: "block",
                "{log_directory_message:?}" br {}
                br{}
                for elem in vec_to_html_multiline(backend_output()) {
                    {elem}
                }
            }
        }
    }
}












