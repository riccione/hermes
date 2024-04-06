use std::path::{Path, PathBuf};
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use dirs;

const FILE_CODEX: &str = "codex";
const PROJECT: &str = "hermes";

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

pub fn get_codex_path() -> PathBuf {
    // using dirs fn to get location of config directory
    let mut codex_path = dirs::config_dir()
        .expect("Failed to get config path");
    codex_path.push(PROJECT);
    codex_path.push(FILE_CODEX);
    codex_path
}

pub fn file_exists(path: &PathBuf) -> bool {
    Path::new(&path).exists()
}

pub fn read_file_to_vec(path: &PathBuf) -> Vec<String> {
    if file_exists(path) {
        let file = File::open(path)
            .expect("There is no codex file");
        let file = BufReader::new(file);
        file.lines()
            .map(|x| x.expect("Could not parse line"))
            .collect()
    } else {
        println!("{FILE_CODEX} file does not exist.\n\
            Please use hermes add command\n\
            or copy existing {FILE_CODEX} to a default location");
        std::process::exit(1);
    }
}

pub fn write(path: &PathBuf, data: &str) {
    let mut data_file = OpenOptions::new()
        .append(true)
        .open(path)
        .expect("Cannot open file");
    data_file
        .write(data.as_bytes())
        .expect("Failed to write codex file");
}

pub fn write_to_file(path: &PathBuf, data: &str, msg: &str) {
    match std::fs::write(path, data) {
        Ok(_) => {
            println!("{msg}");
        }
        Err(e) => {
            eprintln!("Failed to save codex. Error: {e}");
        }
    }
}

pub fn alias_exists(alias: &str, codex_path: &PathBuf) -> bool {
    // read codes file and search for alias
    if let Ok(lines) = read_lines(codex_path) {
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

pub fn create_path(path: &PathBuf) -> bool {
    let mut p: PathBuf = path.to_path_buf();
    p.pop();
    match std::fs::create_dir_all(&p) {
        Ok(_) => true,
        Err(_e) => false,
    }
}
