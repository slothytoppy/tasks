use anathema::{
    component::{Component, KeyCode},
    state::{State, Value},
};

#[derive(Default, Debug, State)]
pub struct TaskEditorState {
    content: Value<String>,
    is_selected: Value<bool>,
    #[state_ignore]
    idx: usize,
}

impl TaskEditorState {
    pub fn new(content: String) -> Self {
        Self {
            idx: content.len(),
            content: content.into(),
            is_selected: Value::new(false),
        }
    }

    pub fn push(&mut self, ch: char) {
        self.content.to_mut().push(ch)
    }

    pub fn remove(&mut self, idx: usize) {
        self.content.to_mut().remove(idx.saturating_sub(1));
    }
}

#[derive(Default)]
pub struct TaskEditor;

impl Component for TaskEditor {
    type State = TaskEditorState;

    type Message = String;

    fn message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        _elements: anathema::widgets::Elements<'_, '_>,
        _context: anathema::prelude::Context<'_, Self::State>,
    ) {
        state.idx = message.len();
        state.content.set(message);
    }

    fn on_key(
        &mut self,
        key: anathema::component::KeyEvent,
        state: &mut Self::State,
        _elements: anathema::widgets::Elements<'_, '_>,
        _context: anathema::prelude::Context<'_, Self::State>,
    ) {
        match key.code {
            KeyCode::Char(c) => {
                state.push(c);
                state.idx += 1;
            }
            KeyCode::Backspace => {
                if state.idx > 0 {
                    state.remove(state.idx);
                    state.idx -= 1;
                }
            }
            KeyCode::Left => state.idx -= 1,
            KeyCode::Right => state.idx += 1,
            _ => {}
        }
    }
}
