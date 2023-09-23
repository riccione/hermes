use clap::{Parser, Subcommand};
use std::time::{SystemTime, UNIX_EPOCH};
use totp_lite::{totp_custom, Sha512, DEFAULT_STEP};
use std::fs::{File, OpenOptions};
//use std::io::Write;
use std::path::Path;
use std::io::{self, BufRead, Write};

/*
MFA state: 
- no encryption for codes
- save them in file similar to /etc/passwd

Usage: 
cargo run -- add -c secret -a alias
cargo run -- add --code secret --alias alias
*/

const FILE_CODES: &str = "codes";

#[derive(Parser)]
#[command(author, version, about, long_about = None)] // Read from Cargo.toml
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Adds code to the hermes
    Add {
        #[clap(short = 'c', long)]
        code: Option<String>,
        #[clap(short = 'a', long)]
        alias: Option<String>,
    },
    /// Remove code from the hermes
    Remove {
        #[clap(short = 'a', long)]
        alias: Option<String>,
    },
    /// Update code by alias
    Update {
        #[clap(short = 'c', long)]
        code: Option<String>,
        #[clap(short = 'a', long)]
        alias: Option<String>,
    },
    /// Get code by alias
    Get {
        #[clap(short = 'a', long)]
        alias: Option<String>,
    },
}

fn main() {
    let args = Args::parse();
    
    //let x, y =
    match &args.command {
        Commands::Add { code, alias } => add(code, alias),
        Commands::Remove { alias } => remove(alias),
        Commands::Update { code, alias } => update(code, alias),
        Commands::Get { alias } => get(alias),
    };
    /*
    match x {
        Some(x) => {
            generate_otp(&x);
            //println!("x is {x}");
        }
        None => {
            println!("Nothing");
        }
    }
    println!("{x:?}");
    */
}

fn add(code: &Option<String>, alias: &Option<String>) {
    let x = alias.clone().unwrap();
    let y = code.clone().unwrap();
    // create a storage file if it does not exist
    let data = format!("{}:{}\n", x, y);
    
    if file_exists() == true {
        // check if alias already exists and return error message
        if alias_exists(&x) == true {
            println!("Alias already exists, please select another one");
            std::process::exit(1);
        }
        let mut data_file = OpenOptions::new()
            .append(true)
            .open(FILE_CODES)
            .expect("cannot open file");
        data_file
            .write(data.as_bytes())
            .expect("write failed");
    } else {
        write_to_file(&data);
    }
    match code {
        Some(code) => { generate_otp(&code); },
        None => { println!("Nothing to code") }
    }
    println!("{code:?}{alias:?}");
}

fn update(code: &Option<String>, alias: &Option<String>) {
    println!("{code:?}{alias:?}");
}

fn remove(alias: &Option<String>) {
    if file_exists() == true {
        // read codes file and search for alias
        let mut data = "".to_owned();
        if let Ok(lines) = read_lines("codes") {
            for line in lines {
                if let Ok(l) = line {
                    let x: Vec<_> = l.split(":").collect();
                    if x[0] != alias.clone().unwrap() {
                        data = data + &l + "\n";
                    }
                }
            }
            write_to_file(&data);
        }
    } else {
        println!("Codes file does not exist. First add a code.");
    }
}

fn get(alias: &Option<String>) {
    if file_exists() == false {
        println!("codes file does not exist");
    } else {    
        // read codes file and search for alias
        if let Ok(lines) = read_lines("codes") {
            for line in lines {
                if let Ok(l) = line {
                    let x: Vec<_> = l.split(":").collect();
                    if x[0] == alias.clone().unwrap() {
                        generate_otp(x[1]);
                    }
                }
            }
        }
    }
}

fn file_exists() -> bool {
    Path::new(FILE_CODES).exists()
}

fn alias_exists(alias: &str) -> bool {
    // read codes file and search for alias
    if let Ok(lines) = read_lines(FILE_CODES) {
        for line in lines {
            if let Ok(l) = line {
                let x: Vec<_> = l.split(":").collect();
                if x[0] == alias {
                    return true;
                }
            }
        }
    }
    false
}

fn write_to_file(data: &str) {
    std::fs::write(FILE_CODES, data).expect("write failed");
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn generate_otp(x: &str) {
    let password = x.as_bytes();
    let seconds: u64 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    let result: String = totp_custom::<Sha512>(DEFAULT_STEP, 6, password, seconds);
    println!("{}", result);
}
