use std::fmt::Display;

use anathema::{
    component::{Component, KeyCode, MouseEvent, MouseState},
    default_widgets::Overflow,
    state::{List, State, Value},
};

use crate::Task;

#[derive(Default, Debug, State)]
pub struct TaskSelectionState {
    selection: Value<List<String>>,
    #[state_ignore]
    list: Task,
    #[state_ignore]
    selected: usize,
}

impl Display for TaskSelectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.selection
            .to_ref()
            .iter()
            .for_each(|item| tracing::info!("{:?}", item.to_ref()));
        Ok(())
    }
}

impl TaskSelectionState {
    pub fn new(list: Task) -> Self {
        let mut data = List::empty();

        for item in &list.item.list {
            let name = item.name().to_string();
            data.push_back(name);
        }

        Self {
            selection: data,
            list,
            selected: 0,
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
        mut _context: anathema::prelude::Context<'_, Self::State>,
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

        let y = mouse.pos().y as usize;
        let mut line: usize = 0;
        for (i, task) in state.list.item.iter().enumerate() {
            let range = line.saturating_sub(3)..=line + 3;
            if range.contains(&y) {
                state.selected = line;
                tracing::info!("index = {}", i);
                break;
            }
            line += task.name().lines().count() + task.data().lines().count() + 1;
            // task status is a line
        }
    }
}
