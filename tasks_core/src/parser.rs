use crate::tasks::*;

#[derive(Debug, Default)]
pub struct Parser {
    source: String,
}

#[derive(Debug, PartialEq, Eq)]
enum ParserState {
    Name,
    Status,
    Data,
}

impl Parser {
    pub fn new(source: String) -> Self {
        Self { source }
    }

    pub fn parse(&mut self) -> Result<TaskList, TaskError> {
        if self.source.is_empty() {
            return Err(TaskError::NoData);
        }

        let mut name: &str = "";
        let mut data: &str;
        let mut status: bool = false;
        let mut offset = usize::default();
        let mut list = TaskList::default();

        let mut state = ParserState::Name;

        for (i, ch) in self.source.bytes().enumerate() {
            if ch != b'\0' {
                continue;
            }
            match state {
                ParserState::Name => {
                    name = self.parse_name(offset)?;
                    state = ParserState::Status;
                    // i + 1 because we need to set the offset so the next parsing call starts
                    // at the next char
                    offset = i + 1;
                }
                ParserState::Status => {
                    status = self.parse_status(offset)?;
                    state = ParserState::Data;
                    // offset += 2 because i is a '\0',
                    offset += 2;
                }
                ParserState::Data => {
                    data = self.parse_data(offset)?;
                    state = ParserState::Name;
                    // i + 1 because we need to set the offset so the next parsing call starts
                    // at the next char
                    //offset = i + 1;
                    offset += data.len() + 1;
                    list.push(TaskItem::new(name.to_string(), data.to_string(), status))
                }
            }
        }
        Ok(list)
    }

    fn parse_name(&self, offset: usize) -> Result<&str, TaskError> {
        for (i, ch) in self.source.bytes().skip(offset).enumerate() {
            if ch == b'\0' {
                return Ok(&self.source[offset..offset + i]);
            }
        }
        println!("parse_header no data");
        Err(TaskError::NoData)
    }

    fn parse_status(&self, offset: usize) -> Result<bool, TaskError> {
        let mut status = false;

        for ch in self.source.bytes().skip(offset) {
            match ch {
                b'0' => status = false,
                b'1' => status = true,
                b'\0' => return Ok(status),
                _ => {
                    return Err(TaskError::ParseError(format!(
                        "expected '0' or '1', found {ch}"
                    )));
                }
            };
        }
        println!("parse_status no data");
        Err(TaskError::NoData)
    }

    fn parse_data(&self, offset: usize) -> Result<&str, TaskError> {
        for (i, ch) in self.source.bytes().skip(offset).enumerate() {
            if ch == b'\0' {
                return Ok(&self.source[offset..offset + i]);
            }
        }
        Ok(&self.source[offset..])
    }
}
