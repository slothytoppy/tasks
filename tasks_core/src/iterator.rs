use std::fmt::Display;

use crate::tasks::TaskList;

pub struct TaskIter<'iter> {
    pub list: &'iter TaskList,
    pub idx: usize,
}

impl<'iter> TaskIter<'iter> {
    pub fn new(list: &'iter TaskList) -> Self {
        Self { list, idx: 0 }
    }
}

#[derive(Debug, Clone)]
pub struct TaskItem {
    pub name: String,
    pub data: String,
    pub status: bool,
}

impl Display for TaskItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("[{}]\n{}\n{}", self.name, self.status, self.data))
    }
}

impl TaskItem {
    pub fn new(name: String, data: String, status: bool) -> Self {
        Self { name, data, status }
    }
}

impl<'iter> Iterator for TaskIter<'iter> {
    type Item = TaskItem;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx > self.list.list.len().saturating_sub(1) {
            return None;
        }

        let res = self.list.get(self.idx)?;
        self.idx += 1;
        Some(res.clone())
    }
}
