use crate::Scalar;

pub trait InputDevice: Send + Sync {
    fn name(&self) -> &str;
    fn on_register(&mut self) {}
    fn on_unregister(&mut self) {}
    fn process(&mut self);
    fn query_axis(&self, name: &str) -> Option<Scalar>;
    fn query_trigger(&self, name: &str) -> Option<bool>;
}
