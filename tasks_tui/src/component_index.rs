use anathema::component::{Component, ComponentId};

pub struct ComponentIndex {
    component: ComponentId<String>,
}

impl Component for ComponentIndex {
    type Message = ();
    type State = ();

    fn on_mouse(
        &mut self,
        mouse: anathema::component::MouseEvent,
        _: &mut Self::State,
        mut elements: anathema::widgets::Elements<'_, '_>,
        context: anathema::prelude::Context<'_, Self::State>,
    ) {
        if mouse.lsb_down() {
            elements
                .at_position(mouse.pos())
                .by_attribute("id", "button")
                .first(|_, _| context.emit(self.component, "hai".into()));
            tracing::info!("sent message: hai");
        }
    }
}

impl ComponentIndex {
    pub fn new(component: ComponentId<String>) -> Self {
        Self { component }
    }
}
