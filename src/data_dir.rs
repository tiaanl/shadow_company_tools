use std::{
    io::Seek,
    path::{Path, PathBuf},
};

use thiserror::Error;

use crate::{
    gut::{GutError, GutFile},
    io::PathExt,
};

#[derive(Debug, Error)]
pub enum DataDirError {
    #[error("File not found: {0}")]
    FileNotFound(String),
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("GUT file error: {0}")]
    GutError(#[from] GutError),
}

pub struct DataDir {
    root: PathBuf,
}

#[derive(Debug)]
pub enum File {
    Standalone {
        file: std::fs::File,
    },
    Archived {
        gut_file: std::fs::File,
        offset: u64,
        size: u64,
        is_plain_text: bool,
    },
}

impl File {
    pub fn size(&mut self) -> std::io::Result<u64> {
        match *self {
            File::Standalone { ref mut file } => {
                let pos = file.stream_position()?;
                let size = file.seek(std::io::SeekFrom::End(0))?;
                file.seek(std::io::SeekFrom::Start(pos))?;
                Ok(size)
            }
            File::Archived { size, .. } => Ok(size),
        }
    }
}

impl std::io::Read for File {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match *self {
            Self::Standalone { ref mut file } => file.read(buf),
            Self::Archived {
                ref mut gut_file,
                offset,
                size,
                is_plain_text,
            } => {
                // Seek to the start of the data.
                let current_pos = gut_file.stream_position()?;
                if current_pos < offset {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        format!(
                            "file pointer is outside of bounds (current: {})",
                            current_pos
                        ),
                    ));
                }
                if current_pos - offset + buf.len() as u64 > size {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        format!(
                            "failed to fill whole buffer (size: {}, current: {}, to read: {}, available: {})",
                            size,
                            current_pos,
                            buf.len(),
                            size - (current_pos - offset)
                        ),
                    ));
                }

                gut_file.read_exact(buf)?;
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
            Self::Standalone { ref mut file } => file.seek(pos),
            Self::Archived {
                ref mut gut_file,
                offset,
                size,
                ..
            } => match pos {
                std::io::SeekFrom::Start(i) => gut_file.seek(std::io::SeekFrom::Start(offset + i)),
                std::io::SeekFrom::End(i) => gut_file.seek(std::io::SeekFrom::Start(
                    (offset + size).wrapping_add_signed(i),
                )),
                std::io::SeekFrom::Current(_) => {
                    // TODO: Check the bounds of the seek.
                    gut_file.seek(pos)
                }
            },
        }
    }
}

impl DataDir {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().into(),
        }
    }

    pub fn open(&self, path: impl AsRef<Path>) -> Result<File, DataDirError> {
        // Check for an external file. Make sure the separators are for the OS as we're checking the
        // file system directly.
        let external_path = self.root.join(path.as_ref().with_os_separators());
        if external_path.exists() {
            return Ok(File::Standalone {
                file: std::fs::File::open(external_path)?,
            });
        }

        // Check if we can find the entry in a .gut file.
        if let Some(gut_file_path) = self.find_gut_file_path_for(path.as_ref()) {
            let mut file = std::fs::File::open(&gut_file_path)?;
            let gut = GutFile::open(&mut file)?;

            // We check the entries using the data dir separators.
            let path_as_str = path.as_ref().with_data_dir_separators();
            let path_as_str = path_as_str.to_string_lossy();

            // See if the entry is inside the .gut file.
            let entry = gut
                .entries()
                .find(|entry| entry.name.eq_ignore_ascii_case(&path_as_str));

            if let Some(entry) = entry {
                let mut gut_file = std::fs::File::open(gut_file_path)?;
                // Seek the file to the start of the data.
                gut_file.seek(std::io::SeekFrom::Start(entry.offset))?;
                return Ok(File::Archived {
                    gut_file,
                    offset: entry.offset,
                    size: entry.size,
                    is_plain_text: entry.is_plain_text,
                });
            }
        }

        Err(DataDirError::FileNotFound(format!(
            "{}",
            path.as_ref().display()
        )))
    }

    fn find_gut_file_path_for(&self, path: impl AsRef<Path>) -> Option<PathBuf> {
        // Use OS separators for the path, because we'll be checking the filesystem with it.
        let path = path.as_ref().with_os_separators();

        let first = path.components().next()?;
        let gut_file = self.root.join(first).with_extension("gut");

        gut_file.exists().then_some(gut_file)
    }
}
