
use dioxus::prelude::*;




#[component]
pub fn LoadingIcon(class: String) -> Element {
    rsx! {
        svg {
            width: "16",
            height: "16",
            fill: "currentColor",
            class: format!("bi bi-arrow-clockwise {class}"),
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
}




#[component]
pub fn StoppedIcon(class: String) -> Element {
    rsx! {
        svg {
            width: "16",
            height: "16",
            fill: "currentColor",
            class:  format!("bi bi-square-fill {class}"),
            view_box: "0 0 16 16",
            path {
                d: "M0 2a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H2a2 2 0 0 1-2-2z"
            }
        }
    }
}













