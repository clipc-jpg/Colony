

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod backend;
mod logging;
mod components;
mod pages;

use std::path::PathBuf;

#[allow(unused_imports)]
use dioxus::prelude::*;
#[allow(unused_imports)]
use tao::dpi::{LogicalSize,LogicalPosition};
use tao::window::Icon;
use image;

use backend::*;
//use components::*;
use pages::*;


const ICON: Asset = asset!("/public/images/ColonyIcon.ico");

fn main() {
    //launching window with application GUI
    LaunchBuilder::new()
        .with_cfg(desktop!({
            use dioxus::desktop::{Config, LogicalSize, WindowBuilder};
            Config::new().with_window(
                WindowBuilder::default()
                    .with_title("Colony")
                    .with_inner_size(LogicalSize::new(800.0, 525.0))
                    //.with_min_inner_size(LogicalSize::new(505.0, 300.0))
                    //.with_inner_size(LogicalSize::new(525.0, 800.0))
                    //.with_min_inner_size(LogicalSize::new(300.0, 505.0))
                    //.with_position(LogicalPosition::new(200.0,200.0))
                    //.with_decorations(false)
                    .with_window_icon(load_icon(ICON.resolve()))
                    .with_resizable(true),
            )
        }))
        .launch(CompleteApp);
}

fn load_icon(path: PathBuf) -> Option<Icon> {

    // alternatively, you can embed the icon in the binary through `include_bytes!` macro and use `image::load_from_memory`
    let image = image::open(path).ok().map(|img| img.into_rgba8());
    let icon = image.map(|img| {
        let (width, height) = img.dimensions();
        let rgba = img.into_raw();
        Icon::from_rgba(rgba, width, height).expect("Failed to open icon")
    });
    return icon;
}




