

use std::path::PathBuf;

use dioxus::prelude::*;
use itertools::Itertools;

pub const BACKGROUND_IMG: Asset = asset!("/public/images/dna_background_v2.jpg");


#[component]
pub fn MultiLinePathString(path: PathBuf) -> Element {

    let path_separator = if std::env::consts::OS == "windows" {"\\"} else {"/"};

    let first_path_component =
        match path.components().next() {
            Some(c) => {
                match c.as_os_str().to_str() {
                    Some(s) => { format!("{}{}", s, if std::env::consts::OS == "windows" {"\\\\"} else {"/"}) },
                    None => {String::from("UNREADABLE")}
                }
            }
            None => {String::from("EMPTY PATH")}
    };

    rsx! {
        {first_path_component}
        {
            #[allow(unstable_name_collisions)]
            path.components()
                .skip(1)
                .map(|c| {
                    match c.as_os_str().to_str() {
                        Some(s) => {s},
                        None => {"UNREADABLE"}
                    }
                })
                .filter(|s| !s.is_empty() && s != &path_separator)
                .map(|s|  rsx! { {s} } )
                .intersperse(rsx! { wbr {} {path_separator} })
        }
    }
}

























