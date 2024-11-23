use std::{fmt::Display, path::PathBuf};

#[derive(Default, Debug)]
pub struct Task<'a> {
    data: &'a str,
    name: &'a str,
    status: bool,
}

impl<'a> Task<'a> {
    pub fn new(data: &'a str, name: &'a str) -> Self {
        Self {
            data,
            name,
            status: false,
        }
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

#[derive(Debug, PartialEq, Eq)]
pub enum TaskError {
    ENOFILE(String),
    NoData,
    ParseError(u32),
}

impl Display for TaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskError::ParseError(c) => write!(f, "ParseError at char {c}"),
            TaskError::ENOFILE(file) => write!(f, "ENOFILE: {file}"),
            TaskError::NoData => write!(f, "No data to parse"),
        }
    }
}

fn open<P: AsRef<std::path::Path>>(file: P) -> std::io::Result<std::fs::File> {
    std::fs::File::open(file)
}

//fn parse(data: &str) -> Result<Task, TaskError> {
//    if data.is_empty() {
//        return Err(TaskError::NoData);
//    }
//
//    let mut task = Task::default();
//    let mut has_start = false;
//    let mut has_end = false;
//    let mut pos = 0;
//    let data = data.trim();
//
//    for (i, c) in data.chars().enumerate() {
//        if !has_end {
//            match c {
//                ' ' => continue,
//                '[' => has_start = true,
//                ']' => {
//                    if !has_start {
//                        return Err(TaskError::ParseError(i as u32 + 1));
//                    }
//                    task.name = &data[1..=i - 1];
//                    has_end = true;
//                    pos = i + 1;
//                }
//                _ => {}
//            }
//        }
//    }
//
//    println!("{data:?}");
//
//    if data[pos..].is_empty() {
//        return Err(TaskError::NoData);
//    }
//
//    task.name = &data[1..=pos.saturating_sub(2)];
//    task.data = data[pos..].trim();
//
//    Ok(task)
//}

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

    Err(TaskError::ParseError(420000))
}

fn parse_data(data: &str) -> Result<(&str, usize), TaskError> {
    println!("from parse_data");
    //println!("{data:?}");
    if data.is_empty() {
        return Err(TaskError::NoData);
    }

    for (i, c) in data.chars().enumerate() {
        match c {
            //'\t' | '\n' => start += 1,
            //'!'..='Z' | '\\' | '^'..='~' => found_alphanum = true,
            //' ' => {
            //    if !found_alphanum {
            //        start += 1;
            //    }
            //}
            '[' => {
                //println!("{i:?} {:?}", &data[0..i]);
                return Ok((data[0..i].trim(), i.saturating_sub(1)));
            }
            ']' => return Err(TaskError::ParseError(i as u32)),
            _ => {}
        };
    }
    Ok((data, data.len()))
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

    pub fn serialize(&self, file: PathBuf) {}

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
            //let (name, new_pos) = parse_header(&data[pos..])?;
            pos += new_pos;
            let (data, new_pos) = parse_data(&data[pos..])?;
            pos += new_pos;
            list.push(Task::new(data, name));
        }

        Ok(list)
    }
}

#[cfg(test)]
mod test {
    use super::TaskList;

    #[test]
    pub fn test_list() {
        let data = r#"[hai]
            bai
            [bai]
            urmom
            [urmom]
            hai"#;

        match TaskList::deserialize(data) {
            Ok(list) => println!("{list:?}"),
            Err(e) => println!("{e:?}"),
        }

        panic!();
    }
}
