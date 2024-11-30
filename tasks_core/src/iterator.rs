use std::{ops::Range, slice::SliceIndex};

pub struct TaskIter {
    idx: usize,
    source: String,
    names: Vec<Range<usize>>,
    datas: Vec<Range<usize>>,
    status: Vec<bool>,
}

impl TaskIter {
    pub fn new(
        source: String,
        names: Vec<Range<usize>>,
        datas: Vec<Range<usize>>,
        status: Vec<bool>,
    ) -> Self {
        Self {
            source,
            names,
            datas,
            status,
            idx: 0,
        }
    }
}

pub struct TaskItem {
    pub name: String,
    pub data: String,
    pub status: bool,
}

impl TaskItem {
    pub fn new(name: String, data: String, status: bool) -> Self {
        Self { name, data, status }
    }
}

impl Iterator for TaskIter {
    type Item = TaskItem;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.names.len()
            || self.idx < self.datas.len()
            || self.idx < self.status.len()
        {
            return None;
        }
        let name = self.names.get(self.idx);
        if name.is_none() {
            return None;
        }
        let name = name.unwrap();
        let name = self.source.get(name.start..name.end).unwrap();

        let data = self.datas.get(self.idx);
        if data.is_none() {
            return None;
        }
        let data = data.unwrap();
        let data = self.source.get(data.start..data.end).unwrap();

        let status = self.status.get(self.idx);
        if status.is_none() {
            return None;
        }
        let status = status.unwrap();

        self.idx += 1;
        return Some(TaskItem::new(name.to_string(), data.to_string(), *status));
    }
}
