use std::{
    io::Seek,
    path::{Path, PathBuf},
};

use crate::common::skip_sinister_header;

#[derive(Debug)]
pub enum FileManagerError {
    FileNotFound(String),
    Io(std::io::Error),
}

impl From<std::io::Error> for FileManagerError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

pub struct FileManager {
    root: PathBuf,
}

#[derive(Debug)]
pub enum File {
    Standalone(std::fs::File),
    Archived(std::fs::File, u64, u64, bool),
}

impl File {
    pub fn size(&mut self) -> std::io::Result<u64> {
        match *self {
            File::Standalone(ref mut handle) => {
                let pos = handle.seek(std::io::SeekFrom::Current(0))?;
                let size = handle.seek(std::io::SeekFrom::End(0))?;
                handle.seek(std::io::SeekFrom::Start(pos))?;
                Ok(size)
            }
            File::Archived(_, _, size, _) => Ok(size),
        }
    }
}

impl std::io::Read for File {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match *self {
            Self::Standalone(ref mut handle) => handle.read(buf),
            Self::Archived(ref mut handle, offset, size, is_plain_text) => {
                let current_pos = handle.seek(std::io::SeekFrom::Current(0))? - offset;
                if current_pos + buf.len() as u64 > size {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        format!(
                            "failed to fill whole buffer (size: {}, current: {}, to read: {}, available: {})",
                            size,
                            current_pos,
                            buf.len(),
                            size - current_pos
                        ),
                    ));
                }
                handle.read(buf)?;
                if is_plain_text {
                    crate::common::decrypt_buf(buf);
                }
                Ok(buf.len())
            }
        }
    }
}

impl std::io::Seek for File {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        match *self {
            Self::Standalone(ref mut handle) => handle.seek(pos),
            Self::Archived(ref mut handle, offset, size, ..) => match pos {
                std::io::SeekFrom::Start(i) => handle.seek(std::io::SeekFrom::Start(offset + i)),
                std::io::SeekFrom::End(i) => handle.seek(std::io::SeekFrom::Start(
                    (offset + size).wrapping_add_signed(i),
                )),
                std::io::SeekFrom::Current(_) => {
                    // TODO: Check the bounds of the seek.
                    handle.seek(pos)
                }
            },
        }
    }
}

impl FileManager {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().into(),
        }
    }

    pub fn open_file(&self, path: impl AsRef<Path>) -> Result<File, FileManagerError> {
        // Check for standalone file.
        let standalone_path = self.root.join(path.as_ref());
        if standalone_path.exists() {
            return Ok(File::Standalone(std::fs::File::open(standalone_path)?));
        }

        // Search for the file in an archive.
        let mut gut_path = path.as_ref().to_path_buf();
        loop {
            if let Some(parent) = gut_path.parent() {
                gut_path = parent.to_path_buf();
            } else {
                return Err(FileManagerError::FileNotFound(
                    path.as_ref().to_string_lossy().to_string(),
                ));
            };
            let maybe_file = self.root.join(&gut_path).with_extension("gut");
            if maybe_file.exists() {
                break;
            }
        }

        let gut_path = self.root.join(&gut_path).with_extension("gut");
        // println!("gut_path: {gut_path:?}");

        let mut file = std::fs::File::open(gut_path)?;
        let header_size = skip_sinister_header(&mut file)?;

        let entries = crate::gut::read_gut_header(&mut file)?;

        if let Some(entry) = entries
            .iter()
            .find(|e| path.as_ref().to_string_lossy().to_lowercase() == e.name)
        {
            let offset = entry.offset + header_size;
            file.seek(std::io::SeekFrom::Start(offset))?;
            return Ok(File::Archived(
                file,
                offset,
                entry.size,
                entry.is_plain_text,
            ));
        }

        Err(FileManagerError::FileNotFound(
            path.as_ref().to_string_lossy().to_string(),
        ))
    }
}
