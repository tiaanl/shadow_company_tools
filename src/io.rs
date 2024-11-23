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

    fn skip_sinister_header(&mut self) -> std::io::Result<u64> {
        let header_start = self.stream_position()?;

        let mut ch = self.read_u8()?;
        let mut buf = vec![];
        loop {
            // Check the first character of the line.
            if ch != 0x2A {
                break;
            }

            // Consume the rest of the line.
            while ch != 0x0A && ch != 0x0D {
                buf.push(ch);
                ch = self.read_u8()?;
            }

            // Consume the newline characters.
            while ch == 0x0A || ch == 0x0D {
                buf.push(ch);
                ch = self.read_u8()?;
            }
        }

        // Read the ID string.
        // TODO: What is this really??!!
        // 1A FA 31 C1 | DE ED 42 13
        let _ = self.read_u32::<LE>()?;
        let _ = self.read_u32::<LE>()?;

        // We read into the data by 1 character, so reverse it.
        let header_end = self.seek(std::io::SeekFrom::Current(-1))?;

        Ok(header_end - header_start)
    }
}

impl<R: std::io::Read + std::io::Seek + Sized> Reader for R {}
