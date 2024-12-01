use sha1::Digest;
use clap::{Arg, Command};
use std::path::Path;
use std::{

/*
 *     env, 
 *     Used if I want to use environment variables. 
*/
    fs,
    error::Error,
    fs::File,
    io::{BufRead, BufReader, Write},
};

const SHA1_HEX_STRING_LENGTH: usize = 40;

/* Function to check if a file is empty */
fn is_file_empty(file_path: &str) -> std::io::Result<bool> {
    
    let metadata = fs::metadata(file_path)?;
    Ok(metadata.len() == 0)

}

/* Function to display verbose output. Return a bool and string */
fn is_verbose(matches: &clap::ArgMatches) -> bool {

   if matches.get_flag("verbose") {

       return true;
   }

   return false;

}

fn main() -> Result<(), Box<dyn Error>> {

    // Get command line arguments.
    /*let args: Vec<String> = env::args().collect();

    // Check for correct number of arguments.
     if args.len() != 3 {

        println!("usage:");
        println!("sha1_cracker: <wordlist> <sha1_hash>");
        return Ok(());

    }*/

    // CLI arguments.
    let matches = Command::new("SHA1 Checker")
        .version("0.1")
        .author("Duane Dunston <thedunston@gmail.com>")
        .about("First Rust Program enchanced.")
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Display verbose output")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("file")
                .short('f')
                .long("file")
                .value_name("FILE")
                .help("Specifies the wordlist file.")
                .required(true)
        )
        .arg(
            Arg::new("hash")
                .long("hash")
                .value_name("HASH")
                .help("Hash to search for in the wordlist.")
                .required(true)
        )
    .get_matches();

    // Check if a file and hash argument is provided
    let file = matches.get_one::<String>("file").expect("File argument is required.");
    let hash_to_crack = matches.get_one::<String>("hash").expect("Hash argument is required.").trim();

    // Check that the wordlist file exists.
    if !Path::new(file).exists() {

        return Err("Wordlist file does not exist".into());

    }

    // Check if the file is empty.
    if is_file_empty(file)? {

        return Err("Wordlist file is empty".into());

    }

    // Check if the hash is valid by checking the length.
    if hash_to_crack.len() != SHA1_HEX_STRING_LENGTH {

        return Err("sha1 has is not valid".into());

    }

    // Open the wordlist and read each line.
    let wordlist_file = File::open(&file)?;

    // Read the wordlist line by line.
    let reader = BufReader::new(&wordlist_file);
    
     // Debug file if verbose is set to true.
     let mut debug_file = None;
  
     if is_verbose(&matches) {
     
         let debug_filename = format!("{}.debug", file);
         debug_file = Some(File::create(&debug_filename)?);

    }

    // Loop through the wordlist.
    for line in reader.lines() {
    
        let line = line?;
        let common_password = line.trim();
        let gen_hash = &hex::encode(sha1::Sha1::digest(common_password.as_bytes()));

        // If verbose is set to true, save the output to a file.
        if is_verbose(&matches) { 
           
            if let Some(debug_file) = debug_file.as_mut() {
           
                writeln!(debug_file, "Checking password: {}", common_password)?;
                writeln!(debug_file, "Generated hash: {}", gen_hash)?;
           
            }
            // let message = format!("Checking password: {}", common_password); println!("{}", message)
        
        }   

        // If verbose is set to true, print the hash.
        if is_verbose(&matches) {
            
             let message = format!("Generated hash: {}", gen_hash); println!("{}", message)
        
        }
        
        // Check if the password in the wordlist matches the hash.
        if hash_to_crack == gen_hash {

             // Write to a file and print the results.
             let output_filename = format!("{}.result", file);
             let mut output_file = File::create(&output_filename)?;
             writeln!(output_file, "Password found: {} - {}", common_password, gen_hash)?;
  
             // Print the results.
             println!("Password found: {} - {}", &common_password, gen_hash);

                return Ok(())

        }
    
    }

    println!("Password not found in wordlist :(");

    Ok(())

}
