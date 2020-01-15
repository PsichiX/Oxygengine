use crate::interface::ResourceModify;
use core::{app::AppLifeCycle, state::StateToken};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct AppLifeCycleScripted {
    pub running: bool,
    pub delta_time_seconds: f64,
    pub current_state_token: StateToken,
}

impl From<&AppLifeCycle> for AppLifeCycleScripted {
    fn from(value: &AppLifeCycle) -> Self {
        Self {
            running: value.running,
            delta_time_seconds: value.delta_time_seconds(),
            current_state_token: value.current_state_token(),
        }
    }
}

impl ResourceModify<AppLifeCycleScripted> for AppLifeCycle {
    fn modify_resource(&mut self, source: AppLifeCycleScripted) {
        self.running = source.running;
    }
}
