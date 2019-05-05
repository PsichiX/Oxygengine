use specs::World;

pub enum StateChange {
    None,
    Push(Box<dyn State>),
    Pop,
    Swap(Box<dyn State>),
    Quit,
}

pub trait State {
    fn on_enter(&mut self, _world: &mut World) {}
    fn on_exit(&mut self, _world: &mut World) {}
    fn on_pause(&mut self, _world: &mut World) {}
    fn on_resume(&mut self, _world: &mut World) {}
    fn on_process(&mut self, _world: &mut World) -> StateChange {
        StateChange::None
    }
    fn on_process_background(&mut self, _world: &mut World) {}
}

impl State for () {}
