use crate::tasks::*;

#[derive(Debug, PartialEq, Eq)]
pub enum ParserState {
    Name,
    Status,
    Data,
}

pub trait Parser<T, E> {
    fn parse(&mut self, content: String) -> Result<T, E>;
}

impl Parser<TaskItem, TaskError> for TaskItem {
    fn parse(&mut self, source: String) -> Result<TaskItem, TaskError> {
        let mut state = ParserState::Name;

        let mut offset = usize::default();

        let mut item = TaskItem::default();

        for (i, ch) in source.bytes().enumerate() {
            if ch != b'\0' {
                continue;
            }
            match state {
                ParserState::Name => {
                    if ch == b'\0' {
                        item.set_name(source[offset..offset + i].to_string());
                        state = ParserState::Status;
                        offset = i + 1;
                        // i + 1 because we need to set the offset so the next parsing call starts
                        // at the next char
                        continue;
                    }
                    println!("parse_header no data");
                    panic!("header: NoData")
                }
                ParserState::Status => {
                    // offset += 2 because i is a '\0',
                    match ch {
                        b'0' => {
                            state = ParserState::Data;
                            item.set_status(false);
                            offset += 2;
                            continue;
                        }
                        b'1' => {
                            state = ParserState::Data;
                            offset += 2;
                            item.set_status(true);
                            continue;
                        }
                        b'\0' => continue,
                        _ => {
                            panic!("status: NoData")
                        }
                    };
                }
                ParserState::Data => {
                    for (i, ch) in source.bytes().skip(i).enumerate() {
                        if ch == b'\0' {
                            item.set_data(source[offset..offset + i].to_string());
                            tracing::info!("{item}");
                            return Ok(item);
                        }
                    }

                    item.set_data(source[offset..].to_string());

                    // i + 1 because we need to set the offset so the next parsing call starts
                    // at the next char
                    break;
                }
            }
        }
        tracing::info!("{item}");
        Ok(item)
    }
}

impl Parser<TaskList, TaskError> for TaskList {
    fn parse(&mut self, content: String) -> Result<TaskList, TaskError> {
        self.source = content;
        if self.source.is_empty() {
            return Err(TaskError::ParseError("hai".to_string()));
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
}

//impl Parser {
//    pub fn new(source: String) -> Self {
//        Self { source }
//    }
//
//    pub fn parse(&mut self) -> Result<TaskList, TaskError> {
//        if self.source.is_empty() {
//            return Err(TaskError::NoData);
//        }
//
//        let mut name: &str = "";
//        let mut data: &str;
//        let mut status: bool = false;
//        let mut offset = usize::default();
//        let mut list = TaskList::default();
//
//        let mut state = ParserState::Name;
//
//        for (i, ch) in self.source.bytes().enumerate() {
//            if ch != b'\0' {
//                continue;
//            }
//            match state {
//                ParserState::Name => {
//                    name = self.parse_name(offset)?;
//                    state = ParserState::Status;
//                    // i + 1 because we need to set the offset so the next parsing call starts
//                    // at the next char
//                    offset = i + 1;
//                }
//                ParserState::Status => {
//                    status = self.parse_status(offset)?;
//                    state = ParserState::Data;
//                    // offset += 2 because i is a '\0',
//                    offset += 2;
//                }
//                ParserState::Data => {
//                    data = self.parse_data(offset)?;
//                    state = ParserState::Name;
//                    // i + 1 because we need to set the offset so the next parsing call starts
//                    // at the next char
//                    //offset = i + 1;
//                    offset += data.len() + 1;
//                    list.push(TaskItem::new(name.to_string(), data.to_string(), status))
//                }
//            }
//        }
//        Ok(list)
//    }
//
//    fn parse_name(&self, offset: usize) -> Result<&str, TaskError> {
//        for (i, ch) in self.source.bytes().skip(offset).enumerate() {
//            if ch == b'\0' {
//                return Ok(&self.source[offset..offset + i]);
//            }
//        }
//        println!("parse_header no data");
//        Err(TaskError::NoData)
//    }
//
//    fn parse_status(&self, offset: usize) -> Result<bool, TaskError> {
//        let mut status = false;
//
//        for ch in self.source.bytes().skip(offset) {
//            match ch {
//                b'0' => status = false,
//                b'1' => status = true,
//                b'\0' => return Ok(status),
//                _ => {
//                    return Err(TaskError::ParseError(format!(
//                        "expected '0' or '1', found {ch}"
//                    )));
//                }
//            };
//        }
//        println!("parse_status no data");
//        Err(TaskError::NoData)
//    }
//
//    fn parse_data(&self, offset: usize) -> Result<&str, TaskError> {
//        for (i, ch) in self.source.bytes().skip(offset).enumerate() {
//            if ch == b'\0' {
//                return Ok(&self.source[offset..offset + i]);
//            }
//        }
//        Ok(&self.source[offset..])
//    }
//}
