use crate::tasks::{TaskItem, TaskList};

pub struct TaskIter<'iter> {
    pub list: &'iter TaskList,
    pub idx: usize,
}

impl<'iter> TaskIter<'iter> {
    pub fn new(list: &'iter TaskList) -> Self {
        Self { list, idx: 0 }
    }
}

impl Iterator for TaskIter<'_> {
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
