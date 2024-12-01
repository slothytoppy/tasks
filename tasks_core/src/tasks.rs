use crate::iterator::*;
use std::ops::Range;
use std::{fmt::Display, path::PathBuf};

#[derive(Debug, PartialEq, Eq)]
pub enum TaskError {
    NoFile(String),
    NoData,
    ParseError((char, usize)),
}

#[derive(Debug, Default)]
struct Parser {
    source: String,
    offset: usize,
}

impl Parser {
    fn new(source: String) -> Self {
        Self { source, offset: 0 }
    }

    fn parse(&mut self) -> Result<TaskList, TaskError> {
        if self.source.is_empty() {
            return Err(TaskError::NoData);
        }
        let mut name: &str;
        let mut data: &str;
        let mut status: (usize, bool);
        let mut offset = 0;

        let mut list = TaskList::default();

        for (i, ch) in self.source.chars().enumerate() {
            if ch == ']' {
                name = &self.source[self.parse_header()?];
                offset = i;
                status = self.parse_status(i + 1)?;
                offset += status.0;
                data = self.parse_data(offset)?;
                list.push(TaskItem::new(
                    name.trim().to_string(),
                    data.trim().to_string(),
                    status.1,
                ))
            }
        }

        self.offset = offset;
        Ok(list)
    }

    fn parse_header(&self) -> Result<Range<usize>, TaskError> {
        let mut start = 0;
        for (i, ch) in self.source.chars().enumerate() {
            if ch == '[' {
                start = i + 1;
            }
            if ch == ']' {
                return Ok(start..i);
            }
        }
        Err(TaskError::NoData)
    }

    fn parse_status(&self, offset: usize) -> Result<(usize, bool), TaskError> {
        let mut skip = 0;
        for (i, ch) in self.source.chars().skip(offset).enumerate() {
            match ch {
                '0' => return Ok((i + 1 + skip, false)),
                '1' => return Ok((i + 1 + skip, true)),
                '\n' | '\t' | ' ' => skip += 1,
                _ => {
                    return Err(TaskError::ParseError((ch, i)));
                }
            };
        }
        Err(TaskError::NoData)
    }

    fn parse_data(&self, offset: usize) -> Result<&str, TaskError> {
        for (i, ch) in self.source.chars().skip(offset).enumerate() {
            if ch == '[' {
                return Ok(&self.source[offset..i]);
            }
        }
        Ok(&self.source[offset..])
    }
}

impl Display for TaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskError::ParseError((ch, idx)) => {
                write!(f, "Failed to parse: {:#?} at index {}", ch, idx)
            }
            TaskError::NoFile(file) => write!(f, "ENOFILE: {file}"),
            TaskError::NoData => write!(f, "No data to parse"),
        }
    }
}

pub fn open<P: AsRef<std::path::Path>>(file: P) -> std::io::Result<std::fs::File> {
    std::fs::File::open(file)
}

#[derive(Default, Debug, Clone)]
pub struct TaskList {
    pub list: Vec<TaskItem>,
}

impl TaskList {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn iter(&self) -> TaskIter<'_> {
        TaskIter { list: self, idx: 0 }
    }

    pub fn serialize(&self, file: PathBuf) {
        let mut buff = String::default();
        self.clone().iter().for_each(|item| {
            buff.push_str(&format!("[{}]\n{} {}\n", item.name, item.status, item.data));
        });
        let _ = std::fs::write(file, buff);
    }

    pub fn deserialize(source: String) -> Result<Self, TaskError> {
        Parser::new(source).parse()
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
        let data = r#"[hai]
        0
        bai
            [bai]
            1
            urmom
            [urmom]
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
