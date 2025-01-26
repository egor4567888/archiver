use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path};
use std::fs::{self};
use std::os::unix::fs::PermissionsExt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DirEntry {
    pub path: String,  
    pub data: Vec<u8>,
    pub permissions: u32,
}


pub fn read_file(path: &str) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

pub fn write_file(path: &str, data: &[u8]) -> io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(data)?;
    Ok(())
}

pub fn read_dir_recursive(current_path: &Path, root_path: &Path) -> io::Result<Vec<DirEntry>> {
    let mut entries = Vec::new();
    if current_path.is_file() {
        let data = read_file(current_path.to_str().unwrap())?;
        let perm = fs::metadata(current_path)?.permissions().mode();
        let rel_path = current_path.strip_prefix(root_path)
            .unwrap_or(current_path)
            .to_str().unwrap()
            .to_owned();
        entries.push(DirEntry {
            path: rel_path,
            data,
            permissions: perm,
        });
    } else if current_path.is_dir() {
        for entry in fs::read_dir(current_path)? {
            let entry = entry?;
            let path = entry.path();
            let mut sub_entries = read_dir_recursive(&path, root_path)?;
            entries.append(&mut sub_entries);
        }
    }
    Ok(entries)
}

pub fn write_dir_entries(entries: &[DirEntry], base_path: &Path) -> io::Result<()> {
    for e in entries {
        let real_path = base_path.join(&e.path);
        if let Some(parent) = real_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = File::create(&real_path)?;
        file.write_all(&e.data)?;
        fs::set_permissions(&real_path, fs::Permissions::from_mode(e.permissions))?;
    }
    Ok(())
}
