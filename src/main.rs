use byteorder::{LittleEndian, ReadBytesExt};
use std::{
    io::{BufRead, Cursor, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};

const HASH_LOOKUP_TABLE: [u16; 256] = [
    0x0000, 0x1021, 0x2042, 0x3063, 0x4084, 0x50A5, 0x60C6, 0x70E7, 0x8108, 0x9129, 0xA14A, 0xB16B,
    0xC18C, 0xD1AD, 0xE1CE, 0xF1EF, 0x1231, 0x210, 0x3273, 0x2252, 0x52B5, 0x4294, 0x72F7, 0x62D6,
    0x9339, 0x8318, 0xB37B, 0xA35A, 0xD3BD, 0xC39C, 0xF3FF, 0xE3DE, 0x2462, 0x3443, 0x420, 0x1401,
    0x64E6, 0x74C7, 0x44A4, 0x5485, 0xA56A, 0xB54B, 0x8528, 0x9509, 0xE5EE, 0xF5CF, 0xC5AC, 0xD58D,
    0x3653, 0x2672, 0x1611, 0x630, 0x76D7, 0x66F6, 0x5695, 0x46B4, 0xB75B, 0xA77A, 0x9719, 0x8738,
    0xF7DF, 0xE7FE, 0xD79D, 0xC7BC, 0x48C4, 0x58E5, 0x6886, 0x78A7, 0x840, 0x1861, 0x2802, 0x3823,
    0xC9CC, 0xD9ED, 0xE98E, 0xF9AF, 0x8948, 0x9969, 0xA90A, 0xB92B, 0x5AF5, 0x4AD4, 0x7AB7, 0x6A96,
    0x1A71, 0xA50, 0x3A33, 0x2A12, 0xDBFD, 0xCBDC, 0xFBBF, 0xEB9E, 0x9B79, 0x8B58, 0xBB3B, 0xAB1A,
    0x6CA6, 0x7C87, 0x4CE4, 0x5CC5, 0x2C22, 0x3C03, 0xC60, 0x1C41, 0xEDAE, 0xFD8F, 0xCDEC, 0xDDCD,
    0xAD2A, 0xBD0B, 0x8D68, 0x9D49, 0x7E97, 0x6EB6, 0x5ED5, 0x4EF4, 0x3E13, 0x2E32, 0x1E51, 0xE70,
    0xFF9F, 0xEFBE, 0xDFDD, 0xCFFC, 0xBF1B, 0xAF3A, 0x9F59, 0x8F78, 0x9188, 0x81A9, 0xB1CA, 0xA1EB,
    0xD10C, 0xC12D, 0xF14E, 0xE16F, 0x1080, 0xA1, 0x30C2, 0x20E3, 0x5004, 0x4025, 0x7046, 0x6067,
    0x83B9, 0x9398, 0xA3FB, 0xB3DA, 0xC33D, 0xD31C, 0xE37F, 0xF35E, 0x2B1, 0x1290, 0x22F3, 0x32D2,
    0x4235, 0x5214, 0x6277, 0x7256, 0xB5EA, 0xA5CB, 0x95A8, 0x8589, 0xF56E, 0xE54F, 0xD52C, 0xC50D,
    0x34E2, 0x24C3, 0x14A0, 0x481, 0x7466, 0x6447, 0x5424, 0x4405, 0xA7DB, 0xB7FA, 0x8799, 0x97B8,
    0xE75F, 0xF77E, 0xC71D, 0xD73C, 0x26D3, 0x36F2, 0x691, 0x16B0, 0x6657, 0x7676, 0x4615, 0x5634,
    0xD94C, 0xC96D, 0xF90E, 0xE92F, 0x99C8, 0x89E9, 0xB98A, 0xA9AB, 0x5844, 0x4865, 0x7806, 0x6827,
    0x18C0, 0x8E1, 0x3882, 0x28A3, 0xCB7D, 0xDB5C, 0xEB3F, 0xFB1E, 0x8BF9, 0x9BD8, 0xABBB, 0xBB9A,
    0x4A75, 0x5A54, 0x6A37, 0x7A16, 0xAF1, 0x1AD0, 0x2AB3, 0x3A92, 0xFD2E, 0xED0F, 0xDD6C, 0xCD4D,
    0xBDAA, 0xAD8B, 0x9DE8, 0x8DC9, 0x7C26, 0x6C07, 0x5C64, 0x4C45, 0x3CA2, 0x2C83, 0x1CE0, 0xCC1,
    0xEF1F, 0xFF3E, 0xCF5D, 0xDF7C, 0xAF9B, 0xBFBA, 0x8FD9, 0x9FF8, 0x6E17, 0x7E36, 0x4E55, 0x5E74,
    0x2E93, 0x3EB2, 0xED1, 0x1EF0,
];

fn decrypt_buf(s: &mut [u8]) {
    // let mut ptr = s.as_mut_ptr() as *mut u32;
    // let fat_len = s.len() - 1 >> 2;

    // for _ in 0..fat_len {
    //     unsafe { *ptr = !*ptr };
    //     ptr = ptr.wrapping_add(1);
    // }
    // for i in fat_len * 4..s.len() {
    //     s[i] = !s[i];
    // }

    let mut i = 0;
    while i < s.len() - 4 {
        let c1 = s[i];
        let c2 = s[i + 1];
        let c3 = s[i + 2];
        let c4 = s[i + 3];
        let v = !u32::from_le_bytes([c1, c2, c3, c4]);
        let [c1, c2, c3, c4] = v.to_le_bytes();
        s[i] = c1;
        s[i + 1] = c2;
        s[i + 2] = c3;
        s[i + 3] = c4;
        i += 4;
    }
    while i < s.len() {
        s[i] = !s[i];
        i += 1;
    }
}

struct Entry {
    name: String,
    offset: usize,
    size: usize,
    is_encrypted: bool,
    _unknown: u32,
}

fn read_entries(data: &[u8]) -> Vec<Entry> {
    let mut c = Cursor::new(&data);

    // lines are separated by [0x0D, 0x0A]

    let mut header = vec![];
    for _ in 0..9 {
        c.read_until('\n' as u8, &mut header).unwrap();
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
        let unknown = c.read_u32::<LittleEndian>().unwrap();
        let mut encrypted_filename = vec![0; filename_length as usize];
        c.read_exact(&mut encrypted_filename).unwrap();
        decrypt_buf(encrypted_filename.as_mut());

        entries.push(Entry {
            name: String::from_utf8_lossy(
                encrypted_filename[0..(filename_length - 1) as usize].as_mut(),
            )
            .to_string(),
            offset: file_offset as usize + header_size as usize,
            size: file_size as usize,
            is_encrypted: is_text,
            _unknown: unknown,
        });
    }

    entries
}

/*
fn _encode_char(hash: &mut u16, ch: u8) {
    let index = (*hash >> 8) ^ ch as u16;
    let hash_lookup = HASH_LOOKUP_TABLE[index as usize];
    *hash = hash_lookup ^ *hash << 8;
}

fn _hash(path: &[u8]) -> u32 {
    let mut parts = [0xFFFFu16; 2];
    let mut i = 0;
    loop {
        let ch = {
            if path[i] >= 0x41 && path[i] <= 0x5A {
                path[i] + 0x20
            } else {
                path[i]
            }
        };
        encode_char(&mut parts[(i & 1) as usize], ch);
        i += 1;
        if i == path.len() {
            break;
        }
    }

    let temp = parts[0];
    parts[0] = parts[1];
    parts[1] = temp;

    let mut result = 0u32;
    for i in 0..2 {
        result |= (parts[i] as u32) << (i * 16);
    }

    result
}

fn decode_path(hash: u16) -> String {
    let mut hash = hash;
    let mut path = String::new();
    while hash != 0 {
        let ch = (hash & 0xFF) as u8;
        path.push(ch as char);
        hash >>= 8;
    }
    path
}
*/

struct GutFile {
    path: PathBuf,
    data: Vec<u8>,
}

impl GutFile {
    fn load(path: &Path) -> Result<Self, std::io::Error> {
        let mut s = Self {
            path: path.to_path_buf(),
            data: vec![],
        };
        s.ensure_data()?;
        Ok(s)
    }

    fn read_entries(&self) -> Vec<Entry> {
        read_entries(self.data.as_ref())
    }

    fn get_contents(&mut self, entry: &Entry) -> &mut [u8] {
        let data = &mut self.data[entry.offset..entry.offset + entry.size];
        if entry.is_encrypted {
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

fn main() {
    // let encoded = encode_path("AUDIO_NULL".as_bytes());

    // println!("encoded: {:08X}", encoded);
    // let decoded = decode_path(encoded);
    // println!("encoded: {:04X}, decoded: {}", encoded, decoded);

    let extract_path = PathBuf::from("C:\\Games\\shadow_company\\extracted");

    walkdir::WalkDir::new("C:\\Games\\shadow_company")
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().unwrap_or_default() == "gut")
        .for_each(|e| {
            let prefix = e
                .path()
                .strip_prefix("C:\\Games\\shadow_company")
                .unwrap()
                .parent()
                .unwrap();

            let mut gut_file = GutFile::load(e.path()).unwrap();
            let entries = gut_file.read_entries();

            // println!("[{}]", gut_file.path.display());
            for entry in entries {
                // println!("{}: {} ({} bytes)", entry.name, entry.offset, entry.size);

                /*
                let c = gut_file.get_contents(&entry);
                for ch in c.iter() {
                    print!("{}", *ch as char);
                }
                println!();
                */

                let full_path = extract_path.join(prefix).join(&entry.name);
                println!("{}: full_path: {}", entry.is_encrypted, full_path.display());
                std::fs::create_dir_all(full_path.parent().unwrap()).unwrap();
                std::fs::write(full_path, gut_file.get_contents(&entry)).unwrap();
            }
        });
}
