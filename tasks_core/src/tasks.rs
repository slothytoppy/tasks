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

fn open<P: AsRef<std::path::Path>>(file: P) -> std::io::Result<std::fs::File> {
    std::fs::File::open(file)
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
    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

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

        let chars = data.chars();
        let mut list = TaskList::default();

        let mut valid_start = false;
        let mut status_section = false;

        let mut header_start = 0;
        let mut data_start = 0;

        let mut name = "";
        let mut content: &str;
        let mut status = false;

        for (i, ch) in chars.enumerate() {
            if list.is_empty() && i == data.len().saturating_sub(1) {
                content = &data[data_start + 1..data.len()];
                list.push(Task::new(content.trim(), name, status));
            }
            match ch {
                '[' => {
                    header_start = i + 1;
                    if valid_start {
                        //println!("front {push_front} back {push_back}");
                        //println!("start {data_start} end {data_end}");
                        //println!("stuff {:#?}", &data[41..=44]);
                        content = &data[data_start..i.saturating_sub(1)];
                        list.push(Task::new(content.trim(), name, status));
                        continue;
                    }
                    valid_start = true;
                }
                ']' => {
                    if !valid_start {
                        return Err(TaskError::ParseError(('[', i)));
                    }
                    name = &data[header_start..i];
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
            Ok(list) => {
                println!("{list:?}");
                list
            }
            Err(e) => panic!("{e}"),
        };

        panic!();
    }
}
