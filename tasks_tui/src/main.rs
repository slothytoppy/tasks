use anathema::backend::tui::TuiBackend;
use anathema::component::{Component, ComponentId, MouseEvent, MouseState};
use anathema::default_widgets::Overflow;
use anathema::runtime::Runtime;
use anathema::state::{CommonVal, List, State, Value};
use anathema::templates::Document;

use tasks_core::iterator::*;
use tasks_core::tasks::*;

#[derive(Default)]
struct Task {
    item: TaskList,
    message: String,
}

impl Task {
    pub fn new(item: TaskList) -> Self {
        Self {
            message: item.to_string(),
            item,
        }
    }
}

impl State for Task {
    fn to_common(&self) -> Option<CommonVal<'_>> {
        Some(CommonVal::Str(&self.message))
    }
}

#[derive(Default, State)]
struct ListState {
    list: Value<List<String>>,
    #[state_ignore]
    state: Task,
}

impl ListState {
    fn new(state: Task) -> Self {
        let list = List::from_iter(vec![state.message.clone()]);
        Self { state, list }
    }
}

#[derive(Default)]
struct ComponentList;

impl Component for ComponentList {
    type State = ListState;
    type Message = String;

    fn on_mouse(
        &mut self,
        mouse: MouseEvent,
        _: &mut Self::State,
        mut elements: anathema::widgets::Elements<'_, '_>,
        _: anathema::prelude::Context<'_, Self::State>,
    ) {
        elements.by_tag("overflow").first(|el, _| {
            let overflow = el.to::<Overflow>();
            match mouse.state {
                MouseState::ScrollUp => overflow.scroll_up_by(3),
                MouseState::ScrollDown => overflow.scroll_down_by(3),
                _ => {}
            }
        });
    }

    fn message(
        &mut self,
        _: Self::Message,
        state: &mut Self::State,
        _: anathema::widgets::Elements<'_, '_>,
        _: anathema::prelude::Context<'_, Self::State>,
    ) {
        state
            .list
            .push_back(format!("message: {}", state.state.message));
    }
}

struct ComponentIndex {
    component: ComponentId<String>,
}

impl Component for ComponentIndex {
    type Message = ();
    type State = ();

    fn on_mouse(
        &mut self,
        mouse: anathema::component::MouseEvent,
        state: &mut Self::State,
        mut elements: anathema::widgets::Elements<'_, '_>,
        mut context: anathema::prelude::Context<'_, Self::State>,
    ) {
        if mouse.lsb_down() {
            elements
                .at_position(mouse.pos())
                .by_attribute("id", "button")
                .first(|_, _| context.emit(self.component, "message".into()));
        }
    }
}

impl ComponentIndex {
    pub fn new(component: ComponentId<String>) -> Self {
        Self { component }
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

    let task_list = TaskList::deserialize(data.to_string()).expect("failed to parse file");

    let list_state = ListState::new(Task::new(task_list));

    let list = runtime
        .register_component::<ComponentList>(
            "list",
            "./templates/list.aml",
            ComponentList {},
            list_state,
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
