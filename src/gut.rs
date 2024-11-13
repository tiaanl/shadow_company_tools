use std::{
    io::{BufRead, Cursor, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{
    common::{decrypt_buf, hash},
    io::Reader,
};

pub struct Entry {
    pub name: String,
    pub offset: u64,
    pub size: u64,
    pub is_plain_text: bool,
    pub hash: u32,
}

pub struct GutFile {
    path: PathBuf,
    data: Vec<u8>,
}

impl GutFile {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let mut s = Self {
            path: path.as_ref().to_owned(),
            data: vec![],
        };
        s.ensure_data()?;
        Ok(s)
    }

    pub fn read_entries(&self) -> Vec<Entry> {
        let mut c = Cursor::new(&self.data);

        // lines are separated by [0x0D, 0x0A]
        let mut header = vec![];
        for _ in 0..9 {
            c.read_until(b'\n', &mut header).unwrap();
        }

        // 8 - Unknown (maybe hash?)
        // 4 - file count
        // 32 - filename (null)

        c.seek(SeekFrom::Current(8)).unwrap();

        let file_count = c.read_u32::<LittleEndian>().unwrap();

        let mut filename: [u8; 32] = [0; 32];
        c.read_exact(&mut filename).unwrap();

        let header_size = c.position();

        let mut entries = vec![];

        // 4 - Filename Length
        // 4 - File Size
        // 4 - File Offset [+headerSize]
        // 4 - null
        // 4 - hash

        for _ in 0..file_count {
            let filename_length = c.read_u32::<LittleEndian>().unwrap();
            let file_size = c.read_u32::<LittleEndian>().unwrap();
            let file_offset = c.read_u32::<LittleEndian>().unwrap();
            let is_text = c.read_u32::<LittleEndian>().unwrap() != 0;
            let filename_hash = c.read_u32::<LittleEndian>().unwrap();
            let mut encrypted_filename = vec![0; filename_length as usize];
            c.read_exact(&mut encrypted_filename).unwrap();
            crate::common::decrypt_buf(encrypted_filename.as_mut());

            if filename_hash != hash(encrypted_filename.as_ref()) {
                eprintln!(
                    "hashes do not match for file: {}",
                    String::from_utf8_lossy(&encrypted_filename)
                );
            }

            entries.push(Entry {
                name: String::from_utf8_lossy(
                    encrypted_filename[0..(filename_length - 1) as usize].as_mut(),
                )
                .to_string(),
                offset: file_offset as u64 + header_size,
                size: file_size as u64,
                is_plain_text: is_text,
                hash: filename_hash,
            });
        }

        entries
    }

    pub fn get_contents(&mut self, entry: &Entry) -> &mut [u8] {
        let data =
            &mut self.data[entry.offset as usize..entry.offset as usize + entry.size as usize];
        if entry.is_plain_text {
            decrypt_buf(data);
        }
        data
    }

    fn ensure_data(&mut self) -> Result<(), std::io::Error> {
        if self.data.is_empty() {
            self.data = std::fs::read(&self.path)?;
        }
        Ok(())
    }
}

pub fn read_gut_header(r: &mut impl Reader) -> std::io::Result<Vec<Entry>> {
    let entry_count = r.read_u32::<LittleEndian>()?;

    // Skip the name of the .gut file, which is a fixed length string
    // of 32 characters.
    r.seek(SeekFrom::Current(32))?;

    // According to the offsets for each entry, the header stops here.
    // Which is 36 bytes from where we started to read.

    let mut entries = Vec::with_capacity(entry_count as usize);
    for _ in 0..entry_count {
        let filename_length = r.read_u32::<LittleEndian>().unwrap();
        let file_size = r.read_u32::<LittleEndian>().unwrap();
        let file_offset = r.read_u32::<LittleEndian>().unwrap();
        let is_text = r.read_u32::<LittleEndian>().unwrap() != 0;
        let filename_hash = r.read_u32::<LittleEndian>().unwrap();
        let mut encrypted_filename = vec![0; filename_length as usize];
        r.read_exact(&mut encrypted_filename).unwrap();
        crate::common::decrypt_buf(encrypted_filename.as_mut());

        if filename_hash != hash(encrypted_filename.as_ref()) {
            eprintln!(
                "hashes do not match for file: {}",
                String::from_utf8_lossy(&encrypted_filename)
            );
        }

        entries.push(Entry {
            name: String::from_utf8_lossy(
                encrypted_filename[0..(filename_length - 1) as usize].as_mut(),
            )
            .to_string(),
            offset: file_offset as u64 + 36,
            size: file_size as u64,
            is_plain_text: is_text,
            hash: filename_hash,
        });
    }

    Ok(entries)
}
