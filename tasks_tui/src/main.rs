use anathema::backend::tui::TuiBackend;
use anathema::component::Component;
use anathema::runtime::Runtime;
use anathema::state::{CommonVal, State, Value};
use anathema::templates::Document;

use tasks_core::tasks::*;

#[derive(Debug)]
struct List {
    task_list: TaskList,

    list: Value<String>,
}

impl State for List {
    fn to_common(&self) -> Option<CommonVal> {
        self.list.to_common()
    }
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

    //println!("{:?}", data.bytes());

    let task_list = TaskList::deserialize(data.to_string()).expect("failed to parse file");

    println!("{task_list:?}");

    //let list = List {
    //    list: Value::new(task_list.to_string()),
    //    task_list,
    //};
    //
    //let _ = runtime
    //    .register_component::<List>("list", "./templates/list.aml", list, ListState::default())
    //    .expect("failed to register list component");
    //
    //let _ = runtime.register_default::<App>("main", "./templates/main.aml");
    //
    //runtime.finish().unwrap().run();
}
