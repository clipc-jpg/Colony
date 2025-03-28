

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod backend;
mod components;
mod pages;

#[allow(unused_imports)]
use tao::dpi::{LogicalSize,LogicalPosition};
#[allow(unused_imports)]
use tao::window::WindowBuilder;
use dioxus::prelude::*;

use backend::*;
//use components::*;
use pages::*;


fn main() {
    //launching window with application GUI
    LaunchBuilder::new()
        .with_cfg(desktop!({
            use dioxus::desktop::{Config, LogicalSize, WindowBuilder};
            Config::new().with_window(
                WindowBuilder::default()
                    .with_title("Colony")
                    .with_inner_size(LogicalSize::new(800.0, 525.0))
                    .with_min_inner_size(LogicalSize::new(505.0, 300.0))
                    //.with_inner_size(LogicalSize::new(525.0, 800.0))
                    //.with_min_inner_size(LogicalSize::new(300.0, 505.0))
                    //.with_position(LogicalPosition::new(200.0,200.0))
                    //.with_decorations(false)
                    .with_resizable(true),
            )
        }))
        .launch(CompleteApp);
}






