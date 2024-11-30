use crate::iterator::*;
use std::ops::Range;
use std::{fmt::Display, path::PathBuf};

#[derive(Debug, PartialEq, Eq)]
pub enum TaskError {
    NoFile(String),
    NoData,
    ParseError((char, usize)),
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

#[derive(Default, Debug, Copy, Clone)]
pub struct Task<'task> {
    data: &'task str,
    name: &'task str,
    status: bool,
}

impl IntoIterator for TaskList {
    type Item = TaskItem;
    type IntoIter = TaskIter;

    fn into_iter(self) -> Self::IntoIter {
        TaskIter::new(self.source, self.names, self.datas, self.status)
    }
}

impl<'task> Display for Task<'task> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("[{}]\n{}\n{}", self.name, self.status, self.data))
    }
}

impl<'task> Task<'task> {
    pub fn new(data: &'task str, name: &'task str, status: bool) -> Self {
        Self { data, name, status }
    }

    pub fn data(&self) -> &'task str {
        self.data
    }

    pub fn set_data(&mut self, data: &'task str) {
        self.data = data
    }

    pub fn status(&self) -> bool {
        self.status
    }

    pub fn set_status(&mut self, status: bool) {
        self.status = status
    }

    pub fn name(&self) -> &str {
        self.name
    }
}

#[derive(Default, Debug, Clone)]
pub struct TaskList {
    source: String,
    names: Vec<Range<usize>>,
    datas: Vec<Range<usize>>,
    status: Vec<bool>,
}

impl TaskList {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn serialize(&self, file: PathBuf) {
        let mut buff = String::default();
        self.clone().into_iter().for_each(|item| {
            buff.push_str(&format!("[{}]\n{} {}\n", item.name, item.status, item.data));
        });
        let _ = std::fs::write(file, buff);
    }

    pub fn deserialize(source: String) -> Result<Self, TaskError> {
        if source.is_empty() {
            return Err(TaskError::NoData);
        }

        let chars = source.chars();

        let mut list = TaskList::default();
        list.source = source.clone();

        let mut valid_start = false;
        let mut status_section = false;

        let mut header_start = 0;
        let mut data_start = 0;

        let mut name = Range::<usize>::default();
        let mut content = Range::<usize>::default();
        let mut status = false;

        for (i, ch) in chars.enumerate() {
            if list.is_empty() && i == source.len().saturating_sub(1) {
                content = data_start + 1..source.len();
                list.push(content, name.clone(), status);
            }
            match ch {
                '[' => {
                    header_start = i + 1;
                    if valid_start {
                        //println!("front {push_front} back {push_back}");
                        //println!("start {data_start} end {data_end}");
                        //println!("stuff {:#?}", &data[41..=44]);
                        content = data_start..i.saturating_sub(1);
                        list.push(content, name.clone(), status);
                        continue;
                    }
                    valid_start = true;
                }
                ']' => {
                    if !valid_start {
                        return Err(TaskError::ParseError(('[', i)));
                    }
                    name = header_start..i;
                    status_section = true;
                }
                '0' | '1' => {
                    if status_section {
                        status = ch != '0';
                        status_section = false;
                        data_start = i + 1;
                    }
                }
                _ => {
                    if !valid_start {
                        return Err(TaskError::ParseError((ch, i)));
                    }
                }
            }
        }

        Ok(list)
    }

    pub fn is_empty(&self) -> bool {
        //self.list.is_empty()
        true
    }

    pub fn get(&self, idx: usize) -> Option<&Task> {
        //self.list.get(idx)
        None
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut Task> {
        //self.list.get_mut(idx)
        None
    }

    pub fn push(&mut self, name: Range<usize>, data: Range<usize>, status: bool) {
        self.names.push(name);
        self.datas.push(data);
        self.status.push(status);
    }

    pub fn set(&mut self, idx: usize, task: Task) {
        //self.list.insert(idx, task)
    }
}

impl Display for TaskList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.clone().into_iter().for_each(|item| {
            let _ = write!(f, "[{}]\n{}\n", item.name, item.name);
        });
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

        let _ = match TaskList::deserialize(data.to_string()) {
            Ok(list) => {
                println!("{list:?}");
                list
            }
            Err(e) => panic!("{e}"),
        };

        panic!();
    }
}
