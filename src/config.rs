use byteorder::ReadBytesExt;

#[derive(Debug)]
pub struct ConfigLine {
    pub name: String,
    pub params: Vec<String>,
}

fn read_line<R>(r: &mut R) -> std::io::Result<String>
where
    R: std::io::Read + std::io::Seek,
{
    let mut str = String::new();

    macro_rules! next_char {
        () => {
            match r.read_u8() {
                Ok(ch) => ch,
                Err(err) => {
                    if let std::io::ErrorKind::UnexpectedEof = err.kind() {
                        return Ok("".to_string());
                    }
                    return Err(err);
                }
            }
        };
    }

    let mut ch = next_char!();
    while ch != 0x0A && ch != 0x0D {
        str.push(ch as char);
        ch = next_char!();
    }

    // Consume the newline characters.
    while ch == 0x0A || ch == 0x0D {
        ch = next_char!();
    }

    r.seek(std::io::SeekFrom::Current(-1))?;

    Ok(str)
}

pub fn read_config_line<R>(r: &mut R) -> std::io::Result<Option<ConfigLine>>
where
    R: std::io::Read + std::io::Seek,
{
    let mut line;
    loop {
        line = read_line(r)?;

        if line.is_empty() {
            return Ok(None);
        }
        line = line
            .trim_matches(|ch| ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r')
            .to_string();

        if line.is_empty() || line.starts_with(';') {
            line.clear();
            continue;
        }
        break;
    }

    let mut parts = Vec::new();
    let mut in_quote = false;
    let mut current_part = String::new();

    for c in line.chars() {
        match c {
            '"' => {
                if in_quote {
                    parts.push(current_part.clone());
                    current_part.clear();
                }
                in_quote = !in_quote;
            }
            _ if in_quote => {
                current_part.push(c);
            }
            _ if c.is_whitespace() => {
                if !current_part.is_empty() {
                    parts.push(current_part.clone());
                    current_part.clear();
                }
            }
            _ => {
                current_part.push(c);
            }
        }
    }
    if !current_part.is_empty() {
        parts.push(current_part);
    }

    if parts.is_empty() {
        return Ok(None);
    }

    Ok(Some(ConfigLine {
        name: parts.remove(0),
        params: parts,
    }))
}
