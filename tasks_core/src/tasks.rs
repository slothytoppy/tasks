use crate::iterator::*;
use crate::parser::*;
use std::{fmt::Display, path::PathBuf};

#[derive(Debug, PartialEq, Eq)]
pub enum TaskError {
    NoFile(String),
    NoData,
    ParseError(String),
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
