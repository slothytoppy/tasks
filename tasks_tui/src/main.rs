use std::io::Read;

use anathema::backend::tui::TuiBackend;
use anathema::component::Component;
use anathema::runtime::Runtime;
use anathema::state::{State, Value};
use anathema::templates::Document;

use tasks_core::tasks::*;

#[derive(Clone)]
struct List {
    list: TaskList,
}

#[derive(Default, State)]
struct ListState {
    selected: Value<u64>,
}

impl Component for List {
    type State = ListState;

    type Message = ();
}

#[derive(Default)]
struct App {}

#[derive(Default, State)]
struct AppState {}

impl Component for App {
    type State = AppState;
    type Message = ();
}

impl App {
    pub fn new() -> Self {
        Self {}
    }
}

fn main() {
    let document = Document::new("@main");

    let backend = TuiBackend::builder()
        .enable_alt_screen()
        .enable_raw_mode()
        .enable_mouse()
        .hide_cursor()
        .finish()
        .unwrap();

    let mut runtime = Runtime::builder(document, backend);

    let data = std::fs::read_to_string("./examples/tasks.tl").unwrap();

    let task_list = TaskList::deserialize(data).expect("failed to parse file");

    println!("{task_list:?}");
    let list = List { list: task_list };

    let app = App::new();

    let _ = runtime
        .register_component::<List>("list", "./templates/list.aml", list, ListState::default())
        .expect("failed to register list component");

    runtime.register_component("main", "./templates/main.aml", app, AppState::default());

    runtime.finish().unwrap().run();
}
