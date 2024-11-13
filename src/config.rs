use byteorder::ReadBytesExt;

use crate::io::Reader;

#[derive(Debug)]
pub enum EndType {
    None,
    StartKey(&'static str),
    EndKey(&'static str),
}

#[derive(Debug, thiserror::Error)]
pub enum ParseConfigError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid key for \"{0}\": \"{1}\"")]
    InvalidKey(String, String),
}

pub type ParseConfigResult = Result<(), ParseConfigError>;

pub trait Config
where
    Self: Default + From<&'static ConfigLine>,
{
    const HAS_CONFIG_CHILD_FIELDS: bool;

    fn parse_config_line<R>(&mut self, reader: &mut ConfigReader<R>) -> ParseConfigResult
    where
        R: Reader;

    fn parse_config_with_end<R>(
        &mut self,
        reader: &mut ConfigReader<R>,
        end_type: EndType,
    ) -> ParseConfigResult
    where
        R: Reader;

    fn from_config<R>(reader: &mut ConfigReader<R>) -> Result<Self, ParseConfigError>
    where
        R: Reader,
    {
        let mut obj = Self::default();
        obj.parse_config_with_end(reader, EndType::None)?;
        Ok(obj)
    }
}

#[derive(Debug)]
pub struct ConfigLine {
    pub name: String,
    pub params: Vec<String>,
}

pub trait FromParam
where
    Self: Sized,
{
    fn from(param: String) -> Option<Self>;
}

impl FromParam for String {
    fn from(param: String) -> Option<Self> {
        Some(param)
    }
}

impl FromParam for bool {
    fn from(param: String) -> Option<Self> {
        param.parse::<u32>().map(|v| v == 1).ok()
    }
}

macro_rules! impl_parse_param {
    ($t:ty) => {
        impl FromParam for $t {
            fn from(param: String) -> Option<Self> {
                param.parse().ok()
            }
        }
    };
}

impl_parse_param!(i32);
impl_parse_param!(u32);
impl_parse_param!(f32);

impl ConfigLine {
    pub fn param<T: FromParam>(&self, index: usize) -> Option<T> {
        match self.params.get(index) {
            Some(value) => T::from(value.clone()),
            _ => None,
        }
    }
}

fn read_line<R>(reader: &mut R) -> std::io::Result<String>
where
    R: Reader,
{
    let mut str = String::new();

    macro_rules! next_char {
        () => {
            match reader.read_u8() {
                Ok(ch) => ch,
                Err(err) => match err.kind() {
                    std::io::ErrorKind::UnexpectedEof if !str.is_empty() => return Ok(str),
                    _ => return Err(err),
                },
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

    reader.seek(std::io::SeekFrom::Current(-1))?;

    Ok(str)
}

pub fn read_config_line<R>(r: &mut R) -> std::io::Result<Option<ConfigLine>>
where
    R: Reader,
{
    let mut line;
    loop {
        line = match read_line(r) {
            Ok(line) => line,
            Err(err) if matches!(err.kind(), std::io::ErrorKind::UnexpectedEof) => {
                return Ok(None);
            }
            Err(err) => return Err(err),
        };

        if !line.is_empty() {
            line = line
                .trim_matches(|ch| ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r')
                .to_string();
        }

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

pub struct ConfigReader<T>
where
    T: Reader,
{
    reader: T,
    current: Option<ConfigLine>,
}

impl<R> ConfigReader<R>
where
    R: Reader,
{
    pub fn new(reader: R) -> Result<Self, std::io::Error> {
        let mut s = Self {
            reader,
            current: None,
        };
        s.next_line()?;
        Ok(s)
    }

    pub fn current(&self) -> Option<&ConfigLine> {
        self.current.as_ref()
    }

    pub fn next_line(&mut self) -> Result<(), std::io::Error> {
        self.current = read_config_line(&mut self.reader)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_reader() {
        let data = r#"
            NAME training
            SIZE 10
        "#;

        let reader = std::io::Cursor::new(data);
        let mut r = ConfigReader::new(reader).expect("failed to read first line");
        assert!(r.current().is_some());
        assert_eq!(r.current().unwrap().name, "NAME");

        r.next_line().expect("failed to read line");
        assert!(r.current().is_some());
        assert_eq!(r.current().unwrap().name, "SIZE");

        r.next_line().expect("failed to read next file");
        assert!(r.current().is_none());
    }
}
