#![cfg(test)]

use super::{
    ast::Program,
    vm::{Reference, Value, Vm, VmOperation, VmOperationError},
};
use crate::core::prefab::Prefab;

#[test]
fn test_hello_flow() {
    struct Print;

    impl VmOperation for Print {
        fn execute(&mut self, inputs: &[Reference]) -> Result<Vec<Reference>, VmOperationError> {
            println!("=== EXECUTE PRINT | inputs: {:?}", inputs);
            Ok(vec![])
        }
    }

    let content =
        std::fs::read_to_string("scripts/hello_world.yaml").expect("Could not read graph");
    let ast = Program::from_prefab_str(&content).expect("Could not deserialize script");
    println!("{:#?}", ast);
    let mut vm = Vm::new(ast).expect("Could not create VM");
    vm.register_operation("Print", Print);
    assert!(vm.run_event("onEnter", vec![]).is_err());
    vm.run_event(
        "onRun",
        vec![
            Value::String("Hello, World!".to_owned()).into(),
            Value::Number(42.into()).into(),
        ],
    )
    .expect("Could not run event");
    vm.process_events().expect("Could not process events");
    let completed = vm.get_completed_events().collect::<Vec<_>>();
    assert_eq!(completed.len(), 1);
    assert_eq!(completed[0].1.len(), 1);
    let r: Reference = Value::Number((-1).into()).into();
    assert_eq!(completed[0].1[0], r);
}
