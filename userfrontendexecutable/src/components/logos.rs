

use dioxus::prelude::*;


//const IMI_LOGO: Asset = asset!("/public/images/211123_Logo_IMI-rgb_cropped.jpg");
const IMI_LOGO: &str = "./assets/211123_Logo_IMI-rgb_cropped.jpg";

#[component]
pub fn IMILogo(class: String) -> Element {
    rsx! {
        img {
            class: format!("IMI logo {class}"),
            src: IMI_LOGO
        }
    }
}



//#[component]
//pub fn IMILogo_origin(class: String) -> Element {
//    rsx! {
//        img {
//            class: format!("IMI logo {class}"),
//            src: "./211123_Logo_IMI-rgb_cropped.jpg"
//        }
//    }
//}

















