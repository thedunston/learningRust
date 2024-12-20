use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use reqwest;
use std::borrow::Cow;
use std::process::{Command, Output};
use port_check::*;
use std::time::Duration;
use sysinfo::{System};

/* Function to execute a command. */
fn executecommand(cmd: &String) -> String{

    // Declare variables for shell and argument.
    let (the_shell, the_arg);

    // Check the operating system and assign values.
    if std::env::consts::OS == "windows" {
    
        the_shell = "cmd";
        the_arg = "/c";
    
    } else {
    
        the_shell = "bash";
        the_arg = "-c";
    
    }

    // Run the command.
    let res: Output = Command::new(the_shell)
        .arg(the_arg)
        .arg(cmd)
        .output()
        .unwrap();

    // Get the output and error and convert to string.
    let stdout: Cow<str> = String::from_utf8_lossy(&res.stdout);
    let stderr: Cow<str> = String::from_utf8_lossy(&res.stderr);

    // Check which output to return.
    // Debugging, remove later.
    if stderr.is_empty() {
    
        stdout.to_string()
    
    } else {

        stderr.to_string()
    
    }

}

/* Function to check if a file exists. */
fn file_exists(path: &str) -> bool {

    // Check if the file exists.
    std::path::Path::new(path).exists()

}

/* Function to check if a process is running. */
fn check_process(pname: &str) -> bool {

    // Check if the process is running.

    //Debug.
    println!("Checking if {} is running.", pname);
    
    let mut system = System::new_all();
    system.refresh_all();

    // Check if the process is running.
    let is_running = system.processes().values().any(|process| {

        process.name().eq_ignore_ascii_case(pname)
    
    });

    is_running

}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {

    // Run in a loop a check to see if a port is open.
    println!("Checking if port is open.");
    let is_reachable = is_port_reachable("127.0.0.1:61180");
    
    loop {
 
        if is_reachable && check_process("rox") && file_exists("/home/user/rox") {
        
            println!("file exists and the port is open and the process is running.");
            tokio::time::sleep(Duration::from_secs(5)).await;

            continue
        
        } else {
        
            // If the file exists and is not running then start it.
            if file_exists("/home/user/rox") && !check_process("rox") {
            
                println!("Starting rox.");

                let cmd = "/home/user/rox".to_string();
                let output = executecommand(&cmd);

                println!("{}", output);
            
            } else if !file_exists("/home/user/rox") {
            
                println!("Downloading rox.");

                 // Check if a port on a remote host is open.
                let url = "http://127.0.0.1:61180/rox";
                let response = reqwest::get(url).await?;
        
                // Create the file.
                let mut file = File::create("/home/user/rox").await.unwrap();

                // Save rox to the file.
                file.write_all(response.bytes().await.unwrap().as_ref()).await.unwrap();
        
                // Make the file executable.
                // Testing, need to use native function.
                let cmd = "chmod +x /home/user/rox".to_string();
                let _output = executecommand(&cmd);
                
                // sleep 2 seconds.
                tokio::time::sleep(Duration::from_secs(2)).await;
        
                let cmd = "/home/user/rox".to_string();
                let _output = executecommand(&cmd);

            }
            
        }
        
        // Sleep for 5 seconds.
        tokio::time::sleep(Duration::from_secs(5)).await;
 
    }

}