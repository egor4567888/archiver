//! Модуль для работы с файловой системой и сериализацией данных.
//!
//! Этот модуль предоставляет функции для чтения и записи файлов,
//! рекурсивного чтения директорий, а также сериализации и десериализации
//! данных для архивации.

use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use serde::{Deserialize, Serialize};
use crate::ArchiveData;

/// Представляет запись директории с путем, данными и правами доступа.
#[derive(Debug, Serialize, Deserialize)]
pub struct DirEntry {
    /// Относительный путь к файлу или директории
    pub path: String,  
    /// Содержимое файла в виде байтов
    pub data: Vec<u8>,
    /// Права доступа к файлу
    pub permissions: u32,
}

/// Читает содержимое файла по указанному пути и возвращает его как вектор байтов.
///
/// # Аргументы
///
/// * `path` - Строка с путем к файлу.
///
/// # Возвращает
///
/// Результат с вектором байтов или ошибкой ввода/вывода.
pub fn read_file(path: &str) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?; // Открытие файла
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?; // Чтение содержимого файла
    Ok(buffer)
}

/// Записывает данные в файл по указанному пути.
///
/// # Аргументы
///
/// * `path` - Строка с путем к файлу.
/// * `data` - Срез байтов для записи.
///
/// # Возвращает
///
/// Результат операции или ошибку ввода/вывода.
pub fn write_file(path: &str, data: &[u8]) -> io::Result<()> {
    let mut file = File::create(path)?; // Создание файла
    file.write_all(data)?; // Запись данных в файл
    Ok(())
}

/// Рекурсивно читает директорию и собирает информацию о каждом файле.
///
/// # Аргументы
///
/// * `current_path` - Текущий путь для чтения.
/// * `root_path` - Корневой путь для определения относительных путей.
///
/// # Возвращает
///
/// Вектор записей `DirEntry` или ошибку ввода/вывода.
pub fn read_dir_recursive(current_path: &Path, root_path: &Path) -> io::Result<Vec<DirEntry>> {
    let mut entries = Vec::new();
    if current_path.is_file() {
        let data = read_file(current_path.to_str().unwrap())?; // Чтение файла
        let perm = fs::metadata(current_path)?.permissions().mode(); // Получение прав доступа
        let rel_path = current_path.strip_prefix(root_path)
            .unwrap_or(current_path)
            .to_str().unwrap()
            .to_owned(); // Относительный путь
        entries.push(DirEntry {
            path: rel_path,
            data,
            permissions: perm,
        });
    } else if current_path.is_dir() {
        for entry in fs::read_dir(current_path)? { // Чтение содержимого директории
            let entry = entry?;
            let path = entry.path();
            let mut sub_entries = read_dir_recursive(&path, root_path)?; // Рекурсивный вызов
            entries.append(&mut sub_entries);
        }
    }
    Ok(entries)
}

/// Записывает записи директории на диск по базовому пути.
///
/// # Аргументы
///
/// * `entries` - Срез записей `DirEntry`.
/// * `base_path` - Базовый путь для создания файлов.
///
/// # Возвращает
///
/// Результат операции или ошибку ввода/вывода.
pub fn write_dir_entries(entries: &[DirEntry], base_path: &Path) -> io::Result<()> {
    for e in entries {
        let real_path = base_path.join(&e.path); // Формирование полного пути
        if let Some(parent) = real_path.parent() {
            fs::create_dir_all(parent)?; // Создание всех родительских директорий
        }
        let mut file = File::create(&real_path)?; // Создание файла
        file.write_all(&e.data)?; // Запись данных в файл
        fs::set_permissions(&real_path, fs::Permissions::from_mode(e.permissions))?; // Установка прав доступа
    }
    Ok(())
}

/// Преобразует `DirEntry` в байты для сериализации.
///
/// # Аргументы
///
/// * `entry` - Ссылка на запись `DirEntry`.
///
/// # Возвращает
///
/// Вектор байтов, представляющих `DirEntry`.
pub fn dir_entry_to_bytes(entry: &DirEntry) -> Vec<u8> {
    let mut result = Vec::new();

    // Запись прав доступа (4 байта)
    result.extend_from_slice(&entry.permissions.to_le_bytes());

    // Запись пути
    let path_bytes = entry.path.as_bytes();
    let path_len = path_bytes.len() as u32;
    result.extend_from_slice(&path_len.to_le_bytes());
    result.extend_from_slice(path_bytes);

    // Запись данных файла
    let data_len = entry.data.len() as u32;
    result.extend_from_slice(&data_len.to_le_bytes());
    result.extend_from_slice(&entry.data);

    result
}

/// Преобразует байты в `DirEntry` для десериализации.
///
/// # Аргументы
///
/// * `data` - Срез байтов для преобразования.
///
/// # Возвращает
///
/// Результат с `DirEntry` или ошибкой ввода/вывода.
pub fn bytes_to_dir_entry(data: &[u8]) -> std::io::Result<DirEntry> {
    use std::convert::TryInto;
    let mut offset = 0;

    // Чтение прав доступа
    let permissions = u32::from_le_bytes(data[offset..offset+4].try_into().unwrap());
    offset += 4;

    // Чтение пути
    let path_len = u32::from_le_bytes(data[offset..offset+4].try_into().unwrap()) as usize;
    offset += 4;
    let path_bytes = &data[offset..offset+path_len];
    offset += path_len;
    let path_str = String::from_utf8(path_bytes.to_vec())
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Неверный формат"))?;

    // Чтение данных файла
    let data_len = u32::from_le_bytes(data[offset..offset+4].try_into().unwrap()) as usize;
    offset += 4;
    let file_data = data[offset..offset+data_len].to_vec();

    Ok(DirEntry {
        path: path_str,
        data: file_data,
        permissions,
    })
}

/// Преобразование ArchiveData в байты
pub fn archive_data_to_bytes(archive: &ArchiveData) -> Vec<u8> {
    let mut buffer = Vec::new();

    // Запись количества записей DirEntry
    let entries_len = archive.entries.len() as u32;
    buffer.extend_from_slice(&entries_len.to_le_bytes());

    // Запись каждой записи DirEntry
    for entry in &archive.entries {
        let entry_bytes = dir_entry_to_bytes(entry);
        let entry_size = entry_bytes.len() as u32;

        // Сначала записываем размер записи
        buffer.extend_from_slice(&entry_size.to_le_bytes());
        // Затем сами байты записи
        buffer.extend_from_slice(&entry_bytes);
    }

    buffer
}

/// Преобразует байты в `ArchiveData` для десериализации.
///
/// # Аргументы
///
/// * `data` - Срез байтов для преобразования.
///
/// # Возвращает
///
/// Результат с `ArchiveData` или ошибкой ввода/вывода.
pub fn bytes_to_archive_data(data: &[u8]) -> io::Result<ArchiveData> {
    use std::convert::TryInto;
    let mut offset = 0;

    // Чтение количества записей `DirEntry`
    if data.len() < 4 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Недостаточно данных для чтения количества записей"));
    }
    let entries_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
    offset += 4;

    let mut entries = Vec::with_capacity(entries_len);

    for _ in 0..entries_len {
        if data.len() < offset + 4 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Недостаточно данных для чтения размера записи"));
        }
        // Чтение размера записи `DirEntry`
        let entry_size = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;

        if data.len() < offset + entry_size {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Недостаточно данных для чтения записи"));
        }
        // Чтение байтов записи `DirEntry`
        let entry_bytes = &data[offset..offset + entry_size];
        offset += entry_size;

        // Восстановление записи `DirEntry` из байтов
        let dir_entry = bytes_to_dir_entry(entry_bytes)?;
        entries.push(dir_entry);
    }

    Ok(ArchiveData { entries })
}