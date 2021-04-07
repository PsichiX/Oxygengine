use core::Scalar;
use std::any::Any;

pub trait InputDevice: Any + Send + Sync {
    fn name(&self) -> &str;
    fn on_register(&mut self) {}
    fn on_unregister(&mut self) {}
    fn process(&mut self);
    fn query_axis(&self, name: &str) -> Option<Scalar>;
    fn query_trigger(&self, name: &str) -> Option<bool>;
    fn query_text(&self) -> Option<String>;
    fn as_any(&self) -> &dyn Any;
}
