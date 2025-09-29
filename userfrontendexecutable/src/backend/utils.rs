

use std::string::String;
use std::path::PathBuf;
use std::hash::Hash;
use std::collections::HashSet;

use dioxus::prelude::*;
use itertools::Itertools;





#[allow(unstable_name_collisions)]
pub fn to_html_multiline(string: String) -> Vec<Element> {
    return string
        .split("\n")
        .map(|s| rsx! { {s} })
        .intersperse(rsx! {br {display: "block"}})
        .collect();
}


#[allow(unstable_name_collisions)]
pub fn vec_to_html_multiline(strings: Vec<String>) -> Vec<Element> {
    return strings.into_iter()
        .map(|s| rsx! { {s} })
        .intersperse(rsx! {br {display: "block"}})
        .collect();
}



pub fn linux_path_display(path: &PathBuf) -> String {
    let mut path_string = if path.is_absolute() { "/".to_owned() } else { "".to_owned() };
    path_string.push_str(
        &path.components().map(|path_comp| path_comp.as_os_str().to_string_lossy().replace("\\", "/"))
             .join("/")
    );

    return path_string;
}


#[allow(unused)]
fn dedup_inplace<T>(v: &mut Vec<T>) where T: Hash + std::cmp::Eq + Copy {
    let mut set = HashSet::new();

    v.retain(|x| set.insert(*x));
}




