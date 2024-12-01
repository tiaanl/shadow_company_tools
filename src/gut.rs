use byteorder::{LittleEndian as LE, ReadBytesExt};
use thiserror::Error;

use crate::{common::hash, io::Reader};

#[derive(Debug, Error)]
pub enum GutError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug)]
pub struct Entry {
    pub name: String,
    pub offset: u64,
    pub size: u64,
    pub is_plain_text: bool,
    pub hash: u32,
}

#[derive(Debug)]
pub struct GutFile {
    /// The size of the header and start of the data of the first entry.
    pub header_size: u64,

    entries: Vec<Entry>,
}

impl GutFile {
    const MAGIC: &[u32; 2] = &[0xC131FA1A, 0x1342EDDE];

    /// Open a .gut file and read the entries from its header.
    pub fn open(reader: &mut impl Reader) -> Result<Self, GutError> {
        let header_size = reader.skip_sinister_header_2(Self::MAGIC, 0x4000)?;
        let entries = read_entries(reader)?;

        Ok(Self {
            header_size,
            entries,
        })
    }

    /// Get an iterator over the entries in the .gut file.
    pub fn entries(&self) -> EntryIter<'_> {
        EntryIter {
            entries: self.entries.as_ref(),
            current: 0,
        }
    }
}

fn read_entries(reader: &mut impl Reader) -> std::io::Result<Vec<Entry>> {
    // 4 - file count
    // 32 - filename

    let file_count = reader.read_u32::<LE>()?;
    let _filename = reader.read_fixed_string(32)?;

    let header_size = reader.stream_position()?;

    let mut entries = vec![];

    // 4 - Filename Length
    // 4 - File Size
    // 4 - File Offset [+headerSize]
    // 4 - null
    // 4 - hash

    for _ in 0..file_count {
        let filename_length = reader.read_u32::<LE>()?;
        let file_size = reader.read_u32::<LE>()?;
        let file_offset = reader.read_u32::<LE>()?;
        let is_text = reader.read_u32::<LE>()? != 0;
        let filename_hash = reader.read_u32::<LE>()?;
        let mut encrypted_filename = vec![0; filename_length as usize];
        reader.read_exact(&mut encrypted_filename)?;
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

    Ok(entries)
}

pub struct EntryIter<'entries> {
    entries: &'entries [Entry],
    current: usize,
}

impl<'entries> Iterator for EntryIter<'entries> {
    type Item = &'entries Entry;

    fn next(&mut self) -> Option<Self::Item> {
        self.entries.get({
            let index = self.current;
            self.current += 1;
            index
        })
    }
}
