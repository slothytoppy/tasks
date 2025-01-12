use anathema::{
    component::{Component, KeyCode, MouseEvent, MouseState},
    default_widgets::Overflow,
    state::{List, State, Value},
};
use tasks_core::tasks::{TaskItem, TaskList};

#[derive(Default, Debug, State)]
pub struct TaskSelectionState {
    selection: Value<List<String>>,
    border_width: Value<usize>,
    selected: Value<Option<usize>>,
    selected_item: Value<String>,
    #[state_ignore]
    list: TaskList,
    #[state_ignore]
    buffer: String,
    #[state_ignore]
    creating_item: bool,
}

impl TaskSelectionState {
    pub fn new(list: TaskList) -> Self {
        let mut data = List::empty();
        let mut border_width = 0;

        for item in list.iter() {
            let name = item.name().to_string();
            if name.len() > border_width {
                border_width = name.len();
            }
            data.push_back(name);
        }

        // += 2 because the left and right sides of the border are 1 cell
        border_width += 9;

        Self {
            selection: data,
            list,
            border_width: Value::new(border_width),
            selected: Value::new(None),
            ..Default::default()
        }
    }
}

#[derive(Default)]
pub struct TaskSelection;

impl Component for TaskSelection {
    type State = TaskSelectionState;
    type Message = String;

    fn on_key(
        &mut self,
        key: anathema::component::KeyEvent,
        state: &mut Self::State,
        _elements: anathema::widgets::Elements<'_, '_>,
        _context: anathema::prelude::Context<'_, Self::State>,
    ) {
        tracing::info!("from selection");
        if state.creating_item {
            tracing::info!("creating item mode: {}", state.buffer);
        }
        match key.code {
            KeyCode::Char('x') => {
                if state.list.is_empty() | state.selected.to_ref().is_none() {
                    state.selected_item.set(String::default());
                    return;
                }

                let mut index = state
                    .selected
                    .to_ref()
                    .to_number()
                    .map_or(0, |n| n.as_uint());

                if index.saturating_sub(1) > state.selection.len() {
                    return;
                }

                state.selection.remove(index);
                state.list.remove(index);
                index = index.saturating_sub(1);

                if let Some(str) = state.selection.to_ref().get(index) {
                    let item = str.to_ref().to_string();
                    state.selected_item.set(item);

                    state.selected.set(Some(index));
                }
            }
            KeyCode::Char('c') => state.creating_item = true,
            KeyCode::Char('j') => {}
            KeyCode::Char('k') => {}
            KeyCode::Esc => {
                state.creating_item = false;
                state.buffer.clear();
            }
            KeyCode::Enter => {
                if state.creating_item {
                    let item = TaskItem::new(state.buffer.clone(), String::default(), false);
                    state.list.push(item);
                    state.selection.push(state.buffer.clone());
                    state.buffer.clear();
                    state.creating_item = false;
                }
            }
            KeyCode::Char(ch) => {
                if state.creating_item {
                    state.buffer.push(ch)
                }
            }

            _ => {}
        }
    }

    fn on_mouse(
        &mut self,
        mouse: MouseEvent,
        state: &mut Self::State,
        mut elements: anathema::widgets::Elements<'_, '_>,
        mut context: anathema::prelude::Context<'_, Self::State>,
    ) {
        elements.by_tag("overflow").first(|el, _| {
            let overflow = el.to::<Overflow>();
            match mouse.state {
                MouseState::ScrollUp => overflow.scroll_up_by(3),
                MouseState::ScrollDown => overflow.scroll_down_by(3),
                _ => {}
            }
        });

        if !mouse.lsb_down() {
            return;
        }

        let pos = mouse.pos();
        let (x, y) = (pos.x as usize, pos.y as usize);

        let mut line: usize = 0;
        for (i, task) in state.list.iter().enumerate() {
            // we want to skip the top border,
            // we do i + 1 so that clicking on line 1 returns the first task
            if y == i + 1 {
                if state
                    .selected
                    .to_number()
                    .is_some_and(|n| n.as_uint() == line)
                {
                    state.selected.set(None);
                    break;
                }
                if let Some(task) = state.list.get(line) {
                    if x <= task.name().len() {
                        state.selected.set(Some(line));
                        let item = task.to_string();
                        state.selected_item.set(item);
                    }
                }
                context.publish("selection_bar", |state| &state.selected_item);
                break;
            }
            line += task.name().lines().count();
        }
    }
}
