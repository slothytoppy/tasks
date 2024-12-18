use std::{fmt::Display};

use anathema::{
    component::{Component, KeyCode},
    state::{State, Value},
};
use tasks_core::{
    parser::{Parser, ParserState},
    tasks::TaskError,
};

#[derive(Default, Debug, State)]
pub struct TaskEditorState {
    content: Value<String>,
    name: Value<String>,
    status: Value<Option<bool>>,
    data: Value<String>,
    #[state_ignore]
    idx: usize,
}

impl TaskEditorState {
    pub fn new(content: String) -> Self {
        Self {
            idx: content.len(),
            content: content.into(),
            ..Default::default()
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

impl Display for TaskEditorState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "{:?} {:?} {:?}",
            self.name.to_ref(),
            self.status.to_ref(),
            self.data.to_ref()
        ))
    }
}

impl Parser<TaskEditorState, TaskError> for TaskEditorState {
    fn parse(&mut self, content: String) -> Result<TaskEditorState, TaskError> {
        let mut editor = TaskEditorState::default();
        let mut state = ParserState::Name;

        for line in content.lines() {
            match state {
                ParserState::Name => {
                    editor.name.set(line.trim().to_string());
                    state = ParserState::Status;
                }
                ParserState::Status => {
                    if line.eq("true") {
                        editor.status.set(Some(true));
                    } else if line.eq("false") {
                        editor.status.set(Some(false));
                    } else {
                        return Err(TaskError::ParseError("Incorrect status".to_string()));
                    }
                    state = ParserState::Data;
                }
                ParserState::Data => {
                    editor.data.set(line.trim().to_string());
                }
            }
        }
        Ok(editor)
    }
}

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
        if let Ok(item) = state.parse(message.clone()) {
            state.name.set(item.name.to_ref().to_string());
            state.status.set(*item.status.to_ref());

            let str = item.data.to_ref().to_string();

            let mut nl = (false, false);

            let str = str
                .chars()
                .map(|c| match c {
                    '\\' => {
                        nl.0 = true;
                        ' '
                    }
                    'n' => {
                        if nl.0 {
                            nl.1 = true;
                            '\n'
                        } else {
                            'n'
                        }
                    }
                    _ => c,
                })
                .collect();

            state.data.set(str);
        } else {
            tracing::info!("failed to parse");
        }
        tracing::info!("parsed: {state}");
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
