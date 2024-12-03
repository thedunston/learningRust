// Author: Duane Dunston <thedunston@gmail.com>
// Date: 2024-12-02
/*

Features added:
- check the OS and run the respective command shell.

*/
// Executing commands.
use std::process::{Command, Output};
use std::env;
use std::borrow::Cow;

fn executecmd(cmd:&str) -> String {

    let (the_shell, the_arg);

    // Check if the OS is Windows.
    if std::env::consts::OS == "windows" {

        the_shell = "cmd";
        the_arg = "/c";

    } else {

        the_shell = "bash";
        the_arg = "-c";
        
    }

    // Run the command.
    let res: Output = Command::new(the_shell).arg(the_arg).arg(cmd).output().unwrap();

    // Get the output and error and convert to string.
    let stdout: Cow<str> = String::from_utf8_lossy(res.stdout.as_slice());
    let stderr: Cow<str> = String::from_utf8_lossy(res.stderr.as_slice());

    // Check which output to return.
    if stderr.is_empty() {

        return stdout.to_string();

    } else {

        return stderr.to_string();
    }

}

fn main() {

  // Get commandline arguments
  let args:Vec<String> = env::args().collect();

  // Check for correct number of arguments.
  if args.len() == 2 {

    // Run the command.
    let res: String = executecmd(&args[1]);
    
    // Print the output.
    println!("{}", res);

  } else {

    println!("Usage: {} command", args[0])
  
  }

}
