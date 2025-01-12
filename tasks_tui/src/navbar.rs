use std::{fmt::Display, ops::Range};

use anathema::{
    component::{Component, ComponentId},
    state::{CommonVal, List, State, Value},
};

#[derive(Default)]
pub struct NavBar;

#[allow(dead_code)]
pub enum Placement {
    Absolute,
    Relative,
}

impl State for Placement {
    fn to_common(&self) -> Option<CommonVal<'_>> {
        match self {
            Self::Absolute => Some(CommonVal::Str("absolute")),
            Self::Relative => Some(CommonVal::Str("relative")),
        }
    }
}

#[derive(State)]
pub struct NavBarState {
    list: Value<List<NavBarId>>,
    placement: Value<Placement>,
    selected: Value<Option<NavBarMessage>>,
    x: Value<usize>,
    y: Value<usize>,
}

#[derive(Debug)]
pub enum NavBarMessage {
    Editor,
    Selection,
}

impl State for NavBarMessage {
    fn to_common(&self) -> Option<CommonVal<'_>> {
        match self {
            Self::Editor => Some(CommonVal::Str("Editor")),
            Self::Selection => Some(CommonVal::Str("Selection")),
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum NavBarId {
    Editor(ComponentId<String>),
    Selection(ComponentId<String>),
}

impl Display for NavBarId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Editor(_) => f.write_str("Editor"),
            Self::Selection(_) => f.write_str("Selection"),
        }
    }
}

impl NavBarState {
    pub fn new(
        editor: ComponentId<String>,
        selection: ComponentId<String>,
        placement: Placement,
        x: usize,
        y: usize,
    ) -> Self {
        let mut list = List::empty();
        list.push(NavBarId::Editor(editor));
        list.push(NavBarId::Selection(selection));
        Self {
            list,
            selected: None.into(),
            placement: placement.into(),
            x: x.into(),
            y: y.into(),
        }
    }
}

impl Component for NavBar {
    type State = NavBarState;
    type Message = NavBarMessage;

    fn message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        _elements: anathema::widgets::Elements<'_, '_>,
        _context: anathema::prelude::Context<'_, Self::State>,
    ) {
        state.selected.set(Some(message));
    }

    fn on_mouse(
        &mut self,
        mouse: anathema::component::MouseEvent,
        state: &mut Self::State,
        mut elements: anathema::widgets::Elements<'_, '_>,
        _context: anathema::prelude::Context<'_, Self::State>,
    ) {
        let pos = mouse.pos();
        let x: usize = pos.x as usize;
        let buffer_len = 1;
        if mouse.lsb_down() {
            elements.by_tag("border").each(|el, _| {
                let el_size = el.size();
                let mut start = 1;
                for item in state.list.to_ref().iter() {
                    let end = item.to_ref().as_ref().to_string().len().saturating_sub(1);
                    let range = Range::<usize> {
                        start,
                        end: start + end + buffer_len,
                    };
                    if range.contains(&x)
                        //&& pos.x as usize <= el_size.width
                        && pos.y as usize <= el_size.height
                    {
                        match item.to_ref().as_ref() {
                            NavBarId::Editor(_) => state.selected.set(Some(NavBarMessage::Editor)),
                            NavBarId::Selection(_) => {
                                state.selected.set(Some(NavBarMessage::Selection))
                            }
                        }
                        tracing::info!("selected: {:?}", state.selected.to_ref().as_ref());
                        break;
                    }
                    start += end;
                }
            });
        }
    }
}

impl State for NavBarId {
    fn to_common(&self) -> Option<CommonVal<'_>> {
        match self {
            NavBarId::Editor(_) => Some(CommonVal::Str("Editor")),
            NavBarId::Selection(_) => Some(CommonVal::Str("Selection")),
        }
    }
}
