use std::{default, error::Error, fmt::Display, io::Read};

#[derive(Default, Debug)]
pub struct Task<'a> {
    data: &'a str,
    status: bool,
}

impl<'a> Task<'a> {
    pub fn new(data: &'a str) -> Self {
        Self {
            data,
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
}

#[derive(Default, Debug)]
pub struct TaskList<'a> {
    list: Vec<Task<'a>>,
    file: &'a str,
}

impl<'a> TaskList<'a> {
    pub fn new<P: AsRef<std::path::Path> + 'a + ?Sized>(file: &'a P) -> Result<TaskList<'_>, ()> {
        Self::deserialize(file)
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

    pub fn serialize(&self) {}

    pub fn deserialize<P: AsRef<std::path::Path> + 'a + ?Sized>(file: &'a P) -> Result<Self, ()> {
        let file = file.as_ref().to_str().unwrap_or_default();

        let mut file = match std::fs::OpenOptions::new().read(true).open(file) {
            Ok(file) => file,
            Err(..) => {
                return Err(());
            }
        };

        let mut buf = String::default();
        let res = file.read_to_string(&mut buf);

        Ok(Self::default())
    }
}

#[cfg(test)]
mod test {
    use super::{Task, TaskList};

    #[test]
    pub fn test_list() {
        let mut list = match TaskList::new("./tasks.rs") {
            Ok(list) => list,
            Err(e) => {
                panic!("{e:?}")
            }
        };

        list.push(Task::new("hello"));
        list.push(Task::new("bye"));

        println!("{list:?}");
        panic!();
    }
}
