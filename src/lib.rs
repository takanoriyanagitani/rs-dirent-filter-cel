use cel::Context;
use cel::Value;
use cel::objects::{Key, Map};
use cel::{ExecutionError, FunctionContext, ResolveResult};

use std::collections::HashMap;
use std::os::unix::fs::{FileTypeExt, MetadataExt}; // Import FileTypeExt
#[allow(unused_imports)] // PathBuf is only used in tests
use std::path::{Path, PathBuf}; // PathBuf is used in tests
use std::sync::Arc; // This is crucial for nlink()

fn cel_parse_size(ftx: &FunctionContext, s: Arc<String>) -> ResolveResult {
    match parse_size::parse_size(&*s) {
        Ok(size) => Value::Int(size as i64).into(),
        Err(e) => ExecutionError::function_error(&ftx.name, e).into(),
    }
}

pub fn parser2ctx<'a>(ctx: &mut Context<'a>, name: &str) {
    ctx.add_function(name, cel_parse_size);
}

pub struct DirentInfo {
    pub name: String,
    pub is_file: bool,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub is_block_device: bool,
    pub is_char_device: bool,
    pub is_hidden: bool,
    pub is_readonly: bool,
    pub is_socket: bool,
    pub is_fifo: bool,
    pub len: u64,
    pub nlink: u64,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub mtime: i64,
    pub atime: i64,
    pub ctime: i64,
}

impl From<&Path> for DirentInfo {
    fn from(path: &Path) -> Self {
        let name = path
            .file_name()
            .and_then(|os_str| os_str.to_str())
            .unwrap_or("")
            .to_string();

        let metadata = path.symlink_metadata().ok(); // Use symlink_metadata to get info about the entry itself

        DirentInfo {
            name: name.clone(),
            is_file: metadata.as_ref().is_some_and(|m| m.is_file()),
            is_dir: metadata.as_ref().is_some_and(|m| m.is_dir()),
            is_symlink: metadata.as_ref().is_some_and(|m| m.is_symlink()),
            is_block_device: metadata.as_ref().is_some_and(|m| {
                #[cfg(unix)]
                {
                    m.file_type().is_block_device()
                }
                #[cfg(not(unix))]
                {
                    false
                }
            }),
            is_char_device: metadata.as_ref().is_some_and(|m| {
                #[cfg(unix)]
                {
                    m.file_type().is_char_device()
                }
                #[cfg(not(unix))]
                {
                    false
                }
            }),
            is_hidden: name.starts_with('.'),
            is_readonly: metadata
                .as_ref()
                .is_some_and(|m| m.permissions().readonly()),
            is_socket: metadata.as_ref().is_some_and(|m| {
                #[cfg(unix)]
                {
                    m.file_type().is_socket()
                }
                #[cfg(not(unix))]
                {
                    false
                }
            }),
            is_fifo: metadata.as_ref().is_some_and(|m| {
                #[cfg(unix)]
                {
                    m.file_type().is_fifo()
                }
                #[cfg(not(unix))]
                {
                    false
                }
            }),
            len: metadata.as_ref().map_or(0, |m| m.len()),
            nlink: metadata.as_ref().map_or(0, |m| {
                #[cfg(unix)]
                {
                    m.nlink()
                }
                #[cfg(not(unix))]
                {
                    1
                }
            }),
            mode: metadata.as_ref().map_or(0, |m| {
                #[cfg(unix)]
                {
                    m.mode()
                }
                #[cfg(not(unix))]
                {
                    0
                }
            }),
            uid: metadata.as_ref().map_or(0, |m| {
                #[cfg(unix)]
                {
                    m.uid()
                }
                #[cfg(not(unix))]
                {
                    0
                }
            }),
            gid: metadata.as_ref().map_or(0, |m| {
                #[cfg(unix)]
                {
                    m.gid()
                }
                #[cfg(not(unix))]
                {
                    0
                }
            }),
            mtime: metadata.as_ref().map_or(0, |m| {
                m.modified()
                    .map(|st| {
                        st.duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs() as i64
                    })
                    .unwrap_or(0)
            }),
            atime: metadata.as_ref().map_or(0, |m| {
                m.accessed()
                    .map(|st| {
                        st.duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs() as i64
                    })
                    .unwrap_or(0)
            }),
            ctime: metadata.as_ref().map_or(0, |m| {
                #[cfg(unix)]
                {
                    m.ctime()
                }
                #[cfg(not(unix))]
                {
                    0
                }
            }),
        }
    }
}

impl From<DirentInfo> for Value {
    fn from(dirent_info: DirentInfo) -> Self {
        let mut map = Map {
            map: Arc::new(HashMap::new()),
        };

        if let Some(internal_map) = Arc::get_mut(&mut map.map) {
            internal_map.insert(
                Key::String(Arc::new("name".to_string())),
                Value::String(dirent_info.name.into()),
            );
            internal_map.insert(
                Key::String(Arc::new("is_file".to_string())),
                Value::Bool(dirent_info.is_file),
            );
            internal_map.insert(
                Key::String(Arc::new("is_dir".to_string())),
                Value::Bool(dirent_info.is_dir),
            );
            internal_map.insert(
                Key::String(Arc::new("is_symlink".to_string())),
                Value::Bool(dirent_info.is_symlink),
            );
            internal_map.insert(
                Key::String(Arc::new("is_block_device".to_string())),
                Value::Bool(dirent_info.is_block_device),
            );
            internal_map.insert(
                Key::String(Arc::new("is_char_device".to_string())),
                Value::Bool(dirent_info.is_char_device),
            );
            internal_map.insert(
                Key::String(Arc::new("is_hidden".to_string())),
                Value::Bool(dirent_info.is_hidden),
            );
            internal_map.insert(
                Key::String(Arc::new("is_readonly".to_string())),
                Value::Bool(dirent_info.is_readonly),
            );
            internal_map.insert(
                Key::String(Arc::new("is_socket".to_string())),
                Value::Bool(dirent_info.is_socket),
            );
            internal_map.insert(
                Key::String(Arc::new("is_fifo".to_string())),
                Value::Bool(dirent_info.is_fifo),
            );
            internal_map.insert(
                Key::String(Arc::new("len".to_string())),
                Value::Int(dirent_info.len as i64),
            );
            internal_map.insert(
                Key::String(Arc::new("nlink".to_string())),
                Value::Int(dirent_info.nlink as i64),
            );
            internal_map.insert(
                Key::String(Arc::new("mode".to_string())),
                Value::Int(dirent_info.mode as i64),
            );
            internal_map.insert(
                Key::String(Arc::new("uid".to_string())),
                Value::Int(dirent_info.uid as i64),
            );
            internal_map.insert(
                Key::String(Arc::new("gid".to_string())),
                Value::Int(dirent_info.gid as i64),
            );
            internal_map.insert(
                Key::String(Arc::new("mtime".to_string())),
                Value::Int(dirent_info.mtime),
            );
            internal_map.insert(
                Key::String(Arc::new("atime".to_string())),
                Value::Int(dirent_info.atime),
            );
            internal_map.insert(
                Key::String(Arc::new("ctime".to_string())),
                Value::Int(dirent_info.ctime),
            );
        }

        Value::Map(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_dirent_info_from_path_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_file.txt");
        fs::File::create(&file_path)
            .unwrap()
            .write_all(b"hello")
            .unwrap();

        let metadata = fs::symlink_metadata(&file_path).unwrap();
        let expected_len = metadata.len();

        let dirent_info = DirentInfo::from(file_path.as_path());

        assert_eq!(dirent_info.name, "test_file.txt");
        assert!(dirent_info.is_file);
        assert!(!dirent_info.is_dir);
        assert!(!dirent_info.is_symlink);
        assert_eq!(dirent_info.len, expected_len);
    }

    #[test]
    fn test_dirent_info_from_path_dir() {
        let dir = tempdir().unwrap();
        let subdir_path = dir.path().join("test_dir");
        fs::create_dir(&subdir_path).unwrap();

        let dirent_info = DirentInfo::from(subdir_path.as_path());

        assert_eq!(dirent_info.name, "test_dir");
        assert!(!dirent_info.is_file);
        assert!(dirent_info.is_dir);
        assert!(!dirent_info.is_symlink);
    }

    #[test]
    fn test_dirent_info_from_path_symlink() {
        let dir = tempdir().unwrap();
        let target_file = dir.path().join("target.txt");
        fs::File::create(&target_file).unwrap();
        let symlink_path = dir.path().join("link_to_target.txt");
        #[cfg(unix)] // Symlinks are primarily a Unix-like feature
        std::os::unix::fs::symlink(&target_file, &symlink_path).unwrap();

        // This test block for symlinks is also OS-specific due to `symlink` function.
        #[cfg(unix)]
        {
            let dirent_info = DirentInfo::from(symlink_path.as_path());
            assert_eq!(dirent_info.name, "link_to_target.txt");
            assert!(!dirent_info.is_file); // symlink is not a file itself
            assert!(!dirent_info.is_dir); // symlink is not a dir itself
            assert!(dirent_info.is_symlink);
        }
    }

    #[test]
    fn test_dirent_info_non_existent_path() {
        let path = PathBuf::from("non_existent_file.txt");
        let dirent_info = DirentInfo::from(path.as_path());

        assert_eq!(dirent_info.name, "non_existent_file.txt");
        assert!(!dirent_info.is_file);
        assert!(!dirent_info.is_dir);
        assert!(!dirent_info.is_symlink);
        assert_eq!(dirent_info.len, 0);
        assert_eq!(dirent_info.nlink, 0);
        assert_eq!(dirent_info.mtime, 0);
    }

    #[test]
    fn test_dirent_info_to_cel_value() {
        let dirent_info = DirentInfo {
            name: "example.txt".to_string(),
            is_file: true,
            is_dir: false,
            is_symlink: false,
            is_block_device: false,
            is_char_device: false,
            is_hidden: false,
            is_readonly: true,
            is_socket: false,
            is_fifo: false,
            len: 123,
            nlink: 1,
            mode: 0o644,
            uid: 1000,
            gid: 1000,
            mtime: 1678886400, // March 15, 2023 12:00:00 AM UTC
            atime: 1678886400,
            ctime: 1678886400,
        };

        let cel_value: Value = dirent_info.into();

        if let Value::Map(map) = cel_value {
            let internal_map = map.map.as_ref();

            assert_eq!(
                internal_map.get(&Key::String(Arc::new("name".to_string()))),
                Some(&Value::String("example.txt".to_string().into()))
            );
            assert_eq!(
                internal_map.get(&Key::String(Arc::new("is_file".to_string()))),
                Some(&Value::Bool(true))
            );
            assert_eq!(
                internal_map.get(&Key::String(Arc::new("is_dir".to_string()))),
                Some(&Value::Bool(false))
            );
            assert_eq!(
                internal_map.get(&Key::String(Arc::new("is_symlink".to_string()))),
                Some(&Value::Bool(false))
            );
            assert_eq!(
                internal_map.get(&Key::String(Arc::new("is_block_device".to_string()))),
                Some(&Value::Bool(false))
            );
            assert_eq!(
                internal_map.get(&Key::String(Arc::new("is_char_device".to_string()))),
                Some(&Value::Bool(false))
            );
            assert_eq!(
                internal_map.get(&Key::String(Arc::new("is_hidden".to_string()))),
                Some(&Value::Bool(false))
            );
            assert_eq!(
                internal_map.get(&Key::String(Arc::new("is_readonly".to_string()))),
                Some(&Value::Bool(true))
            );
            assert_eq!(
                internal_map.get(&Key::String(Arc::new("is_socket".to_string()))),
                Some(&Value::Bool(false))
            );
            assert_eq!(
                internal_map.get(&Key::String(Arc::new("is_fifo".to_string()))),
                Some(&Value::Bool(false))
            );
            assert_eq!(
                internal_map.get(&Key::String(Arc::new("len".to_string()))),
                Some(&Value::Int(123))
            );
            assert_eq!(
                internal_map.get(&Key::String(Arc::new("nlink".to_string()))),
                Some(&Value::Int(1))
            );
            assert_eq!(
                internal_map.get(&Key::String(Arc::new("mode".to_string()))),
                Some(&Value::Int(0o644))
            );
            assert_eq!(
                internal_map.get(&Key::String(Arc::new("uid".to_string()))),
                Some(&Value::Int(1000))
            );
            assert_eq!(
                internal_map.get(&Key::String(Arc::new("gid".to_string()))),
                Some(&Value::Int(1000))
            );
            assert_eq!(
                internal_map.get(&Key::String(Arc::new("mtime".to_string()))),
                Some(&Value::Int(1678886400))
            );
            assert_eq!(
                internal_map.get(&Key::String(Arc::new("atime".to_string()))),
                Some(&Value::Int(1678886400))
            );
            assert_eq!(
                internal_map.get(&Key::String(Arc::new("ctime".to_string()))),
                Some(&Value::Int(1678886400))
            );
        } else {
            panic!("Expected a CEL Map Value");
        }
    }
}
