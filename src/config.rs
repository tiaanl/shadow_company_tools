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

    let mut ch = r.read_u8()?;
    while ch != 0x0A && ch != 0x0D {
        str.push(ch as char);
        ch = r.read_u8()?;
    }

    // Consume the newline characters.
    while ch == 0x0A || ch == 0x0D {
        ch = r.read_u8()?;
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

        if line.is_empty() || line.starts_with(";") {
            line.clear();
            continue;
        }
        break;
    }

    let mut parts: Vec<String> = line
        .split('\t')
        .filter_map(|t| {
            if !t.is_empty() {
                Some(t.to_string())
            } else {
                None
            }
        })
        .collect();

    Ok(Some(ConfigLine {
        name: parts.remove(0),
        params: parts,
    }))
}
