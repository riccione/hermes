use crate::models::Record;
use dirs;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const FILE_CODEX: &str = "codex";
const PROJECT: &str = "hermes";

pub fn get_default_path() -> PathBuf {
    // using dirs fn to get location of config directory
    dirs::config_dir()
        .map(|mut path| {
            path.push(PROJECT);
            path.push(FILE_CODEX);
            path
        })
        .expect("Error: Failed to get config path")
}

pub fn file_exists(path: &Path) -> bool {
    path.exists()
}

fn read_lines<P: AsRef<Path>>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>> {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

pub fn read_file_to_vec(path: &Path) -> io::Result<Vec<String>> {
    read_lines(path)?
        .collect::<io::Result<Vec<String>>>()
        .map_err(|e| {
            io::Error::new(
                e.kind(),
                format!("Error reading codex at {:?}: {e}", path),
            )
        })
}

pub fn append_to_file(path: &Path, data: &str) -> io::Result<()> {
    let mut data_file = OpenOptions::new().append(true).open(path)?;
    writeln!(data_file, "{}", data.trim())
}

pub fn overwrite_file(path: &Path, data: &str) -> io::Result<()> {
    std::fs::write(path, data)
}

pub fn alias_exists(alias: &str, path: &Path) -> bool {
    read_file_to_vec(path)
        .map(|lines| {
            lines.iter().any(|line| {
                Record::from_line(line)
                    .map(|r| r.alias == alias)
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

pub fn ensure_dir_exists(path: &Path) -> io::Result<()> {
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

fn perform_backup(path: &Path, extension: &str) -> io::Result<PathBuf> {
    if !path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Error: No Codex file found to backup.",
        ));
    }

    let mut backup_path = path.to_path_buf();
    backup_path.set_extension(extension);

    std::fs::copy(path, &backup_path)?;
    Ok(backup_path)
}

// routine backups for add and remove cmd
pub fn create_routine_backup(path: &Path) -> io::Result<PathBuf> {
    perform_backup(path, "bak")
}

// backup for migration with UNIX timestamp
pub fn create_snapshot_backup(path: &Path) -> io::Result<PathBuf> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    perform_backup(path, &format!("{}.bak", timestamp))
}
