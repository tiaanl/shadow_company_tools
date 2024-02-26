#[derive(Debug)]
pub struct ConfigLine {
    pub name: String,
    pub params: Vec<String>,
}

pub fn read_config_line(cur: &mut impl std::io::BufRead) -> std::io::Result<Option<ConfigLine>> {
    let mut line = String::new();
    loop {
        if cur.read_line(&mut line)? == 0 {
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
