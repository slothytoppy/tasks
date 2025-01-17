use anathema::backend::tui::Screen;
use anathema::backend::tui::TuiBackend;
use anathema::component::Component;
use anathema::component::ComponentId;
use anathema::runtime::Runtime;
use anathema::state::State;
use anathema::templates::Document;

use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use task_editor::TaskEditor;
use task_editor::TaskEditorState;
use tracing_subscriber::FmtSubscriber;

use tasks_core::tasks::*;

mod navbar;
mod selection;
mod task_editor;

use navbar::*;
use selection::*;

#[derive(Default)]
struct App;

#[derive(Debug, State)]
struct AppState {
    #[state_ignore]
    id: ComponentId<String>,
}

impl Component for App {
    type State = AppState;
    type Message = ();

    fn accept_focus(&self) -> bool {
        false
    }

    fn receive(
        &mut self,
        ident: &str,
        value: anathema::state::CommonVal<'_>,
        state: &mut Self::State,
        _elements: anathema::widgets::Elements<'_, '_>,
        mut context: anathema::prelude::Context<'_, Self::State>,
    ) {
        if !ident.eq("task") {
            tracing::info!("incorrect ident: {ident}, expected ident: task");
            return;
        }
        context.set_focus("id", 1);
        context.emit(state.id, value.to_string());
    }
}

fn main() {
    setup_hook();
    setup_logger("log");

    let data = std::fs::read_to_string("examples/tasks.tl").unwrap();
    let task_list = TaskList::deserialize(data).expect("failed with");

    let document = Document::new("@main");

    let backend = TuiBackend::builder()
        .enable_alt_screen()
        .enable_raw_mode()
        .enable_mouse()
        .hide_cursor()
        .finish()
        .unwrap();

    let mut runtime = Runtime::builder(document, backend);

    let editor = runtime
        .register_component(
            "editor",
            "./templates/task_editor.aml",
            TaskEditor {},
            TaskEditorState::new("".to_string()),
        )
        .unwrap();

    let selection = runtime
        .register_component(
            "selection",
            "./templates/list.aml",
            TaskSelection {},
            TaskSelectionState::new(task_list),
        )
        .expect("failed to register list component");

    let _ = runtime.register_component(
        "main",
        "./templates/main.aml",
        App {},
        AppState { id: editor },
    );

    let _ = runtime
        .register_component(
            "navbar",
            "./templates/navbar.aml",
            NavBar {},
            NavBarState::new(editor, selection, Placement::Relative, 0, 0),
        )
        .expect("failed to register navbar");

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
