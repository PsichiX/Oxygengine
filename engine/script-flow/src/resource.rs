use crate::{
    ast::Program,
    vm::{Vm, VmError},
};
use std::collections::HashMap;

#[derive(Default)]
pub struct FlowScriptManager {
    /// {name: (vm, paused)}
    vms: HashMap<String, (Vm, bool)>,
}

impl FlowScriptManager {
    pub fn create_vm(&mut self, name: &str, ast: Program) -> Result<(), VmError> {
        let vm = Vm::new(ast)?;
        self.vms.insert(name.to_owned(), (vm, false));
        Ok(())
    }

    pub fn destroy_vm(&mut self, name: &str) -> bool {
        self.vms.remove(name).is_some()
    }

    pub fn is_paused(&self, name: &str) -> Option<bool> {
        Some(self.vms.get(name)?.1)
    }

    pub fn set_paused(&mut self, name: &str, paused: bool) -> Option<()> {
        self.vms.get_mut(name)?.1 = paused;
        Some(())
    }

    pub fn get(&self, name: &str) -> Option<&Vm> {
        self.vms.get(name).map(|(vm, _)| vm)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut Vm> {
        self.vms.get_mut(name).map(|(vm, _)| vm)
    }

    pub fn vms(&self) -> impl Iterator<Item = &Vm> {
        self.vms.values().map(|(vm, _)| vm)
    }

    pub fn vms_mut(&mut self) -> impl Iterator<Item = &mut Vm> {
        self.vms.values_mut().map(|(vm, _)| vm)
    }

    pub fn process_events(&mut self) -> Result<(), VmError> {
        for (vm, paused) in self.vms.values_mut() {
            if !*paused {
                vm.process_events()?;
            }
        }
        Ok(())
    }
}
