
#![allow(unused)]



use std::error::Error;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::io::Read;
use std::process::{Child, Command, ChildStdout};
use std::io::BufRead;
use std::io::BufReader;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;

use regex::{Regex, Captures};
use fs_extra::dir::CopyOptions;
use itertools::Itertools;

use crate::persistent_state::exe_dir;

extern crate fs_extra;



//################################################################################
//## Constants
//################################################################################

pub const CREATE_NO_WINDOW: u32 = 0x08000000;



//################################################################################
//## Utilities
//################################################################################

pub fn wslify_windows_path(windows_path: &str) -> String {
    let windows_path_regex = Regex::new(r#"(?<hd>\w):\\(.*)"#).unwrap();
    let result = windows_path_regex.replace(windows_path, |cpt: &Captures| {
        let drive = cpt[1].to_ascii_lowercase();
        let rest = cpt[2].replace("\\", "/");
        format!("/mnt/{}/{}", &drive, &rest)
    });
    return result.into_owned();
}

pub fn unwslify_wsl_linux_path(wsl_linux_path: &str) -> String {
    let wsl_linux_path_regex = Regex::new(r#"/mnt/(?<hd>\w)/(.*)"#).unwrap();
    let result = wsl_linux_path_regex.replace(wsl_linux_path, |cpt: &Captures| {
        let drive = cpt[1].to_ascii_uppercase();
        let rest = cpt[2].replace("/", "\\");
        format!("{}:\\{}", &drive, &rest)
    });
    return result.into_owned();
}

//################################################################################
//## File management
//################################################################################

#[cfg(not(target_arch = "wasm32"))]
pub fn choose_sif_file(starting_dir: &Option<PathBuf>) -> Option<PathBuf> {
    println!("Choosing sif file");
    let path = match starting_dir {
        Some(ptb) => ptb.clone(),
        None => std::env::current_dir().unwrap()
    };

    println!("Opening file dialog at {:?}",&path);
    let res = rfd::FileDialog::new()
                    .set_directory(&path)
                    .add_filter("SIF files", &["sif"])
                    .add_filter("All files", &["*"])
                    .pick_file();

    println!("The user chose: {:#?}", res);

    return res;
}

#[cfg(not(target_arch = "wasm32"))]
pub fn choose_file(starting_dir: &Option<PathBuf>) -> Option<PathBuf> {
    let path = match starting_dir {
        Some(ptb) => ptb.clone(),
        None => std::env::current_dir().unwrap()
    };

    let res = rfd::FileDialog::new()
                        .set_directory(&path)
                        .add_filter("All files", &["*"])
                        .pick_file();

    println!("The user chose: {:#?}", res);

    return res;
}

#[cfg(not(target_arch = "wasm32"))]
pub fn choose_files(starting_dir: &Option<PathBuf>) -> Option<Vec<PathBuf>> {
    let path = match starting_dir {
        Some(ptb) => ptb.clone(),
        None => std::env::current_dir().unwrap()
    };

    let res = rfd::FileDialog::new()
                        .set_directory(&path)
                        .add_filter("All files", &["*"])
                        .pick_files();

    println!("The user choose: {:#?}", res);

    return res;
}

#[cfg(not(target_arch = "wasm32"))]
pub fn choose_directory(starting_dir: &Option<PathBuf>) -> Option<PathBuf> {
    let path = match starting_dir {
        Some(ptb) => ptb.clone(),
        None => std::env::current_dir().unwrap()
    };

    let res = rfd::FileDialog::new()
    .set_directory(&path)
    .pick_folder();

    println!("The user choose: {:#?}", res);

    return res;
}

#[cfg(not(target_arch = "wasm32"))]
pub fn choose_directories(starting_dir: &Option<PathBuf>) -> Option<Vec<PathBuf>> {
    let path = match starting_dir {
        Some(ptb) => ptb.clone(),
        None => std::env::current_dir().unwrap()
    };

    let res = rfd::FileDialog::new()
                        .set_directory(&path)
                        .pick_folders();

    println!("The user choose: {:#?}", res);

    return res;
}


//################################################################################
//## General WSL command
//################################################################################

pub fn run_wsl_command(command: &str) -> Result<Child,std::io::Error> {

    println!("Command: /C wsl -d ColonyWSL -e {}", &command);
    //return Command::new("cmd")
    //                .args(["/C", "wsl", "-e", &command])
    //                .args(command_args)
    //                .spawn();
    return Command::new("cmd")
    .creation_flags(CREATE_NO_WINDOW) // create no window
    .arg(format!("/C wsl -d ColonyWSL -e {}", &command))
    .env("WSL_UTF8", "1")
    .stdin(std::process::Stdio::piped())
    .stdout(std::process::Stdio::piped())
    .spawn();
}
