


// emit linter errors on unfinished code and on possibly unsound code
#![cfg_attr(feature = "dev", warn(clippy::todo, clippy::unimplemented, clippy::unreachable))]
// enforce compile time errors on unfinished code in non-development builds
#![cfg_attr(not(feature = "dev"), deny(clippy::todo, clippy::unimplemented))]


mod server;
mod endpoints;





//#[tokio::main]
fn main() {

    #[cfg(not(target_os = "linux"))]
    compile_error!("This crate can only be built on Linux!");

    let message_db = server::HardTypedDBAccess::new();
}



