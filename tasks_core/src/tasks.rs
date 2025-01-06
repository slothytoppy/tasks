use crate::iterator::*;
use std::{fmt::Display, path::PathBuf};

#[derive(Debug, PartialEq, Eq)]
pub enum TaskError {
    NoFile(String),
    NoData,
    ParseError(String),
    Eof,
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
            TaskError::Eof => write!(f, "Reached EOF/End of data"),
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

    pub fn parse(
        &mut self,
        source: String,
        mut offset: usize,
    ) -> Result<(TaskItem, usize), TaskError> {
        if source.is_empty() {
            return Err(TaskError::NoData);
        }
        let mut state = ParserState::Name;

        let mut tokenized_input = String::default();

        let mut repeate_space = false;

        source.chars().for_each(|ch| match ch {
            '!'..='~' => {
                tokenized_input.push(ch);
            }
            ' ' => {
                if !repeate_space {
                    repeate_space = true;
                    tokenized_input.push(ch);
                } else {
                    repeate_space = false;
                }
            }
            _ => {}
        });

        let mut item = TaskItem::default();

        for (i, ch) in tokenized_input.chars().enumerate() {
            if state != ParserState::Data && ch == ' ' {
                offset += 1;
                continue;
            }
            if offset >= tokenized_input.len().saturating_sub(1) {
                return Err(TaskError::Eof);
            }
            tracing::info!("offset: {}", offset);
            match state {
                ParserState::Name => {
                    let (name, off) = item.parse_name(&tokenized_input[offset..])?;
                    tracing::info!("name: {}", name.name);
                    item.set_name(name.name);
                    offset += off;
                    state = ParserState::Status;
                }

                ParserState::Status => {
                    let (status, off) = self.parse_status(&tokenized_input[offset..])?;
                    tracing::info!("status: {}", status.status);
                    item.set_status(status.status);
                    offset += off;
                    state = ParserState::Data;
                }
                ParserState::Data => {
                    let (data, off) = self.parse_data(&tokenized_input[offset..])?;
                    offset += off;
                    item.set_data(data.data);
                    return Ok((item, offset + i));

                    //let (data, off) = self.parse_data(&tokenized_input[offset..])?;
                    //tracing::info!("data: {}", data.data);
                }
            }
        }
        Err(TaskError::Eof)
    }

    fn parse_name(&self, source: &str) -> Result<(TaskName, usize), TaskError> {
        tracing::info!("name source: {source}");
        let (mut found_start, mut start_idx) = (false, 0);
        for (i, ch) in source.chars().enumerate() {
            if i == source.len().saturating_sub(1) {
                return Err(TaskError::Eof);
            }
            match ch {
                '[' => {
                    found_start = true;
                    start_idx = i;
                }
                ']' => {
                    if found_start {
                        return Ok((
                            TaskName::new(source[start_idx + 1..i].to_string()),
                            i + start_idx + 1,
                        ));
                    }
                    break;
                }
                _ => {}
            }
        }
        Err(TaskError::ParseError(format!(
            "failed to find name in {source}"
        )))
    }

    fn parse_status(&self, source: &str) -> Result<(TaskStatus, usize), TaskError> {
        tracing::info!("status source: {source}");
        enum StatusState {
            Status,
            Equal,
            Bool,
        }

        let mut state = StatusState::Status;

        let mut buffer: &str;

        let mut skip_amount = "status".len();

        for (i, ch) in source.chars().enumerate() {
            if ch == ' ' {
                continue;
            }
            if i + skip_amount >= source.len().saturating_sub(1) {
                return Err(TaskError::Eof);
            }
            buffer = &source[i..i + skip_amount];
            match state {
                StatusState::Status => {
                    if buffer == "status" {
                        state = StatusState::Equal;
                        skip_amount = "=".len();
                    }
                }
                StatusState::Equal => {
                    if ch == '=' {
                        state = StatusState::Bool;
                        skip_amount = "true".len();
                    }
                }
                StatusState::Bool => {
                    if buffer == "true" {
                        //tracing::info!("found true");
                        return Ok((TaskStatus::new(true), i + "true".len().saturating_sub(1)));
                    } else if &source[i..i + "false".len()] == "false" {
                        //tracing::info!("found false");
                        return Ok((TaskStatus::new(false), i + "false".len().saturating_sub(1)));
                    } else {
                        return Err(TaskError::ParseError(
                            "Syntax Error: expected 'true' or 'false'".to_string(),
                        ));
                    }
                }
            }
        }
        Err(TaskError::ParseError("failed to find status".to_string()))
    }

    fn parse_data(&self, source: &str) -> Result<(TaskData, usize), TaskError> {
        tracing::info!("data source {source}");
        enum DataState {
            Data,
            Equal,
            Content,
        }

        let mut state = DataState::Data;

        let mut buffer: &str;

        let mut skip_amount = "data".len();

        let mut found_quote = false;

        let mut start = 0;

        for (i, ch) in source.chars().enumerate() {
            if i >= source.len() {
                return Err(TaskError::Eof);
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
                    if ch == '\"' {
                        match found_quote {
                            false => {
                                start = i + 1;
                                found_quote = true;
                            }
                            true => {
                                tracing::info!("successfully parsed {}", &source[start..i]);
                                return Ok((
                                    TaskData::new(source[start..i].to_string()),
                                    i.saturating_sub(1),
                                ));
                            }
                        }
                    }
                }
            }
        }

        Err(TaskError::ParseError("failed to parse data".to_string()))
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

    fn parse(&mut self, content: String) -> Result<TaskList, TaskError> {
        let mut offset = usize::default();
        let mut list = TaskList::default();

        loop {
            match TaskItem::default().parse(content.clone(), offset) {
                Ok((item, off)) => {
                    offset += off;
                    tracing::info!("{item}");
                    list.push(item);
                }
                Err(error) => match error {
                    TaskError::Eof => return Ok(list),
                    _ => Err(error),
                }?,
            }
        }
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
status = false 
data = "bai"
[2]
status = true
data = "urmom"
[3]
status = true
data = "hai""#;

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
