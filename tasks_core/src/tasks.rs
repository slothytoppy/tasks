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
                write!(f, "Failed to parse: {} at index {}", ch, idx)
            }
            TaskError::NoFile(file) => write!(f, "ENOFILE: {file}"),
            TaskError::NoData => write!(f, "No data to parse"),
        }
    }
}

fn open<P: AsRef<std::path::Path>>(file: P) -> std::io::Result<std::fs::File> {
    std::fs::File::open(file)
}

fn parse_header(data: &str) -> Result<(&str, usize), TaskError> {
    println!("from parse_header");
    //println!("{data:?}");
    if data.is_empty() {
        return Err(TaskError::NoData);
    }
    let mut start = 1;

    for (i, c) in data.chars().enumerate() {
        match c {
            '[' => start = i + 1,
            ']' => {
                //println!("{i:?} {}", &data[1..=i.saturating_sub(1)]);
                return Ok((&data[start..=i.saturating_sub(1)], i + 1));
            }
            _ => {}
        }
    }

    Err(TaskError::ParseError((
        data.chars().next().unwrap_or_default(),
        data.len(),
    )))
}

fn parse_status(data: &str) -> Result<(bool, usize), TaskError> {
    println!("from parse_status");

    if data.is_empty() {
        return Err(TaskError::NoData);
    }

    for (i, c) in data.chars().enumerate() {
        match c {
            '0' => return Ok((false, i + 1)),
            '1' => return Ok((true, i + 1)),
            ' ' | '\n' | '\t' => continue,
            _ => {
                println!("{}", data);
                return Err(TaskError::ParseError((
                    data.chars().next().unwrap_or_default(),
                    i,
                )));
            }
        }
    }

    Err(TaskError::NoData)
}

fn parse_data(data: &str) -> Result<(&str, usize), TaskError> {
    println!("from parse_data");
    //println!("{data:?}");
    if data.is_empty() {
        return Err(TaskError::NoData);
    }

    for (i, c) in data.chars().enumerate() {
        match c {
            '[' => {
                //println!("{i:?} {:?}", &data[0..i]);
                return Ok((data[0..i].trim(), i.saturating_sub(1)));
            }
            ']' => {
                return Err(TaskError::ParseError((
                    data.chars().next().unwrap_or_default(),
                    data.len(),
                )))
            }
            _ => {}
        };
    }
    Ok((data, data.len()))
}

#[derive(Default, Debug)]
pub struct Task<'a> {
    data: &'a str,
    name: &'a str,
    status: bool,
}

impl<'a> Task<'a> {
    pub fn new(data: &'a str, name: &'a str, status: bool) -> Self {
        Self { data, name, status }
    }

    pub fn data(&self) -> &'a str {
        self.data
    }

    pub fn set_data(&mut self, data: &'a str) {
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

#[derive(Default, Debug)]
pub struct TaskList<'a> {
    list: Vec<Task<'a>>,
}

impl<'a> TaskList<'a> {
    pub fn get(&self, idx: usize) -> Option<&Task<'a>> {
        self.list.get(idx)
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut Task<'a>> {
        self.list.get_mut(idx)
    }

    pub fn push(&mut self, task: Task<'a>) {
        self.list.push(task);
    }

    pub fn set(&mut self, idx: usize, task: Task<'a>) {
        self.list.insert(idx, task)
    }

    pub fn serialize(&self, file: PathBuf) {
        let mut buff = String::default();
        self.list.iter().for_each(|task| {
            buff.push_str(&format!(
                "[{}]\n{} {}\n",
                task.name(),
                task.status(),
                task.data()
            ));
        });
        let _ = std::fs::write(file, buff);
    }

    pub fn deserialize(data: &'a str) -> Result<Self, TaskError> {
        if data.is_empty() {
            return Err(TaskError::NoData);
        }

        let chars = data.trim().chars();
        let mut list = TaskList::default();
        let mut pos = 0;

        for _ in chars.enumerate() {
            let (name, new_pos) = match parse_header(&data[pos..]) {
                Ok(res) => res,
                Err(e) => {
                    if e == TaskError::NoData {
                        return Ok(list);
                    } else {
                        return Err(e);
                    }
                }
            };
            pos += new_pos;
            let (status, new_pos) = parse_status(&data[pos..])?;
            pos += new_pos;
            let (data, new_pos) = parse_data(&data[pos..])?;
            pos += new_pos;
            list.push(Task::new(data, name, status));
        }

        Ok(list)
    }
}

impl<'a> Display for TaskList<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.list.iter().for_each(|task: &Task<'a>| {
            let _ = write!(f, "[{}]\n{}\n", task.name(), task.name());
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

        let list = match TaskList::deserialize(data) {
            Ok(list) => list,
            Err(e) => panic!("{e}"),
        };

        //let Ok(path) = PathBuf::from_str("urmom.tl");
        //list.serialize(path);

        println!("{list:?}");

        panic!();
    }
}
