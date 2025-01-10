use crate::iterator::*;
use std::{fmt::Display, path::PathBuf};

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

#[derive(Debug, PartialEq, Eq)]
pub enum ParserOffset {
    Offset(usize),
    Eof,
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

#[derive(Debug, Default, Clone)]
pub struct TaskItem {
    name: String,
    status: bool,
    data: String,
}

impl Display for TaskItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "[{}]\n{}\n{}\n",
            self.name(),
            self.status(),
            self.data()
        ))
    }
}

impl TaskItem {
    pub fn new(name: String, data: String, status: bool) -> Self {
        Self { name, data, status }
    }

    pub fn parse(source: &str, mut offset: usize) -> Result<(TaskItem, ParserOffset), TaskError> {
        let mut state = ParserState::Name;

        let mut item = TaskItem::default();

        for ch in source.chars() {
            if state != ParserState::Data && ch == ' ' {
                offset += 1;
            }
            tracing::info!("offset: {}", offset);
            match state {
                ParserState::Name => {
                    let (name, off) = TaskItem::parse_name(&source[offset..])?;
                    offset += off;
                    item.set_name(name.name);
                    tracing::info!("name offset: {offset}");
                    state = ParserState::Status;
                }

                ParserState::Status => {
                    let (status, off) = TaskItem::parse_status(&source[offset..])?;
                    offset += off;
                    item.set_status(status.status);
                    tracing::info!("status offset: {offset}");
                    state = ParserState::Data;
                }

                ParserState::Data => {
                    let (data, off) = TaskItem::parse_data(&source[offset..])?;
                    offset += off;
                    tracing::info!("data: {}", data.data);
                    tracing::info!("returned offset: {}", offset);
                    item.set_data(data.data);
                    if offset >= source.len().saturating_sub(1) {
                        return Ok((item, ParserOffset::Eof));
                    }
                    return Ok((item, ParserOffset::Offset(offset)));
                }
            }
        }
        Err(TaskError::NoData)
    }

    fn parse_name(source: &str) -> Result<(TaskName, usize), TaskError> {
        tracing::info!("name source: {source}");
        let (mut found_start, mut start_idx) = (false, 0);
        for (i, ch) in source.chars().enumerate() {
            if ch == '\n' {
                continue;
            }
            match ch {
                '[' => {
                    found_start = true;
                    start_idx = i;
                }
                ']' => {
                    if found_start {
                        tracing::info!("name: {}", &source[start_idx + 1..i]);
                        return Ok((
                            TaskName::new(source[start_idx + 1..i].to_string()),
                            i + start_idx + 1,
                        ));
                    }
                    break;
                }
                _ => {
                    if !found_start {
                        return Err(TaskError::ParseError(format!(
                            "expected `[` or `]`, found {ch}"
                        )));
                    }
                }
            }
        }
        Err(TaskError::ParseError(format!(
            "failed to find name in {source}"
        )))
    }

    fn parse_status(source: &str) -> Result<(TaskStatus, usize), TaskError> {
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
                        tracing::info!("status: true");
                        return Ok((TaskStatus::new(true), i + "true".len()));
                    } else if &source[i..i + "false".len()] == "false" {
                        tracing::info!("status: false");
                        return Ok((TaskStatus::new(false), i + "false".len()));
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

    fn parse_data(source: &str) -> Result<(TaskData, usize), TaskError> {
        tracing::info!("data source {source}");
        #[derive(PartialEq)]
        enum DataState {
            Data,
            Equal,
            Content,
        }

        let mut state = DataState::Data;

        let mut buffer: &str;
        let mut data_buffer: Vec<char> = vec![];

        let mut skip_amount = "data".len();

        let mut found_quote = false;

        for (i, ch) in source.chars().enumerate() {
            if state != DataState::Content && ch == ' ' {
                continue;
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
                    if !found_quote && ch == ' ' {
                        continue;
                    }
                    if ch == '\"' {
                        match found_quote {
                            false => {
                                found_quote = true;
                                continue;
                            }
                            true => {
                                let data = data_buffer.iter().collect::<String>();
                                tracing::info!("data: {data}");
                                let data = TaskData::new(data_buffer.iter().collect::<String>());
                                return Ok((data, i + 1));
                            }
                        }
                    }
                    if !found_quote && !data_buffer.is_empty() {
                        return Err(TaskError::ParseError(
                            "data segment does not start with a `\"`".to_string(),
                        ));
                    }
                    if (ch == '[' || ch == ']') && !data_buffer.is_empty() && found_quote {
                        return Err(TaskError::ParseError(
                            "data segment does not end with a `\"`".to_string(),
                        ));
                    }

                    data_buffer.push(ch);
                }
            }
        }

        Err(TaskError::ParseError("failed to parse data".to_string()))
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn data(&self) -> &str {
        &self.data
    }

    pub fn status(&self) -> bool {
        self.status
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name
    }

    pub fn set_data(&mut self, data: String) {
        self.data = data
    }

    pub fn set_status(&mut self, status: bool) {
        self.status = status
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

    fn parse(&mut self, content: String) -> Result<TaskList, TaskError> {
        let mut offset = usize::default();
        let mut list = TaskList::default();

        loop {
            match TaskItem::parse(&content, offset) {
                Ok((item, off)) => {
                    tracing::info!("task list offset {off:?}");
                    match off {
                        ParserOffset::Offset(new_offset) => {
                            offset = new_offset;
                        }
                        ParserOffset::Eof => {
                            list.push(item);
                            return Ok(list);
                        }
                    }
                    tracing::info!("parsed item: {item}");
                    list.push(item);
                }
                Err(error) => return Err(error),
            }
        }
    }

    pub fn iter(&self) -> TaskIter<'_> {
        TaskIter { list: self, idx: 0 }
    }

    pub fn serialize(&self, file: PathBuf) {
        let mut buff = String::default();
        self.clone().iter().for_each(|item| {
            buff.push_str(&format!(
                "{}\n{}{}\n",
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
    use std::fs::OpenOptions;

    use tracing_subscriber::FmtSubscriber;

    use super::TaskList;

    #[test]
    pub fn test_parser() {
        _ = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open("/home/slothy/programming/code/tasks/test_log")
            .expect("truncating log file failed");

        let appender =
            tracing_appender::rolling::never("/home/slothy/programming/code/tasks/", "test_log");
        let subscriber = FmtSubscriber::builder()
            .with_max_level(tracing::Level::TRACE)
            .with_writer(appender)
            .with_ansi(false)
            .finish();

        let _ = tracing::subscriber::set_global_default(subscriber);

        let data = r#"[task1 urmom]
status = false
data = "wasd wasdwa sadwasd wasdwasd\nursogay"
[task2]
status = true
data = "nothing"
[task3]
status = true
data = "urmom"
"#;
        let parser = TaskList::new().parse(data.to_string()).unwrap();
        println!("{parser}");
        assert!(parser.list.len() == 3);
    }
}
