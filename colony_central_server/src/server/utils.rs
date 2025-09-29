

use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::OnceLock;

use serde::{Serialize, Deserialize};


#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct UnixTime(u64);



impl UnixTime {
    pub fn now() -> Self {
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(n) => Self(n.as_secs()),
            Err(_) => Self(0),
        }
    }

    pub fn as_secs(&self) -> u64 {
        return self.0;
    }

}


static EXE_DIR: OnceLock<Option<PathBuf>> = OnceLock::new();

pub fn exe_dir() -> &'static Option<PathBuf>{
    return EXE_DIR.get_or_init(|| {
        match std::env::current_exe() {
            Ok(ptb) => {
                match ptb.parent() {
                    Some(pth) => Some(PathBuf::from(pth)),
                               None => Some(ptb)
                }
            }
            Err(_e) => {
                match std::env::var("COLONY_CONFIGDIR") {
                    Ok(val) => {
                        let pth = PathBuf::from(val);
                        if pth.is_dir() {Some(pth)} else {
                            println!(r#"Environment variable "COLONY_CONFIGDIR" is not a valid directory. Cannot read/write persistent session data."#);
                            None
                        }
                    },
                    Err(_e) => {
                        println!(r#"Environment variable "COLONY_CONFIGDIR" has not been set. Cannot read/write persistent session data."#);
                        None
                    },
                }
            }
        }
    })
}





