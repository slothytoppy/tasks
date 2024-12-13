use anathema::backend::tui::Screen;
use anathema::backend::tui::TuiBackend;
use anathema::runtime::Runtime;
use anathema::templates::Document;

use std::fs::OpenOptions;
use std::path::Path;
use tracing_subscriber::FmtSubscriber;

use std::io::Write;
use tasks_core::tasks::*;

mod component_index;
mod selection;

use component_index::*;
use selection::*;

#[derive(Default, Debug)]
struct Task {
    item: TaskList,
}

impl Task {
    pub fn new(item: TaskList) -> Self {
        Self { item }
    }
}

fn main() {
    setup_hook();
    setup_logger("log");

    let data = std::fs::read_to_string("examples/tasks.tl").unwrap();

    let task_list = TaskList::deserialize(data.to_string()).expect("failed with");

    let document = Document::new("@main");

    let backend = TuiBackend::builder()
        .enable_alt_screen()
        .enable_raw_mode()
        .enable_mouse()
        .hide_cursor()
        .finish()
        .unwrap();

    let mut runtime = Runtime::builder(document, backend);

    let selection_state = TaskSelectionState::new(Task::new(task_list));

    tracing::info!("{}", selection_state);

    let list = runtime
        .register_component::<TaskSelection>(
            "selection",
            "./templates/list.aml",
            TaskSelection {},
            selection_state,
        )
        .expect("failed to register list component");

    let _ = runtime.register_component::<ComponentIndex>(
        "main",
        "./templates/main.aml",
        ComponentIndex::new(list),
        (),
    );

    runtime.finish().unwrap().run();
}

fn setup_hook() {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let mut stdout = std::io::stdout();
        let _ = write!(stdout, "\x1B[0 q");
        _ = Screen::new((0, 0)).restore(stdout);
        hook(info);
    }));
}

fn setup_logger<P: AsRef<Path>>(file: P) {
    _ = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(file.as_ref())
        .expect("truncating log file failed");

    let appender = tracing_appender::rolling::never(".", file);
    let subscriber = FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .with_writer(appender)
        .with_ansi(false)
        .finish();

    let _ = tracing::subscriber::set_global_default(subscriber);
}
