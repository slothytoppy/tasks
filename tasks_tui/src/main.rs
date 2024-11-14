use tasks_core::tasks::*;

fn main() {
    let task_list = TaskList::deserialize("./example/task.tl");

    println!("{task_list:?}");
}
