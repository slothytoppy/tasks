use anathema::backend::tui::Screen;
use anathema::backend::tui::TuiBackend;
use anathema::component::Component;
use anathema::component::ComponentId;
use anathema::runtime::Runtime;
use anathema::state::State;
use anathema::templates::Document;
use task_editor::TaskEditor;
use task_editor::TaskEditorState;

use std::fmt::Display;
use std::fs::OpenOptions;
use std::path::Path;
use tracing_subscriber::FmtSubscriber;

use std::io::Write;
use tasks_core::tasks::*;

mod selection;
mod task_editor;

use selection::*;

#[derive(Default, Debug)]
struct Task {
    item: TaskList,
}

impl Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.item.to_string())
    }
}

impl Task {
    pub fn new(item: TaskList) -> Self {
        Self { item }
    }
}

#[derive(Default)]
struct App;

impl Component for App {
    type State = AppState;
    type Message = ();

    fn accept_focus(&self) -> bool {
        false
    }

    fn receive(
        &mut self,
        _ident: &str,
        value: anathema::state::CommonVal<'_>,
        state: &mut Self::State,
        _elements: anathema::widgets::Elements<'_, '_>,
        mut context: anathema::prelude::Context<'_, Self::State>,
    ) {
        context.set_focus("id", 1);
        context.emit(state.id, value.to_string());
    }
}

#[derive(Debug, State)]
struct AppState {
    #[state_ignore]
    id: ComponentId<String>,
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

    let id = runtime
        .register_component(
            "editor",
            "./templates/task_editor.aml",
            TaskEditor {},
            TaskEditorState::new("".to_string()),
        )
        .unwrap();

    let _ = runtime.register_component("main", "./templates/main.aml", App {}, AppState { id });

    let selection_state = TaskSelectionState::new(Task::new(task_list));

    let _ = runtime
        .register_component(
            "selection",
            "./templates/list.aml",
            TaskSelection {},
            selection_state,
        )
        .expect("failed to register list component");

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
