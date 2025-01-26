use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::fs::{self, create_dir_all};

#[derive(Debug)]
pub struct DirEntry {
    pub path: String,
    pub data: Vec<u8>,
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


pub fn read_directory(root_dir: &str) -> io::Result<Vec<DirEntry>> {
    let mut entries = Vec::new();
    fn visit_dir(current_path: &Path, root_dir: &Path, entries: &mut Vec<DirEntry>) -> io::Result<()> {
        for entry in fs::read_dir(current_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dir(&path, root_dir, entries)?;
            } else {
                let mut file = File::open(&path)?;
                let mut buf = Vec::new();
                file.read_to_end(&mut buf)?;
                let rel_path = path.strip_prefix(root_dir).unwrap();
                entries.push(DirEntry {
                    path: rel_path.to_string_lossy().to_string(),
                    data: buf,
                });
            }
        }
        Ok(())
    }
    visit_dir(Path::new(root_dir), Path::new(root_dir), &mut entries)?;
    Ok(entries)
}

pub fn write_directory(dir_path: &str, entries: &[DirEntry]) -> io::Result<()> {
    for e in entries {
        let full_path = Path::new(dir_path).join(&e.path);
        if let Some(parent) = full_path.parent() {
            create_dir_all(parent)?;
        }
        let mut file = File::create(&full_path)?;
        file.write_all(&e.data)?;
    }
    Ok(())
}

pub fn pack_directory(entries: &[DirEntry]) -> Vec<u8> {
    let mut out = Vec::new();
    for e in entries {
        let path_bytes = e.path.as_bytes();
        let path_len = path_bytes.len() as u32;
        out.extend_from_slice(&path_len.to_le_bytes());
        out.extend_from_slice(&path_bytes);
        let data_len = e.data.len() as u64;
        out.extend_from_slice(&data_len.to_le_bytes());
        out.extend_from_slice(&e.data);
    }
    out
}

pub fn unpack_directory(data: &[u8]) -> Vec<DirEntry> {
    let mut idx = 0;
    let mut result = Vec::new();
    while idx < data.len() {
        let path_len = u32::from_le_bytes(data[idx..idx+4].try_into().unwrap()) as usize;
        idx += 4;
        let path = String::from_utf8(data[idx..idx+path_len].to_vec()).unwrap();
        idx += path_len;
        let data_len = u64::from_le_bytes(data[idx..idx+8].try_into().unwrap()) as usize;
        idx += 8;
        let file_data = data[idx..idx+data_len].to_vec();
        idx += data_len;
        result.push(DirEntry { path, data: file_data });
    }
    result
}