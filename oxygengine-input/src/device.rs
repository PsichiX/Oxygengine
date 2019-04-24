use crate::Scalar;

pub trait InputDevice: Send + Sync {
    fn on_register(&mut self) {}
    fn on_unregister(&mut self) {}
    fn process(&mut self);
    fn query_axes(&self) -> &[(&str, Scalar)];
    fn query_triggers(&self) -> &[(&str, bool)];
}
