use crate::iterator::*;
use std::{fmt::Display, ops::Index, path::PathBuf};

#[derive(Debug, PartialEq, Eq)]
pub enum TaskError {
    NoFile(String),
    NoData,
    ParseError(String),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParserState {
    Name,
    Status,
    Data,
}

impl Display for TaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskError::ParseError(str) => {
                write!(f, "Failed to parse: {}", str)
            }
            TaskError::NoFile(file) => write!(f, "ENOFILE: {file}"),
            TaskError::NoData => write!(f, "No data to parse"),
        }
    }
}

#[derive(Default, Debug, Clone)]
struct TaskName {
    name: String,
}

impl TaskName {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

#[derive(Default, Debug, Clone)]
struct TaskData {
    data: String,
}

impl TaskData {
    pub fn new(data: String) -> Self {
        Self { data }
    }
}

#[derive(Default, Debug, Clone)]
struct TaskStatus {
    status: bool,
}

impl Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status = match self.status {
            true => "true",
            false => "false",
        };

        f.write_str(status)
    }
}

impl TaskStatus {
    pub fn new(status: bool) -> Self {
        Self { status }
    }
}

impl std::fmt::Binary for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buff = String::default();
        match self.status {
            true => buff.push('1'),
            false => buff.push('0'),
        };

        buff.push('\0');

        f.write_str(&buff)
    }
}

#[derive(Debug, Default, Clone)]
pub struct TaskItem {
    name: TaskName,
    status: TaskStatus,
    data: TaskData,
}

impl Display for TaskItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "[{}]\n{}\n{}\n",
            self.name(),
            self.status,
            self.data()
        ))
    }
}

impl TaskItem {
    pub fn new(name: String, data: String, status: bool) -> Self {
        Self {
            name: TaskName::new(name),
            data: TaskData::new(data),
            status: TaskStatus::new(status),
        }
    }

    pub fn name(&self) -> &str {
        &self.name.name
    }

    pub fn data(&self) -> &str {
        &self.data.data
    }

    pub fn status(&self) -> bool {
        self.status.status
    }

    pub fn set_name(&mut self, name: String) {
        self.name = TaskName::new(name)
    }

    pub fn set_data(&mut self, data: String) {
        self.data = TaskData::new(data)
    }

    pub fn set_status(&mut self, status: bool) {
        self.status = TaskStatus::new(status)
    }

    fn skip_until(&self, start: usize, source: &str) -> Option<usize> {
        for (i, ch) in source.chars().skip(start).enumerate() {
            if let 'a'..='z' = ch {
                return Some(i);
            }
        }
        None
    }

    fn parse_name(&self, source: &str) -> Result<(TaskName, usize), TaskError> {
        let mut start = 0;
        for (i, ch) in source.chars().enumerate() {
            match ch {
                '[' => start = i + 1,
                ']' => {
                    return Ok((TaskName::new(source[start..i].to_string()), i + 1));
                }
                _ => {}
            }
        }
        Err(TaskError::ParseError("failed to find name".to_string()))
    }

    fn parse_status(&self, source: &str) -> Result<(TaskStatus, usize), TaskError> {
        enum StatusState {
            Status,
            Equal,
            Bool,
        }

        let mut state = StatusState::Status;

        let mut buffer: &str;

        let mut skip_amount = "status".len();

        tracing::info!("source: {source}");

        for (i, ch) in source.chars().skip(skip_amount).enumerate() {
            buffer = &source[i..i + skip_amount];
            tracing::info!("buffer {buffer}");
            match state {
                StatusState::Status => {
                    if buffer == "status" {
                        state = StatusState::Equal;
                        skip_amount = "=".len();
                    }
                }
                StatusState::Equal => {
                    if buffer == "=" {
                        tracing::info!("found `=`");
                        tracing::info!("new buffer = {buffer}");
                        state = StatusState::Bool;
                        skip_amount = "true".len();
                    }
                }
                StatusState::Bool => {
                    if buffer == "true" {
                        return Ok((TaskStatus::new(true), i + skip_amount + 1));
                    } else if &source[i..i + "false".len()] == "false" {
                        return Ok((TaskStatus::new(false), i + skip_amount + 1));
                    }
                    tracing::info!("in bool state: {ch}");
                }
            }
        }
        Err(TaskError::ParseError("failed to find status".to_string()))
    }

    fn parse_data(&self, source: &str) -> Result<TaskData, TaskError> {
        enum DataState {
            Data,
            Equal,
            Content,
        }

        let mut state = DataState::Data;

        let mut buffer = "";

        let mut skip_amount = "data".len();

        let mut found_quote = false;

        let mut start = 0;

        for (i, ch) in source.chars().enumerate() {
            if i + skip_amount == source.len() + 1 {
                break;
            }
            buffer = &source[i..i + skip_amount];
            match state {
                DataState::Data => {
                    if buffer == "data" {
                        state = DataState::Equal;
                        skip_amount = "=".len();
                    }
                }
                DataState::Equal => {
                    if buffer == "=" {
                        state = DataState::Content;
                        skip_amount = 1;
                    }
                }
                DataState::Content => {
                    if buffer == "\"" {
                        tracing::info!("content buffer {buffer}");
                        tracing::info!("ch {ch}");
                        match found_quote {
                            false => {
                                start = i + 1;
                                found_quote = true;
                            }
                            true => {
                                tracing::info!("found quote");
                                tracing::info!("content buffer {buffer} start {start} i {i}");
                                return Ok(TaskData::new(source[start..i].to_string()));
                            }
                        }
                    }
                }
            }
        }

        Err(TaskError::ParseError("failed to parse data".to_string()))
    }

    pub fn parse(&mut self, source: String) -> Result<TaskItem, TaskError> {
        let mut state = ParserState::Name;

        let mut tokenized_input = String::default();

        source.chars().for_each(|ch| {
            match ch {
                '!'..='~' => tokenized_input.push(ch),
                _ => {}
            };
        });

        let mut offset = usize::default();

        let mut item = TaskItem::default();

        let (mut start, mut end) = (0, 0);

        for (i, ch) in tokenized_input.chars().enumerate() {
            match state {
                ParserState::Name => {
                    let (name, off) = item.parse_name(&tokenized_input).unwrap();
                    item.set_name(name.name);
                    state = ParserState::Status;
                    offset += off;
                }

                ParserState::Status => {
                    let (status, off) = self.parse_status(&tokenized_input[offset..]).unwrap();
                    item.set_status(status.status);
                    state = ParserState::Data;
                    offset += off;
                    //if let 'a'..='z' = ch {
                    //    start = i;
                    //}
                    //if ch == ' ' {
                    //    end = i - 1;
                    //    //if source[start:end]=="status" {
                    //    //
                    //    //    ParserState::Data;
                    //    //}
                    //}
                }
                ParserState::Data => {
                    let data = self.parse_data(&tokenized_input[offset..]).unwrap();
                    item.set_data(data.data);
                    tracing::info!("parsed: {item:?}");
                    return Ok(item);
                }
            }
        }

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
                            tracing::info!("data {item}");
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

#[derive(Default, Debug, Clone)]
pub struct TaskList {
    pub source: String,
    pub list: Vec<TaskItem>,
}

impl TaskList {
    pub fn new() -> Self {
        TaskList::default()
    }

    pub fn iter(&self) -> TaskIter<'_> {
        TaskIter { list: self, idx: 0 }
    }

    pub fn serialize(&self, file: PathBuf) {
        let mut buff = String::default();
        self.clone().iter().for_each(|item| {
            buff.push_str(&format!(
                "{}\n{:b}{}\n",
                item.name(),
                item.status,
                item.data()
            ));
        });
        if let Err(e) = std::fs::write(file, buff) {
            panic!("{e}");
        }
    }

    pub fn deserialize(source: String) -> Result<Self, TaskError> {
        TaskList::default().parse(source)
    }

    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    pub fn get(&self, idx: usize) -> Option<&TaskItem> {
        self.list.get(idx)
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut TaskItem> {
        self.list.get_mut(idx)
    }

    pub fn push(&mut self, item: TaskItem) {
        self.list.push(item)
    }

    pub fn set(&mut self, idx: usize, task: TaskItem) {
        self.list.insert(idx, task)
    }

    pub fn remove(&mut self, idx: usize) {
        self.list.remove(idx);
    }

    pub fn parse_name(&self, offset: usize) -> Result<&str, TaskError> {
        for (i, ch) in self.source.bytes().skip(offset).enumerate() {
            if ch == b'\0' {
                return Ok(&self.source[offset..offset + i]);
            }
        }
        println!("parse_header no data");
        Err(TaskError::NoData)
    }

    pub fn parse_status(&self, offset: usize) -> Result<bool, TaskError> {
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

    pub fn parse_data(&self, offset: usize) -> Result<&str, TaskError> {
        for (i, ch) in self.source.bytes().skip(offset).enumerate() {
            if ch == b'\0' {
                return Ok(&self.source[offset..offset + i]);
            }
        }
        Ok(&self.source[offset..])
    }

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

impl Display for TaskList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for item in self.iter() {
            let _ = write!(f, "{item}");
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::TaskList;

    #[test]
    pub fn test_list() {
        let data = r#"[1]
0
bai
[2]
1
urmom
[3]
1
hai"#;

        let list = match TaskList::deserialize(data.to_string()) {
            Ok(list) => {
                println!("{list:?}");
                list
            }
            Err(e) => panic!("{e}"),
        };

        assert!(list.list.len() == 3);

        panic!();
    }
}
