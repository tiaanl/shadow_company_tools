use std::io::SeekFrom;

use byteorder::{LittleEndian, ReadBytesExt};

#[derive(Clone, Copy, Debug)]
pub struct Vector {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector {
    pub fn read<R>(r: &mut R) -> std::io::Result<Self>
    where
        R: std::io::Read,
    {
        let x = r.read_f32::<LittleEndian>()?;
        let y = r.read_f32::<LittleEndian>()?;
        let z = r.read_f32::<LittleEndian>()?;

        Ok(Self { x, y, z })
    }
}

impl std::fmt::Display for Vector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl std::fmt::Display for Quaternion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {}, {})", self.x, self.y, self.z, self.w)
    }
}

impl Quaternion {
    pub fn read<R>(r: &mut R) -> std::io::Result<Self>
    where
        R: std::io::Read,
    {
        let w = r.read_f32::<LittleEndian>()?;
        let x = r.read_f32::<LittleEndian>()?;
        let y = r.read_f32::<LittleEndian>()?;
        let z = r.read_f32::<LittleEndian>()?;

        Ok(Self { x, y, z, w })
    }
}

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

fn encode_char(hash: &mut u16, ch: u8) {
    let index = (*hash >> 8) ^ ch as u16;
    let hash_lookup = HASH_LOOKUP_TABLE[index as usize];
    *hash = hash_lookup ^ *hash << 8;
}

pub fn hash(path: &[u8]) -> u32 {
    let mut parts = [0xFFFFu16; 2];
    let mut i = 0;
    loop {
        if i >= path.len() {
            break;
        }
        let ch = {
            if path[i] >= 0x41 && path[i] <= 0x5A {
                path[i] + 0x20
            } else {
                path[i]
            }
        };
        if ch == 0 {
            break;
        }

        encode_char(&mut parts[i & 1], ch);
        i += 1;
    }

    (parts[0] as u32) << 16 | parts[1] as u32
}

pub fn decrypt_buf(s: &mut [u8]) {
    s.iter_mut().for_each(|c| *c = !*c);
}

pub fn print_buf(c: &mut impl std::io::Read, len: usize) {
    for i in 0..len {
        let ch = c.read_u8().unwrap();
        println!("({:04}) [{:02X}] {}", i, ch, ch as char);
    }
}

pub fn read_fixed_string(c: &mut impl std::io::Read, len: usize) -> String {
    let mut s = String::with_capacity(len);
    let mut len = len - 1;
    loop {
        let ch = c.read_u8().unwrap();
        if ch == 0 {
            break;
        }
        s.push(ch as char);
        len -= 1;
    }
    while len != 0 {
        c.read_u8().unwrap();
        len -= 1;
    }
    s
}

pub fn skip_sinister_header<R>(r: &mut R) -> std::io::Result<u64>
where
    R: std::io::Read + std::io::Seek,
{
    let header_start = r.stream_position()?;

    let mut ch = r.read_u8()?;
    let mut buf = vec![];
    loop {
        // Check the first character of the line.
        if ch != 0x2A {
            break;
        }

        // Consume the rest of the line.
        while ch != 0x0A && ch != 0x0D {
            buf.push(ch);
            ch = r.read_u8()?;
        }

        // Consume the newline characters.
        while ch == 0x0A || ch == 0x0D {
            buf.push(ch);
            ch = r.read_u8()?;
        }
    }

    // Read the ID string.
    // TODO: What is this really??!!
    // 1A FA 31 C1 | DE ED 42 13
    let _ = r.read_u32::<LittleEndian>()?;
    let _ = r.read_u32::<LittleEndian>()?;

    // We read into the data by 1 character, so reverse it.
    let header_end = r.seek(SeekFrom::Current(-1))?;

    Ok(header_end - header_start)
}
