pub mod function;
pub mod node;

use crate::{
    material::{
        common::{
            BakedMaterialShaders, MaterialCompilationState, MaterialCompile, MaterialDataType,
            MaterialShaderType, MaterialSignature, MaterialValue, MaterialValueType,
        },
        graph::{
            function::MaterialFunctionInput,
            node::{
                MaterialGraphInput, MaterialGraphNode, MaterialGraphNodeId, MaterialGraphOperation,
                MaterialGraphOutput, MaterialGraphTransfer,
            },
        },
        MaterialError,
    },
    math::vek::*,
    resources::material_library::MaterialLibrary,
    StringBuffer,
};
use core::Ignite;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Ignite, Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialGraph {
    nodes: HashMap<MaterialGraphNodeId, MaterialGraphNode>,
}

impl MaterialGraph {
    pub fn add_node(&mut self, node: MaterialGraphNode) -> MaterialGraphNodeId {
        let id = MaterialGraphNodeId::new();
        self.nodes.insert(id, node);
        id
    }

    pub fn remove_node(&mut self, id: MaterialGraphNodeId) -> bool {
        if self.nodes.remove(&id).is_some() {
            self.disconnect_output(id);
            true
        } else {
            false
        }
    }

    pub fn node(&self, id: MaterialGraphNodeId) -> Option<&MaterialGraphNode> {
        self.nodes.get(&id)
    }

    pub fn nodes(&self) -> impl Iterator<Item = (MaterialGraphNodeId, &MaterialGraphNode)> {
        self.nodes.iter().map(|(k, v)| (*k, v))
    }

    pub fn inputs(&self) -> impl Iterator<Item = (MaterialGraphNodeId, &MaterialGraphInput)> {
        self.nodes.iter().filter_map(|(k, v)| {
            if let MaterialGraphNode::Input(node) = v {
                Some((*k, node))
            } else {
                None
            }
        })
    }

    pub fn outputs(&self) -> impl Iterator<Item = (MaterialGraphNodeId, &MaterialGraphOutput)> {
        self.nodes.iter().filter_map(|(k, v)| {
            if let MaterialGraphNode::Output(node) = v {
                Some((*k, node))
            } else {
                None
            }
        })
    }

    pub fn connect(
        &mut self,
        from: MaterialGraphNodeId,
        to: MaterialGraphNodeId,
        param: Option<&str>,
    ) -> Result<(), MaterialError> {
        if from == to {
            return Err(MaterialError::CannotConnectNodeToItself(from));
        }
        if !self.nodes.contains_key(&from) {
            return Err(MaterialError::NodeDoesNotExists(from));
        }
        if let Some(node) = self.nodes.get_mut(&to) {
            match node {
                MaterialGraphNode::Operation(node) => {
                    if let Some(param) = param {
                        if let Some(connection) = node.input_connections.get_mut(param) {
                            *connection = from;
                        } else {
                            return Err(MaterialError::InvalidConnectionParam {
                                target: to,
                                name: param.to_owned(),
                            });
                        }
                    } else {
                        return Err(MaterialError::TargetNodeRequiresParamName(to));
                    }
                }
                MaterialGraphNode::Transfer(node) => {
                    node.input_connection = Some(from);
                }
                MaterialGraphNode::Output(node) => {
                    node.input_connection = Some(from);
                }
                _ => {}
            }
            Ok(())
        } else {
            Err(MaterialError::NodeDoesNotExists(to))
        }
    }

    pub fn disconnect_input(
        &mut self,
        id: MaterialGraphNodeId,
        param: Option<&str>,
    ) -> Result<(), MaterialError> {
        if let Some(node) = self.nodes.get_mut(&id) {
            match node {
                MaterialGraphNode::Operation(node) => {
                    if let Some(param) = param {
                        node.input_connections.remove(param);
                    } else {
                        return Err(MaterialError::TargetNodeRequiresParamName(id));
                    }
                }
                MaterialGraphNode::Transfer(node) => {
                    node.input_connection = None;
                }
                MaterialGraphNode::Output(node) => {
                    node.input_connection = None;
                }
                _ => {}
            }
            Ok(())
        } else {
            Err(MaterialError::NodeDoesNotExists(id))
        }
    }

    pub fn disconnect_output(&mut self, id: MaterialGraphNodeId) {
        for node in self.nodes.values_mut() {
            match node {
                MaterialGraphNode::Operation(node) => {
                    let found = node.input_connections.iter().find_map(|(k, v)| {
                        if *v == id {
                            Some(k.to_owned())
                        } else {
                            None
                        }
                    });
                    if let Some(param) = found {
                        node.input_connections.remove(&param);
                    }
                }
                MaterialGraphNode::Transfer(node) => {
                    if node.input_connection == Some(id) {
                        node.input_connection = None;
                    }
                }
                MaterialGraphNode::Output(node) => {
                    if node.input_connection == Some(id) {
                        node.input_connection = None;
                    }
                }
                _ => {}
            }
        }
    }

    pub fn validate(&self, library: &MaterialLibrary) -> Result<(), MaterialError> {
        // does operations point to existing functions?
        for (id, node) in &self.nodes {
            if let MaterialGraphNode::Operation(node) = node {
                if !library.has_function(&node.name) {
                    return Err(MaterialError::FunctionNotFoundInLibrary {
                        node: *id,
                        name: node.name.to_owned(),
                    });
                }
            }
        }
        // does nodes inputs points to existing nodes?
        for (id, node) in &self.nodes {
            match node {
                MaterialGraphNode::Operation(node) => {
                    for from in node.input_connections.values() {
                        if !self.nodes.contains_key(from) {
                            return Err(MaterialError::InvalidConnectionSource {
                                target: *id,
                                source: *from,
                            });
                        }
                    }
                }
                MaterialGraphNode::Transfer(node) => {
                    if let Some(from) = node.input_connection {
                        if !self.nodes.contains_key(&from) {
                            return Err(MaterialError::InvalidConnectionSource {
                                target: *id,
                                source: from,
                            });
                        }
                    }
                }
                MaterialGraphNode::Output(node) => {
                    if let Some(from) = node.input_connection {
                        if !self.nodes.contains_key(&from) {
                            return Err(MaterialError::InvalidConnectionSource {
                                target: *id,
                                source: from,
                            });
                        }
                    }
                }
                _ => {}
            }
        }
        // is graph cyclic?
        {
            let mut cache = Vec::with_capacity(self.nodes.len());
            for (id, node) in &self.nodes {
                if matches!(node, MaterialGraphNode::Output(_)) {
                    cache.clear();
                    if self.is_cyclic_until(node, *id, &mut cache) {
                        return Err(MaterialError::GraphIsCyclic(cache.into_iter().collect()));
                    }
                }
            }
        }
        // are all nodes inputs connected?
        for (id, node) in &self.nodes {
            match node {
                MaterialGraphNode::Operation(node) => {
                    if let Some(function) = library.function(&node.name) {
                        for item in &function.inputs {
                            if !node.input_connections.contains_key(&item.name) {
                                return Err(MaterialError::MissingConnection {
                                    id: *id,
                                    param: Some(item.name.to_owned()),
                                });
                            }
                        }
                    }
                }
                MaterialGraphNode::Transfer(node) => {
                    if node.input_connection.is_none() {
                        return Err(MaterialError::MissingConnection {
                            id: *id,
                            param: None,
                        });
                    }
                }
                MaterialGraphNode::Output(node) => {
                    if node.input_connection.is_none() {
                        return Err(MaterialError::MissingConnection {
                            id: *id,
                            param: None,
                        });
                    }
                }
                _ => {}
            }
        }
        // are input types matching node types?
        for (id, node) in &self.nodes {
            if let MaterialGraphNode::Output(n) = node {
                self.validate_types_output(*id, n, library)?;
            }
        }
        // do transfer nodes exists on the path from inputs to outputs?
        {
            let inputs = self
                .nodes
                .iter()
                .filter_map(|(id, node)| {
                    if let MaterialGraphNode::Input(n) = node {
                        if n.is_vertex_input() {
                            return Some(*id);
                        }
                    }
                    None
                })
                .collect::<Vec<_>>();
            let outputs = self.nodes.iter().filter_map(|(id, node)| {
                if let MaterialGraphNode::Output(n) = node {
                    if n.is_fragment_output() {
                        return Some((id, node));
                    }
                }
                None
            });
            for (id, node) in outputs {
                for target in &inputs {
                    self.validate_transfer(*id, node, *target)?;
                }
            }
        }
        Ok(())
    }

    pub fn subgraph(&self, target_outputs: &HashSet<String>) -> Option<Self> {
        let graph_outputs = self.get_fragment_outputs();
        if !target_outputs.is_subset(&graph_outputs) {
            return None;
        }
        let mut visited = HashSet::with_capacity(self.nodes.len());
        for (id, node) in &self.nodes {
            if let MaterialGraphNode::Output(n) = node {
                if n.shader_type == MaterialShaderType::Vertex {
                    self.visit(node, *id, &mut visited);
                }
            }
        }
        for (id, node) in &self.nodes {
            if let MaterialGraphNode::Output(n) = node {
                if n.is_fragment_output() && target_outputs.contains(&n.name) {
                    self.visit(node, *id, &mut visited);
                }
            }
        }
        Some(Self {
            nodes: visited
                .into_iter()
                .map(|id| (id, self.nodes.get(&id).unwrap().clone()))
                .collect(),
        })
    }

    pub fn combine(&self, other: &Self) -> Self {
        let inputs_names = self
            .nodes
            .iter()
            .chain(other.nodes.iter())
            .filter_map(|(id, node)| {
                if let MaterialGraphNode::Input(node) = node {
                    if node.data_type == MaterialDataType::Domain {
                        return Some((*id, node.name.to_owned()));
                    }
                }
                None
            })
            .collect::<HashMap<_, _>>();
        let named_outputs = self
            .nodes
            .values()
            .chain(other.nodes.values())
            .filter_map(|node| {
                if let MaterialGraphNode::Output(node) = node {
                    if node.data_type == MaterialDataType::Domain {
                        if let Some(from) = node.input_connection {
                            return Some((node.name.to_owned(), from));
                        }
                    }
                }
                None
            })
            .collect::<HashMap<_, _>>();
        let default_inputs = self
            .nodes
            .values()
            .chain(other.nodes.values())
            .filter_map(|node| {
                if let MaterialGraphNode::Input(node) = node {
                    if node.data_type == MaterialDataType::Domain {
                        if let Some(value) = &node.default_value {
                            let nid = MaterialGraphNodeId::new();
                            let n = MaterialGraphNode::Value(value.clone());
                            return Some((node.name.to_owned(), (nid, n)));
                        }
                    }
                }
                None
            })
            .collect::<HashMap<_, _>>();
        let mut nodes = self
            .nodes
            .iter()
            .filter_map(|(id, node)| {
                if node.is_domain() {
                    return None;
                }
                Some((*id, node.clone()))
            })
            .chain(other.nodes.iter().filter_map(|(id, node)| {
                if node.is_domain() {
                    return None;
                }
                Some((*id, node.clone()))
            }))
            .collect::<HashMap<_, _>>();
        for node in nodes.values_mut() {
            match node {
                MaterialGraphNode::Operation(node) => {
                    for from in node.input_connections.values_mut() {
                        if let Some(name) = inputs_names.get(from) {
                            if let Some(id) = named_outputs.get(name) {
                                *from = *id;
                            } else if let Some((id, _)) = default_inputs.get(name) {
                                *from = *id;
                            }
                        }
                    }
                }
                MaterialGraphNode::Transfer(node) => {
                    if let Some(from) = &mut node.input_connection {
                        if let Some(name) = inputs_names.get(from) {
                            if let Some(id) = named_outputs.get(name) {
                                *from = *id;
                            } else if let Some((id, _)) = default_inputs.get(name) {
                                *from = *id;
                            }
                        }
                    }
                }
                MaterialGraphNode::Output(node) => {
                    if let Some(from) = &mut node.input_connection {
                        if let Some(name) = inputs_names.get(from) {
                            if let Some(id) = named_outputs.get(name) {
                                *from = *id;
                            } else if let Some((id, _)) = default_inputs.get(name) {
                                *from = *id;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        nodes.extend(default_inputs.into_iter().map(|(_, (id, node))| (id, node)));
        Self { nodes }
    }

    pub fn bake(
        &self,
        signature: &MaterialSignature,
        domain: Option<&Self>,
        library: &MaterialLibrary,
        fragment_high_precision_support: bool,
    ) -> Result<Option<BakedMaterialShaders>, MaterialError> {
        self.prevalidate()?;
        let graph = if let Some(domain) = domain {
            domain.prevalidate()?;
            self.combine(domain)
        } else {
            self.clone()
        };
        let targets = signature.targets().map(|(id, _)| id.to_owned()).collect();
        let graph = match graph.subgraph(&targets) {
            Some(graph) => graph,
            None => {
                return Err(MaterialError::CouldNotBuildSubgraphForSignature(
                    signature.to_owned(),
                ))
            }
        };
        let sources = signature
            .sources()
            .map(|(id, _)| id.to_owned())
            .collect::<HashSet<_>>();
        let inputs = graph
            .inputs()
            .filter_map(|(_, input)| {
                if input.is_vertex_input_attribute() && input.default_value.is_none() {
                    Some(input.name.to_owned())
                } else {
                    None
                }
            })
            .collect::<HashSet<_>>();
        if !inputs.is_subset(&sources) {
            return Err(MaterialError::SubgraphInputsDoesNotMatchSignature(
                inputs,
                signature.to_owned(),
            ));
        }
        if let Err(error) = graph.validate(library) {
            return Err(MaterialError::Baking(graph, Box::new(error)));
        }
        let vertex = {
            let vertex = graph.compile(MaterialCompilationState::Main {
                shader_type: MaterialShaderType::Vertex,
                signature,
                library,
                fragment_high_precision_support,
            });
            match vertex {
                Ok(vertex) => vertex,
                Err(error) => {
                    return Err(MaterialError::CouldNotCompileVertexShader(
                        error.to_string(),
                    ))
                }
            }
        };
        let fragment = {
            let fragment = graph.compile(MaterialCompilationState::Main {
                shader_type: MaterialShaderType::Fragment,
                signature,
                library,
                fragment_high_precision_support,
            });
            match fragment {
                Ok(fragment) => fragment,
                Err(error) => {
                    return Err(MaterialError::CouldNotCompileFragmentShader(
                        error.to_string(),
                    ))
                }
            }
        };
        let uniforms = graph
            .inputs()
            .filter_map(|(_, input)| {
                if input.data_type == MaterialDataType::Uniform {
                    Some((input.name.to_owned(), input.value_type.to_owned()))
                } else {
                    None
                }
            })
            .collect();
        let samplers = graph
            .inputs()
            .filter_map(|(_, input)| {
                if input.data_type == MaterialDataType::Uniform
                    && matches!(
                        input.value_type,
                        MaterialValueType::Sampler2d
                            | MaterialValueType::Sampler2dArray
                            | MaterialValueType::Sampler3d
                    )
                {
                    Some(input.name.to_owned())
                } else {
                    None
                }
            })
            .collect();

        Ok(Some(BakedMaterialShaders {
            vertex,
            fragment,
            uniforms,
            samplers,
        }))
    }

    fn get_fragment_outputs(&self) -> HashSet<String> {
        self.nodes
            .values()
            .filter_map(|node| {
                if let MaterialGraphNode::Output(node) = node {
                    if node.is_fragment_output() {
                        return Some(node.name.to_owned());
                    }
                }
                None
            })
            .collect()
    }

    fn prevalidate(&self) -> Result<(), MaterialError> {
        for (id, node) in &self.nodes {
            match node {
                MaterialGraphNode::Input(node) => {
                    if !node.name.starts_with(char::is_alphanumeric) {
                        return Err(MaterialError::InvalidName {
                            node: *id,
                            name: node.name.to_owned(),
                        });
                    }
                    if node.data_type == MaterialDataType::Domain && node.default_value.is_none() {
                        return Err(MaterialError::DomainInputHasNoDefaultValue {
                            node: *id,
                            name: node.name.to_owned(),
                        });
                    }
                }
                MaterialGraphNode::Output(node) => {
                    if !node.name.starts_with(char::is_alphanumeric) {
                        return Err(MaterialError::InvalidName {
                            node: *id,
                            name: node.name.to_owned(),
                        });
                    }
                }
                MaterialGraphNode::Operation(node) => {
                    if !node.name.starts_with(char::is_alphanumeric) {
                        return Err(MaterialError::InvalidName {
                            node: *id,
                            name: node.name.to_owned(),
                        });
                    }
                    for name in node.input_connections.keys() {
                        if !name.starts_with(char::is_alphanumeric) {
                            return Err(MaterialError::InvalidName {
                                node: *id,
                                name: name.to_owned(),
                            });
                        }
                    }
                }
                MaterialGraphNode::Transfer(node) => {
                    if !node.name.starts_with(char::is_alphanumeric) {
                        return Err(MaterialError::InvalidName {
                            node: *id,
                            name: node.name.to_owned(),
                        });
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn validate_types_output(
        &self,
        id: MaterialGraphNodeId,
        node: &MaterialGraphOutput,
        library: &MaterialLibrary,
    ) -> Result<(), MaterialError> {
        let value_type = &node.value_type;
        let input_id = node.input_connection.unwrap();
        let node = self.nodes.get(&input_id).unwrap();
        match node {
            MaterialGraphNode::Value(v) => {
                self.validate_types_value(id, input_id, v, value_type)?;
            }
            MaterialGraphNode::Input(n) => {
                self.validate_types_input(id, input_id, n, value_type)?;
            }
            MaterialGraphNode::Operation(n) => {
                self.validate_types_operation(id, input_id, n, value_type, library)?;
            }
            MaterialGraphNode::Transfer(n) => {
                self.validate_types_transfer(input_id, n, value_type, library)?;
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    fn validate_types_value(
        &self,
        target_id: MaterialGraphNodeId,
        id: MaterialGraphNodeId,
        value: &MaterialValue,
        value_type: &MaterialValueType,
    ) -> Result<(), MaterialError> {
        let t = value.value_type();
        if &t != value_type {
            return Err(MaterialError::MismatchingConnectionTypes {
                from: id,
                from_value_type: Some(t),
                to: target_id,
                to_value_type: Some(value_type.to_owned()),
                param: None,
            });
        }
        Ok(())
    }

    fn validate_types_input(
        &self,
        target_id: MaterialGraphNodeId,
        id: MaterialGraphNodeId,
        node: &MaterialGraphInput,
        value_type: &MaterialValueType,
    ) -> Result<(), MaterialError> {
        if &node.value_type != value_type {
            return Err(MaterialError::MismatchingConnectionTypes {
                from: id,
                from_value_type: Some(node.value_type.to_owned()),
                to: target_id,
                to_value_type: Some(value_type.to_owned()),
                param: None,
            });
        }
        Ok(())
    }

    fn validate_types_operation(
        &self,
        target_id: MaterialGraphNodeId,
        id: MaterialGraphNodeId,
        node: &MaterialGraphOperation,
        value_type: &MaterialValueType,
        library: &MaterialLibrary,
    ) -> Result<(), MaterialError> {
        let function = library.function(&node.name).unwrap();
        if &function.output != value_type {
            return Err(MaterialError::MismatchingConnectionTypes {
                from: id,
                from_value_type: Some(function.output.to_owned()),
                to: target_id,
                to_value_type: Some(value_type.to_owned()),
                param: None,
            });
        }
        for input in &function.inputs {
            let from = *node.input_connections.get(&input.name).unwrap();
            self.validate_types_operation_input(id, from, input, library)?;
        }
        Ok(())
    }

    fn validate_types_operation_input(
        &self,
        operation_id: MaterialGraphNodeId,
        param_id: MaterialGraphNodeId,
        input: &MaterialFunctionInput,
        library: &MaterialLibrary,
    ) -> Result<(), MaterialError> {
        let value_type = &input.value_type;
        let node = self.nodes.get(&param_id).unwrap();
        match node {
            MaterialGraphNode::Value(v) => {
                self.validate_types_value(operation_id, param_id, v, value_type)?;
            }
            MaterialGraphNode::Input(n) => {
                self.validate_types_input(operation_id, param_id, n, value_type)?;
            }
            MaterialGraphNode::Operation(n) => {
                self.validate_types_operation(operation_id, param_id, n, value_type, library)?;
            }
            MaterialGraphNode::Transfer(n) => {
                self.validate_types_transfer(param_id, n, value_type, library)?;
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    fn validate_types_transfer(
        &self,
        id: MaterialGraphNodeId,
        node: &MaterialGraphTransfer,
        value_type: &MaterialValueType,
        library: &MaterialLibrary,
    ) -> Result<(), MaterialError> {
        let from = node.input_connection.unwrap();
        let node = self.nodes.get(&id).unwrap();
        match node {
            MaterialGraphNode::Value(v) => {
                self.validate_types_value(id, from, v, value_type)?;
            }
            MaterialGraphNode::Input(n) => {
                self.validate_types_input(id, from, n, value_type)?;
            }
            MaterialGraphNode::Operation(n) => {
                self.validate_types_operation(id, from, n, value_type, library)?;
            }
            MaterialGraphNode::Transfer(n) => {
                self.validate_types_transfer(from, n, value_type, library)?;
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    fn validate_transfer(
        &self,
        id: MaterialGraphNodeId,
        node: &MaterialGraphNode,
        target: MaterialGraphNodeId,
    ) -> Result<(), MaterialError> {
        let inputs = node.inputs();
        if inputs.is_empty() {
            return if id == target {
                Err(MaterialError::NoTransferFound(target))
            } else {
                Ok(())
            };
        }
        for (id, node) in inputs
            .into_iter()
            .map(|id| (id, self.nodes.get(&target).unwrap()))
        {
            if matches!(
                node,
                MaterialGraphNode::Value(_) | MaterialGraphNode::Transfer(_)
            ) {
                return Ok(());
            }
            self.validate_transfer(id, node, target)?;
        }
        Ok(())
    }

    fn visit(
        &self,
        node: &MaterialGraphNode,
        id: MaterialGraphNodeId,
        visited: &mut HashSet<MaterialGraphNodeId>,
    ) {
        visited.insert(id);
        let inputs = node.inputs();
        if inputs.is_empty() {
            return;
        }
        let iter = inputs.into_iter().map(|id| {
            let node = self
                .nodes
                .get(&id)
                .unwrap_or_else(|| panic!("Node does not exists: {:?}", id));
            (id, node)
        });
        for (id, node) in iter {
            if visited.contains(&id) {
                continue;
            }
            self.visit(node, id, visited);
        }
    }

    fn is_cyclic_until(
        &self,
        node: &MaterialGraphNode,
        id: MaterialGraphNodeId,
        cache: &mut Vec<MaterialGraphNodeId>,
    ) -> bool {
        cache.push(id);
        let inputs = node.inputs();
        if inputs.is_empty() {
            cache.pop();
            return false;
        }
        for from in &inputs {
            if cache.contains(from) {
                return true;
            }
        }
        for (id, node) in inputs
            .into_iter()
            .map(|id| (id, self.nodes.get(&id).unwrap()))
        {
            if self.is_cyclic_until(node, id, cache) {
                return true;
            }
        }
        cache.pop();
        false
    }

    fn compile_inputs_outputs(
        &self,
        output: &mut StringBuffer,
        shader_type: MaterialShaderType,
        signature: &MaterialSignature,
        library: &MaterialLibrary,
        fragment_high_precision_support: bool,
    ) -> std::io::Result<()> {
        let mut transfers = HashMap::new();
        for node in self.nodes.values() {
            match node {
                MaterialGraphNode::Input(n) => {
                    if n.shader_type == shader_type {
                        match n.data_type {
                            MaterialDataType::Attribute => {
                                if let Some((_, location)) =
                                    signature.sources().find(|(id, _)| *id == n.name)
                                {
                                    output.write_str("layout(location = ")?;
                                    output.write_str(location.to_string())?;
                                    output.write_str(") in ")?;
                                    output.write_str(n.value_type.to_string())?;
                                    output.write_space()?;
                                    output.write_str(&n.name)?;
                                } else if let Some(default_value) = &n.default_value {
                                    output.write_str("const ")?;
                                    output.write_str(
                                        n.data_precision
                                            .ensure(fragment_high_precision_support)
                                            .to_string(),
                                    )?;
                                    output.write_space()?;
                                    output.write_str(n.value_type.to_string())?;
                                    output.write_space()?;
                                    output.write_str(&n.name)?;
                                    output.write_str(" = ")?;
                                    output.write_str(default_value.to_string())?;
                                } else {
                                    output.write_str("in ")?;
                                    output.write_str(
                                        n.data_precision
                                            .ensure(fragment_high_precision_support)
                                            .to_string(),
                                    )?;
                                    output.write_space()?;
                                    output.write_str(n.value_type.to_string())?;
                                    output.write_space()?;
                                    output.write_str(&n.name)?;
                                }
                                output.write_str(";\n")?;
                            }
                            MaterialDataType::Uniform => {
                                if let Some(default_value) = &n.default_value {
                                    output.write_str("const ")?;
                                    output.write_str(
                                        n.data_precision
                                            .ensure(fragment_high_precision_support)
                                            .to_string(),
                                    )?;
                                    output.write_space()?;
                                    output.write_str(n.value_type.to_string())?;
                                    output.write_space()?;
                                    output.write_str(&n.name)?;
                                    output.write_str(" = ")?;
                                    output.write_str(default_value.to_string())?;
                                } else {
                                    output.write_str("uniform ")?;
                                    output.write_str(
                                        n.data_precision
                                            .ensure(fragment_high_precision_support)
                                            .to_string(),
                                    )?;
                                    output.write_space()?;
                                    output.write_str(n.value_type.to_string())?;
                                    output.write_space()?;
                                    output.write_str(&n.name)?;
                                }
                                output.write_str(";\n")?;
                            }
                            _ => {}
                        }
                    }
                }
                MaterialGraphNode::Output(n) => {
                    if n.shader_type == shader_type {
                        if let MaterialDataType::BufferOutput = n.data_type {
                            if let Some((_, location)) =
                                signature.targets().find(|(id, _)| *id == n.name)
                            {
                                output.write_str("layout(location = ")?;
                                output.write_str(location.to_string())?;
                                output.write_str(") out ")?;
                            } else {
                                output.write_str("out ")?;
                            }
                            output.write_str(n.value_type.to_string())?;
                            output.write_space()?;
                            output.write_str(&n.name)?;
                            output.write_str(";\n")?;
                        }
                    }
                    self.collect_transfer_types(node, &n.value_type, library, &mut transfers);
                }
                _ => {}
            }
        }
        for (name, value_type) in transfers {
            match shader_type {
                MaterialShaderType::Vertex => output.write_str("out ")?,
                MaterialShaderType::Fragment => output.write_str("in ")?,
                MaterialShaderType::Undefined => unreachable!(),
            }
            output.write_str(value_type)?;
            output.write_space()?;
            output.write_str(name)?;
            output.write_str(";\n")?;
        }
        Ok(())
    }

    fn collect_transfer_types(
        &self,
        node: &MaterialGraphNode,
        value_type: &MaterialValueType,
        library: &MaterialLibrary,
        output: &mut HashMap<String, String>,
    ) {
        match node {
            MaterialGraphNode::Operation(n) => {
                if let Some(function) = library.function(&n.name) {
                    for (name, from) in &n.input_connections {
                        if let Some(input) =
                            function.inputs.iter().find(|input| &input.name == name)
                        {
                            let node = self.nodes.get(from).unwrap();
                            self.collect_transfer_types(node, &input.value_type, library, output);
                        }
                    }
                }
            }
            MaterialGraphNode::Transfer(n) => {
                output.insert(n.name.to_owned(), value_type.to_string());
            }
            MaterialGraphNode::Output(n) => {
                if let Some(from) = n.input_connection {
                    let node = self.nodes.get(&from).unwrap();
                    self.collect_transfer_types(node, &n.value_type, library, output);
                }
            }
            _ => {}
        }
    }

    fn compile_functions(
        &self,
        output: &mut StringBuffer,
        shader_type: MaterialShaderType,
        library: &MaterialLibrary,
    ) -> std::io::Result<()> {
        let mut functions = HashSet::with_capacity(self.nodes.len());
        for node in self.nodes.values() {
            match node {
                MaterialGraphNode::Transfer(node) => {
                    if shader_type == MaterialShaderType::Vertex {
                        if let Some(from) = node.input_connection {
                            self.collect_functions(from, shader_type, &mut functions);
                        }
                    }
                }
                MaterialGraphNode::Output(node) => {
                    if node.shader_type == shader_type {
                        if let Some(from) = node.input_connection {
                            self.collect_functions(from, shader_type, &mut functions);
                        }
                    }
                }
                _ => {}
            }
        }
        for name in &functions {
            if let Some(function) = library.function(name) {
                if function.can_be_compiled() {
                    function.compile_to(output, MaterialCompilationState::FunctionDeclaration)?;
                }
            }
        }
        for name in &functions {
            if let Some(function) = library.function(name) {
                if function.can_be_compiled() {
                    output.write_new_line()?;
                    function.compile_to(output, MaterialCompilationState::FunctionDefinition)?;
                }
            }
        }
        Ok(())
    }

    fn collect_functions(
        &self,
        id: MaterialGraphNodeId,
        shader_type: MaterialShaderType,
        output: &mut HashSet<String>,
    ) {
        if let Some(node) = self.nodes.get(&id) {
            match node {
                MaterialGraphNode::Operation(node) => {
                    output.insert(node.name.to_owned());
                }
                MaterialGraphNode::Transfer(_) => {
                    if shader_type == MaterialShaderType::Fragment {
                        return;
                    }
                }
                _ => {}
            }
            for from in node.inputs() {
                self.collect_functions(from, shader_type, output);
            }
        }
    }

    fn compile_graph_node(
        &self,
        id: MaterialGraphNodeId,
        node: &MaterialGraphNode,
        shader_type: MaterialShaderType,
        library: &MaterialLibrary,
        output: &mut StringBuffer,
        symbols: &mut HashMap<MaterialGraphNodeId, String>,
    ) -> std::io::Result<()> {
        if symbols.contains_key(&id) {
            return Ok(());
        }
        match node {
            MaterialGraphNode::Value(node) => {
                let count = symbols.len();
                symbols
                    .entry(id)
                    .or_insert_with(|| format!("_node{}", count));
                output.write_str(node.value_type().to_string())?;
                output.write_space()?;
                output.write_str(symbols.get(&id).unwrap())?;
                output.write_str(" = ")?;
                output.write_str(node.to_string())?;
                output.write_str(";\n")?;
            }
            MaterialGraphNode::Input(node) => {
                symbols.entry(id).or_insert_with(|| node.name.to_owned());
            }
            MaterialGraphNode::Operation(node) => {
                let function = library.function(&node.name).unwrap();
                for input in &function.inputs {
                    let from = node.input_connections.get(&input.name).unwrap();
                    let n = self.nodes.get(from).unwrap();
                    self.compile_graph_node(*from, n, shader_type, library, output, symbols)?;
                }
                let count = symbols.len();
                symbols
                    .entry(id)
                    .or_insert_with(|| format!("_node{}", count));
                output.write_str(function.output.to_string())?;
                output.write_space()?;
                output.write_str(symbols.get(&id).unwrap())?;
                output.write_str(" = ")?;
                output.write_str(function.call_name().to_owned())?;
                output.write_str("(")?;
                for (index, input) in function.inputs.iter().enumerate() {
                    if index > 0 {
                        output.write_str(", ")?;
                    }
                    let from = node.input_connections.get(&input.name).unwrap();
                    output.write_str(symbols.get(from).unwrap())?;
                }
                output.write_str(");\n")?;
            }
            MaterialGraphNode::Transfer(node) => {
                let from = node.input_connection.unwrap();
                let n = self.nodes.get(&from).unwrap();
                self.compile_graph_node(from, n, shader_type, library, output, symbols)?;
                symbols.entry(id).or_insert_with(|| node.name.to_owned());
                if shader_type == MaterialShaderType::Vertex {
                    output.write_str(&node.name)?;
                    output.write_str(" = ")?;
                    output.write_str(symbols.get(&from).unwrap())?;
                    output.write_str(";\n")?;
                }
            }
            MaterialGraphNode::Output(node) => {
                let from = node.input_connection.unwrap();
                let n = self.nodes.get(&from).unwrap();
                self.compile_graph_node(from, n, shader_type, library, output, symbols)?;
                output.write_str(&node.name)?;
                output.write_str(" = ")?;
                output.write_str(symbols.get(&from).unwrap())?;
                output.write_str(";\n")?;
            }
        }
        Ok(())
    }
}

impl MaterialCompile<StringBuffer, String, MaterialCompilationState<'_>> for MaterialGraph {
    fn compile_to(
        &self,
        output: &mut StringBuffer,
        state: MaterialCompilationState,
    ) -> std::io::Result<()> {
        match state {
            MaterialCompilationState::Main {
                shader_type,
                signature,
                library,
                fragment_high_precision_support,
            } => {
                output.write_str("#version 300 es\n")?;
                match shader_type {
                    MaterialShaderType::Vertex => {
                        output.write_str("precision highp float;\n")?;
                    }
                    MaterialShaderType::Fragment => {
                        if fragment_high_precision_support {
                            output.write_str("precision highp float;\n")?;
                        } else {
                            output.write_str("precision mediump float;\n")?;
                        }
                    }
                    _ => unreachable!(),
                }
                output.write_str("precision lowp sampler2DArray;\n")?;
                output.write_str("precision lowp sampler3D;\n")?;
                self.compile_inputs_outputs(
                    output,
                    shader_type,
                    signature,
                    library,
                    fragment_high_precision_support,
                )?;
                output.write_new_line()?;
                self.compile_functions(output, shader_type, library)?;
                output.write_new_line()?;
                output.write_str("void main() {\n")?;
                self.compile_to(
                    output,
                    MaterialCompilationState::GraphBody {
                        shader_type,
                        library,
                    },
                )?;
                output.write_str("}\n")?;
            }
            MaterialCompilationState::GraphBody {
                shader_type,
                library,
            } => {
                let mut symbols = HashMap::with_capacity(self.nodes.len());
                for (id, node) in &self.nodes {
                    match node {
                        MaterialGraphNode::Transfer(_) => {
                            if shader_type == MaterialShaderType::Vertex {
                                self.compile_graph_node(
                                    *id,
                                    node,
                                    shader_type,
                                    library,
                                    output,
                                    &mut symbols,
                                )?;
                            }
                        }
                        MaterialGraphNode::Output(n) => {
                            if n.shader_type == shader_type {
                                self.compile_graph_node(
                                    *id,
                                    node,
                                    shader_type,
                                    library,
                                    output,
                                    &mut symbols,
                                )?;
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}
