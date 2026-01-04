use dirs;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const FILE_CODEX: &str = "codex";
const PROJECT: &str = "hermes";

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

pub fn get_default_path() -> PathBuf {
    // using dirs fn to get location of config directory
    let mut codex_path = dirs::config_dir().expect("Failed to get config path");
    codex_path.push(PROJECT);
    codex_path.push(FILE_CODEX);
    codex_path
}

pub fn file_exists(path: &PathBuf) -> bool {
    Path::new(&path).exists()
}

pub fn read_file_to_vec(path: &PathBuf) -> io::Result<Vec<String>> {
    read_lines(path)
        .map_err(|_| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!(
                    "Codex file not found at {:?}. Use 'add' to create it.",
                    path
                ),
            )
        })?
        .collect()
}

pub fn write(path: &PathBuf, data: &str) -> io::Result<()> {
    let mut data_file = OpenOptions::new().append(true).open(path)?;
    writeln!(data_file, "{}", data.trim())
}

pub fn write_to_file(path: &PathBuf, data: &str, msg: &str) -> io::Result<()> {
    std::fs::write(path, data)?;
    println!("{msg}");
    Ok(())
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

pub fn create_path(path: &PathBuf) -> io::Result<()> {
    // only attempt to create directories if there is a parent component
    if let Some(parent) = path.parent() {
        // if the path is just "test.codex", parent() might be Some("") or empty
        // call create_dir_all if the parent isn't empty
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

fn perform_backup(path: &PathBuf, extension: String) -> io::Result<PathBuf> {
    if !path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Error: No Codex file found to backup.",
        ));
    }

    let mut backup_path = path.clone();
    backup_path.set_extension(extension);

    std::fs::copy(path, &backup_path)?;
    Ok(backup_path)
}

// routine backups for add and remove cmd
pub fn create_routine_backup(path: &PathBuf) -> io::Result<PathBuf> {
    perform_backup(path, "bak".to_string())
}

// backup for migration with UNIX timestamp
pub fn create_snapshot_backup(path: &PathBuf) -> io::Result<PathBuf> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    perform_backup(path, format!("{}.bak", timestamp))
}
