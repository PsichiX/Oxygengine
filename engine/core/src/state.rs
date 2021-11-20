use crate::{ecs::Universe, id::ID};
use std::marker::PhantomData;

pub enum StateChange {
    None,
    Push(Box<dyn State>),
    Pop,
    Swap(Box<dyn State>),
    Quit,
}

pub trait State: Send + Sync {
    fn on_enter(&mut self, _universe: &mut Universe) {}

    fn on_exit(&mut self, _universe: &mut Universe) {}

    fn on_pause(&mut self, _universe: &mut Universe) {}

    fn on_resume(&mut self, _universe: &mut Universe) {}

    fn on_process(&mut self, _universe: &mut Universe) -> StateChange {
        StateChange::None
    }

    fn on_process_background(&mut self, _universe: &mut Universe) {}
}

impl State for () {}

impl State for bool {
    fn on_process(&mut self, _universe: &mut Universe) -> StateChange {
        if *self {
            StateChange::None
        } else {
            StateChange::Quit
        }
    }
}

impl State for usize {
    fn on_process(&mut self, _universe: &mut Universe) -> StateChange {
        if *self > 0 {
            *self -= 1;
            StateChange::None
        } else {
            StateChange::Quit
        }
    }
}

pub type StateToken = ID<PhantomData<dyn State + Send + Sync>>;
