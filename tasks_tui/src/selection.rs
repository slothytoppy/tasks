use std::{fmt::Display, ops::Deref};

use anathema::{
    component::{Component, MouseEvent, MouseState},
    default_widgets::Overflow,
    state::{CommonVal, List, State, Value},
};

use crate::Task;

#[derive(Default, Debug, State)]
pub struct TaskSelectionState {
    selection: Value<List<String>>,
    border_width: Value<usize>,
    selected: Value<Option<usize>>,
    selected_item: Value<String>,
    list: Value<Task>,
}

impl State for Task {
    fn to_common(&self) -> Option<anathema::state::CommonVal<'_>> {
        Some(CommonVal::Str("hello"))
    }
}

impl Display for TaskSelectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:?}", self.selected.to_ref().to_number()))
    }
}

impl TaskSelectionState {
    pub fn new(list: Task) -> Self {
        let mut data = List::empty();
        let mut border_width = 0;

        for item in &list.item.list {
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
            list: Value::new(list),
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

    fn message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        _elements: anathema::widgets::Elements<'_, '_>,
        _context: anathema::prelude::Context<'_, Self::State>,
    ) {
        tracing::info!("got message {message}");
        state.selection.push_back(message);
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
        for (i, task) in state.list.to_ref().item.iter().enumerate() {
            if y == i + 1 {
                state.selected_item = match state.list.to_ref().item.get(line) {
                    Some(task) => {
                        if x <= task.name().len() {
                            state.selected.set(Some(line));
                            Value::new(task.to_string())
                        } else {
                            break;
                        }
                    }
                    None => Value::new(String::default()),
                };
                context.publish("selected", |state| &state.selected_item);
                break;
            }
            line += task.name().lines().count();
        }
    }
}
