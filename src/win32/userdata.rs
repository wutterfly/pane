use std::{borrow::Borrow, sync::Arc};

use crate::events::EventSystem;

pub struct UserData {
    events: Arc<dyn EventSystem + 'static>,
}

impl UserData {
    pub fn new(events: Arc<dyn EventSystem>) -> Self {
        Self { events }
    }

    pub fn events(&self) -> &dyn EventSystem {
        self.events.borrow()
    }
}
