


// exists for historical reasons; might be used at a later point in time

use std::io::{stdin,stdout,Read,Write};
use std::process::{Command, Child, ChildStdout};
use std::os::windows::process::CommandExt;

const CREATE_NO_WINDOW: u32 = 0x08000000;

fn main() -> Result<(), Box<dyn std::error::Error>> {

    println!("Starting Installation. This wil require multiple reboots.")

    println!("\n\n################################################################################\n## Step 1: Enabling Machine Virtualization\n################################################################################\n\n");

	let mut cmd1 = Command::new("powershell")
                .creation_flags(CREATE_NO_WINDOW) // create no window
                .arg(format!("dism.exe /online /enable-feature /featurename:VirtualMachinePlatform /all /norestart"))
                .env("WSL_UTF8", "1")
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .spawn()
                .unwrap();
    let out1 = cmd1.wait_with_output()?;
    println!("Result1: {}", &out1.status);
    println!("Output: {}", String::from_utf8_lossy(&out1.stdout));
    println!("Errors: {}", String::from_utf8_lossy(&out1.stderr));

    println!("\n\n################################################################################\n## Step 2: Enabling Windows Subsystem for Linux (WSL)\n################################################################################\n\n");

    let mut cmd2 = Command::new("powershell")
                .creation_flags(CREATE_NO_WINDOW) // create no window
                .arg(format!("dism.exe /online /enable-feature /featurename:Microsoft-Windows-Subsystem-Linux /all /norestart"))
                .env("WSL_UTF8", "1")
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .spawn()
                .unwrap();
    let out2 = cmd2.wait_with_output()?;
    println!("Result2: {}", &out2.status);
    println!("Output: {}", String::from_utf8_lossy(&out2.stdout));
    println!("Errors: {}", String::from_utf8_lossy(&out2.stderr));

    println!("\n\n################################################################################\n## Step 3: Installing WSL\n################################################################################\n\n");

    let mut cmd3 = Command::new("powershell")
                .arg(format!("wsl"))
                .env("WSL_UTF8", "1")
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .spawn()
                .unwrap();
    let out3 = cmd3.wait_with_output()?;
    println!("Result3: {}", &out3.status);
    println!("Output: {}", String::from_utf8_lossy(&out3.stdout));
    println!("Errors: {}", String::from_utf8_lossy(&out3.stderr));

    if out3.status != 0 {
        println!("\n\nInstallation of WSL did not succeed. Rebooting the system may resolve the problem and at least one reboot is expected at this point.")
    } else {
        println!("\n\nWSL was installed successfully. Please reboot your system one more time.")
    }

    println!("\n Press Enter to exit the program.")
    let mut s = String::new();
    stdin().read_line(&mut s).expect("Did not enter a correct string");

    return Ok(());
}
