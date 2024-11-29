use byteorder::{LittleEndian as LE, ReadBytesExt};
use glam::{Quat, Vec2, Vec3};

pub trait Reader: std::io::Read + std::io::Seek + Sized {
    fn read_vec2(&mut self) -> std::io::Result<Vec2> {
        let x = self.read_f32::<LE>()?;
        let y = self.read_f32::<LE>()?;
        Ok(Vec2::new(x, y))
    }

    fn read_vec3(&mut self) -> std::io::Result<Vec3> {
        let x = self.read_f32::<LE>()?;
        let y = self.read_f32::<LE>()?;
        let z = self.read_f32::<LE>()?;
        Ok(Vec3::new(x, y, z))
    }

    fn read_quat(&mut self) -> std::io::Result<Quat> {
        let x = self.read_f32::<LE>()?;
        let y = self.read_f32::<LE>()?;
        let z = self.read_f32::<LE>()?;
        let w = self.read_f32::<LE>()?;
        Ok(Quat::from_xyzw(x, y, z, w))
    }

    fn read_fixed_string(&mut self, len: usize) -> std::io::Result<String> {
        let mut result = String::with_capacity(len);
        let mut len = len - 1;
        loop {
            let ch = self.read_u8()?;
            if ch == 0 || len == 0 {
                break;
            }
            result.push(ch as char);
            len -= 1;
        }
        while len != 0 {
            self.read_u8()?;
            len -= 1;
        }
        Ok(result)
    }

    fn skip_sinister_header(&mut self) -> std::io::Result<()> {
        let mut ch = self.read_u8()?;
        loop {
            // If the line doesn't start with a `*` break out of the loop and assume we're reading
            // data from now on.
            if ch != 0x2A {
                break;
            }

            // Consume the rest of the line until we hit one of the line end characters.
            while ch != 0x0D && ch != 0x0A {
                ch = self.read_u8()?;
            }

            // Consume the newline characters.
            while ch == 0x0D || ch == 0x0A {
                ch = self.read_u8()?;
            }
        }

        // Reverse the last read character.
        self.seek(std::io::SeekFrom::Current(-1))?;

        Ok(())
    }

    /// Read bytes until the full sequence is matched and returns the amount of bytes read. An
    /// [std::io::ErrorKind::UnexpectedEof] is returned if the max_length is exceeded.
    fn skip_sinister_header_2(
        &mut self,
        sequence: &[u8],
        max_length: usize,
    ) -> std::io::Result<usize> {
        debug_assert!(!sequence.is_empty(), "sequence can't be empty");

        let mut bytes_read = 0;
        let mut match_index = 0;

        loop {
            let byte = self.read_u8()?;

            bytes_read += 1;
            if bytes_read >= max_length {
                return Err(std::io::ErrorKind::UnexpectedEof.into());
            }

            if byte == sequence[match_index] {
                match_index += 1;
                if match_index == sequence.len() {
                    return Ok(bytes_read);
                }
            } else {
                match_index = 0;
            }
        }
    }
}

impl<R: std::io::Read + std::io::Seek + Sized> Reader for R {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sinister_header() {
        // No match found within the max length.
        let mut c = std::io::Cursor::new(b"data123data");
        let result = c.skip_sinister_header_2(b"456", 10);
        assert!(result.is_err());

        // No match found, max length exceeded.
        let mut c = std::io::Cursor::new(b"data123data");
        let result = c.skip_sinister_header_2(b"123", 5);
        assert!(result.is_err());

        // Matches the first one.
        let mut c = std::io::Cursor::new(b"123data123data");
        let length = c.skip_sinister_header_2(b"123", 10).unwrap();
        assert_eq!(length, 3);

        // Matches at the end.
        let mut c = std::io::Cursor::new(b"data123");
        let length = c.skip_sinister_header_2(b"123", 10).unwrap();
        assert_eq!(length, 7);

        // Matches with repeated sequence.
        let mut c = std::io::Cursor::new(b"123123data");
        let length = c.skip_sinister_header_2(b"123", 10).unwrap();
        assert_eq!(length, 3);

        // Matches with partial sequence at the end.
        let mut c = std::io::Cursor::new(b"data12");
        let result = c.skip_sinister_header_2(b"123", 10);
        assert!(result.is_err());

        // Matches with sequence at the start.
        let mut c = std::io::Cursor::new(b"123data");
        let length = c.skip_sinister_header_2(b"123", 10).unwrap();
        assert_eq!(length, 3);

        // Matches with sequence in the middle.
        let mut c = std::io::Cursor::new(b"data123data");
        let length = c.skip_sinister_header_2(b"123", 10).unwrap();
        assert_eq!(length, 7);
    }
}
