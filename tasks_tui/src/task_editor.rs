use std::fmt::Display;

use anathema::{
    component::{Component, KeyCode},
    state::{State, Value},
};
use tasks_core::tasks::{ParserState, TaskError};

#[derive(Clone, Debug, PartialEq)]
enum EditingState {
    Name,
    Status,
    Data,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Direction {
    Left,
    Right,
}

#[derive(Default, Debug, State)]
pub struct TaskEditorState {
    is_selected: Value<bool>,
    name: Value<String>,
    status: Value<bool>,
    data: Value<String>,
    #[state_ignore]
    idx: usize,
    #[state_ignore]
    selected: Option<EditingState>,
}

impl TaskEditorState {
    pub fn new(content: String) -> Self {
        Self {
            idx: content.len(),
            ..Default::default()
        }
    }

    pub fn push(&mut self, idx: usize, ch: char) {
        let Some(ref state) = self.selected else {
            return;
        };
        match state {
            EditingState::Name => {
                self.name.to_mut().insert(idx, ch);
                self.idx += 1;
            }
            EditingState::Data => {
                self.data.to_mut().insert(idx, ch);
                self.idx += 1;
            }
            _ => {}
        }
    }

    pub fn remove(&mut self, idx: usize) {
        if self.idx == 0 {
            return;
        }
        let Some(ref state) = self.selected else {
            return;
        };
        match state {
            EditingState::Name => {
                self.name.to_mut().remove(idx);
                self.idx -= 1;
            }
            EditingState::Data => {
                self.data.to_mut().remove(idx);
                self.idx -= 1;
            }
            _ => {}
        }
    }

    pub fn move_direction(&mut self, direction: Direction) {
        let Some(ref state) = self.selected else {
            return;
        };
        match direction {
            Direction::Left => match state {
                EditingState::Status => {}
                _ => {
                    if self.idx > 0 {
                        self.idx -= 1;
                    }
                }
            },
            Direction::Right => match state {
                EditingState::Name => {
                    if self.idx < self.name.to_ref().len().saturating_sub(1) {
                        self.idx += 1;
                    }
                }
                EditingState::Data => {
                    if self.idx < self.data.to_ref().len().saturating_sub(1) {
                        self.idx += 1;
                    }
                }
                _ => {}
            },
        }
    }

    pub fn toggle_status(&mut self) {
        self.status.set(self.status.to_bool());
    }

    fn parse(&mut self, content: String) -> Result<TaskEditorState, TaskError> {
        let mut editor = TaskEditorState::default();
        let mut state = ParserState::Name;
        let mut data = String::default();

        for (i, line) in content.lines().enumerate() {
            match state {
                ParserState::Name => {
                    editor.name.set(line.trim().to_string());
                    state = ParserState::Status;
                }
                ParserState::Status => {
                    if line.eq("true") {
                        editor.status.set(true);
                    } else if line.eq("false") {
                        editor.status.set(false);
                    } else {
                        return Err(TaskError::ParseError("Incorrect status".to_string()));
                    }
                    state = ParserState::Data;
                }
                ParserState::Data => {
                    data.push_str(line.trim());
                    editor.data.set(line.trim().to_string());
                    tracing::info!("{i}");
                }
            }
        }
        editor.data.set(data);
        Ok(editor)
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

impl Component for TaskEditor {
    type State = TaskEditorState;
    type Message = String;

    fn on_mouse(
        &mut self,
        mouse: anathema::component::MouseEvent,
        state: &mut Self::State,
        mut elements: anathema::widgets::Elements<'_, '_>,
        mut _context: anathema::prelude::Context<'_, Self::State>,
    ) {
        // if layout changes, this sucks
        if mouse.x < 12 && !mouse.lsb_down() {
            return;
        }
        let name_start = 2..4;
        let status_start = 5..7;
        let data_start = 8..10;
        elements.by_tag("border").each(|_, _| {
            if name_start.contains(&mouse.y) {
                state.selected = Some(EditingState::Name);
                state.idx = state.name.to_ref().len();
                tracing::info!("EDITING NAME");
            }
            if status_start.contains(&mouse.y) {
                state.selected = Some(EditingState::Status);
                state.idx = match state.status.to_ref().to_bool() {
                    true => 4,
                    false => 5,
                };
                tracing::info!("EDITING STATUS");
            }
            if data_start.contains(&mouse.y) {
                state.selected = Some(EditingState::Data);
                state.idx = state.data.to_ref().len();
                tracing::info!("EDITING DATA");
            }
            //tracing::info!("el: {el:?}")
        });
    }

    fn on_key(
        &mut self,
        key: anathema::component::KeyEvent,
        state: &mut Self::State,
        _elements: anathema::widgets::Elements<'_, '_>,
        _context: anathema::prelude::Context<'_, Self::State>,
    ) {
        if state.selected.is_none() {
            return;
        }
        if let Some(selected) = &state.selected {
            match selected {
                EditingState::Status => {
                    if let KeyCode::Char('t') = key.code {
                        state.toggle_status()
                    }
                }
                EditingState::Name | EditingState::Data => match key.code {
                    KeyCode::Char(ch) => state.push(state.idx, ch),
                    KeyCode::Backspace => state.remove(state.idx.saturating_sub(1)),
                    KeyCode::Left => state.move_direction(Direction::Left),
                    KeyCode::Right => state.move_direction(Direction::Right),
                    _ => {}
                },
            }
        }
        //match key.code {
        //    KeyCode::Char('t') => state.toggle_status(),
        //    KeyCode::Char(c) => {
        //        state.push(c);
        //        state.idx += 1;
        //    }
        //    KeyCode::Backspace => {
        //        if state.idx > 0 {
        //            state.remove(state.idx);
        //            state.idx -= 1;
        //        }
        //    }
        //    KeyCode::Left => state.idx -= 1,
        //    KeyCode::Right => state.idx += 1,
        //    _ => {}
        //}
    }

    fn message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        _elements: anathema::widgets::Elements<'_, '_>,
        _context: anathema::prelude::Context<'_, Self::State>,
    ) {
        if let Ok(item) = state.parse(message.clone()) {
            state.is_selected.set(true);
            state.name.set(item.name.to_ref().to_string());
            state.status.set(*item.status.to_ref());

            let str = item.data.to_ref().to_string();

            tracing::info!("out editor {str:?}");

            let mut nl = (false, false);

            let data = str
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

            tracing::info!("in editor {str:?}");
            state.data.set(data);
        } else {
            tracing::info!("failed to parse");
        }
    }
}
