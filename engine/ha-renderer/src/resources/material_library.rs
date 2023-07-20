use crate::{
    builtin_material_functions, code_material_function, code_material_functions,
    graph_material_function,
    material::{
        common::{BakedMaterialShaders, MaterialSignature},
        graph::{
            function::{MaterialFunction, MaterialFunctionContent},
            MaterialGraph,
        },
        MaterialError,
    },
    material_graph,
    math::*,
    mesh::VertexLayout,
    render_target::{RenderTarget, RenderTargetDescriptor},
};
use std::collections::HashMap;

#[derive(Debug)]
pub struct MaterialLibraryInfo {
    pub domains: Vec<String>,
    pub middlewares: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct MaterialLibrary {
    functions: HashMap<String, MaterialFunction>,
    domains: HashMap<String, MaterialGraph>,
    middlewares: HashMap<String, MaterialGraph>,
}

impl MaterialLibrary {
    pub fn info(&self) -> MaterialLibraryInfo {
        MaterialLibraryInfo {
            domains: self.domains.keys().cloned().collect(),
            middlewares: self.middlewares.keys().cloned().collect(),
        }
    }

    pub fn add_function(&mut self, mut function: MaterialFunction) {
        if let MaterialFunctionContent::Graph(graph) = &mut function.content {
            graph.optimize();
        }
        self.functions.insert(function.name.to_owned(), function);
    }

    pub fn with_function(mut self, function: MaterialFunction) -> Self {
        self.add_function(function);
        self
    }

    pub fn add_functions(&mut self, functions: impl IntoIterator<Item = MaterialFunction>) {
        for function in functions {
            self.add_function(function);
        }
    }

    pub fn with_functions(mut self, functions: impl IntoIterator<Item = MaterialFunction>) -> Self {
        self.add_functions(functions);
        self
    }

    pub fn remove_function(&mut self, name: &str) -> Option<MaterialFunction> {
        self.functions.remove(name)
    }

    pub fn has_function(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    pub fn function(&self, name: &str) -> Option<&MaterialFunction> {
        self.functions.get(name)
    }

    pub fn functions_count(&self) -> usize {
        self.functions.len()
    }

    pub fn add_domain(&mut self, name: String, mut graph: MaterialGraph) {
        graph.optimize();
        self.domains.insert(name, graph);
    }

    pub fn with_domain(mut self, name: String, graph: MaterialGraph) -> Self {
        self.add_domain(name, graph);
        self
    }

    pub fn remove_domain(&mut self, name: &str) {
        self.domains.remove(name);
    }

    pub fn has_domain(&self, name: &str) -> bool {
        self.domains.contains_key(name)
    }

    pub fn domain(&self, name: &str) -> Option<&MaterialGraph> {
        self.domains.get(name)
    }

    pub fn domains_count(&self) -> usize {
        self.domains.len()
    }

    pub fn add_middleware(&mut self, name: String, mut graph: MaterialGraph) {
        graph.optimize();
        self.middlewares.insert(name, graph);
    }

    pub fn with_middleware(mut self, name: String, graph: MaterialGraph) -> Self {
        self.add_middleware(name, graph);
        self
    }

    pub fn remove_middleware(&mut self, name: &str) {
        self.middlewares.remove(name);
    }

    pub fn has_middleware(&self, name: &str) -> bool {
        self.middlewares.contains_key(name)
    }

    pub fn middleware(&self, name: &str) -> Option<&MaterialGraph> {
        self.middlewares.get(name)
    }

    pub fn middlewares_count(&self) -> usize {
        self.middlewares.len()
    }

    pub fn validate_material_compilation(
        vertex_layout: &VertexLayout,
        render_target: RenderTargetDescriptor,
        domain: &MaterialGraph,
        graph: &MaterialGraph,
    ) -> Result<Option<BakedMaterialShaders>, MaterialError> {
        let render_target = match render_target {
            RenderTargetDescriptor::Main => match RenderTarget::main() {
                Ok(render_target) => render_target,
                Err(error) => {
                    return Err(MaterialError::CouldNotCreateRenderTarget(Box::new(error)))
                }
            },
            RenderTargetDescriptor::Custom {
                buffers,
                width,
                height,
            } => RenderTarget::new(buffers, width, height),
        };
        let signature = MaterialSignature::from_objects(
            vertex_layout,
            &render_target,
            None,
            vertex_layout.middlewares().into(),
        );
        graph.bake(&signature, Some(domain), &Self::default(), true)
    }

    pub fn assert_material_compilation(
        vertex_layout: &VertexLayout,
        render_target: RenderTargetDescriptor,
        domain: &MaterialGraph,
        graph: &MaterialGraph,
    ) {
        let baked =
            Self::validate_material_compilation(vertex_layout, render_target, domain, graph)
                .unwrap_or_else(|error| match &error {
                    MaterialError::Baking(graph, error) => match &**error {
                        MaterialError::GraphIsCyclic(nodes) => {
                            let nodes = nodes
                                .iter()
                                .map(|id| (id, graph.node(*id).unwrap()))
                                .collect::<Vec<_>>();
                            panic!(
                                "Could not bake shaders from material: {:?} | Cycle: {:#?}",
                                error, nodes
                            );
                        }
                        _ => panic!("Could not bake shaders from material: {:?}", error),
                    },
                    _ => panic!("Could not bake shaders from material: {:?}", error),
                })
                .expect("Baked shaders are empty");
        println!("* compiled vertex material graph text:\n{}", baked.vertex);
        println!(
            "* compiled fragment material graph text:\n{}",
            baked.fragment
        );
    }

    fn with_angle_functions(mut self) -> Self {
        self.add_functions(builtin_material_functions! {
            {fn radians_float(v: float) -> float}
            {fn radians_vec2(v: vec2) -> vec2}
            {fn radians_vec3(v: vec3) -> vec3}
            {fn radians_vec4(v: vec4) -> vec4}
            { "radians" }
        });
        self.add_functions(builtin_material_functions! {
            {fn degrees_float(v: float) -> float}
            {fn degrees_vec2(v: vec2) -> vec2}
            {fn degrees_vec3(v: vec3) -> vec3}
            {fn degrees_vec4(v: vec4) -> vec4}
            { "degrees" }
        });
        self.add_functions(builtin_material_functions! {
            {fn sin_float(v: float) -> float}
            {fn sin_vec2(v: vec2) -> vec2}
            {fn sin_vec3(v: vec3) -> vec3}
            {fn sin_vec4(v: vec4) -> vec4}
            { "sin" }
        });
        self.add_functions(builtin_material_functions! {
            {fn cos_float(v: float) -> float}
            {fn cos_vec2(v: vec2) -> vec2}
            {fn cos_vec3(v: vec3) -> vec3}
            {fn cos_vec4(v: vec4) -> vec4}
            { "cos" }
        });
        self.add_functions(builtin_material_functions! {
            {fn tan_float(v: float) -> float}
            {fn tan_vec2(v: vec2) -> vec2}
            {fn tan_vec3(v: vec3) -> vec3}
            {fn tan_vec4(v: vec4) -> vec4}
            { "tan" }
        });
        self.add_functions(builtin_material_functions! {
            {fn asin_float(v: float) -> float}
            {fn asin_vec2(v: vec2) -> vec2}
            {fn asin_vec3(v: vec3) -> vec3}
            {fn asin_vec4(v: vec4) -> vec4}
            { "asin" }
        });
        self.add_functions(builtin_material_functions! {
            {fn acos_float(v: float) -> float}
            {fn acos_vec2(v: vec2) -> vec2}
            {fn acos_vec3(v: vec3) -> vec3}
            {fn acos_vec4(v: vec4) -> vec4}
            { "acos" }
        });
        self.add_functions(builtin_material_functions! {
            {fn atan_float(v: float) -> float}
            {fn atan_vec2(v: vec2) -> vec2}
            {fn atan_vec3(v: vec3) -> vec3}
            {fn atan_vec4(v: vec4) -> vec4}
            { "atan" }
        });
        self
    }

    fn with_single_functions(mut self) -> Self {
        self.add_functions(builtin_material_functions! {
            {fn pow_float(x: float, y: float) -> float}
            {fn pow_vec2(x: vec2, y: vec2) -> vec2}
            {fn pow_vec3(x: vec3, y: vec3) -> vec3}
            {fn pow_vec4(x: vec4, y: vec4) -> vec4}
            { "pow" }
        });
        self.add_functions(builtin_material_functions! {
            {fn exp_float(v: float) -> float}
            {fn exp_vec2(v: vec2) -> vec2}
            {fn exp_vec3(v: vec3) -> vec3}
            {fn exp_vec4(v: vec4) -> vec4}
            { "exp" }
        });
        self.add_functions(builtin_material_functions! {
            {fn log_float(v: float) -> float}
            {fn log_vec2(v: vec2) -> vec2}
            {fn log_vec3(v: vec3) -> vec3}
            {fn log_vec4(v: vec4) -> vec4}
            { "log" }
        });
        self.add_functions(builtin_material_functions! {
            {fn exp2_float(v: float) -> float}
            {fn exp2_vec2(v: vec2) -> vec2}
            {fn exp2_vec3(v: vec3) -> vec3}
            {fn exp2_vec4(v: vec4) -> vec4}
            { "exp2" }
        });
        self.add_functions(builtin_material_functions! {
            {fn log2_float(v: float) -> float}
            {fn log2_vec2(v: vec2) -> vec2}
            {fn log2_vec3(v: vec3) -> vec3}
            {fn log2_vec4(v: vec4) -> vec4}
            { "log2" }
        });
        self.add_functions(builtin_material_functions! {
            {fn sqrt_float(v: float) -> float}
            {fn sqrt_vec2(v: vec2) -> vec2}
            {fn sqrt_vec3(v: vec3) -> vec3}
            {fn sqrt_vec4(v: vec4) -> vec4}
            { "sqrt" }
        });
        self.add_functions(builtin_material_functions! {
            {fn inversesqrt_float(v: float) -> float}
            {fn inversesqrt_vec2(v: vec2) -> vec2}
            {fn inversesqrt_vec3(v: vec3) -> vec3}
            {fn inversesqrt_vec4(v: vec4) -> vec4}
            { "inversesqrt" }
        });
        self.add_functions(builtin_material_functions! {
            {fn abs_float(v: float) -> float}
            {fn abs_vec2(v: vec2) -> vec2}
            {fn abs_vec3(v: vec3) -> vec3}
            {fn abs_vec4(v: vec4) -> vec4}
            { "abs" }
        });
        self.add_functions(builtin_material_functions! {
            {fn sign_float(v: float) -> float}
            {fn sign_vec2(v: vec2) -> vec2}
            {fn sign_vec3(v: vec3) -> vec3}
            {fn sign_vec4(v: vec4) -> vec4}
            { "sign" }
        });
        self.add_functions(builtin_material_functions! {
            {fn floor_float(v: float) -> float}
            {fn floor_vec2(v: vec2) -> vec2}
            {fn floor_vec3(v: vec3) -> vec3}
            {fn floor_vec4(v: vec4) -> vec4}
            { "floor" }
        });
        self.add_functions(builtin_material_functions! {
            {fn ceil_float(v: float) -> float}
            {fn ceil_vec2(v: vec2) -> vec2}
            {fn ceil_vec3(v: vec3) -> vec3}
            {fn ceil_vec4(v: vec4) -> vec4}
            { "ceil" }
        });
        self.add_functions(builtin_material_functions! {
            {fn round_float(v: float) -> float}
            {fn round_vec2(v: vec2) -> vec2}
            {fn round_vec3(v: vec3) -> vec3}
            {fn round_vec4(v: vec4) -> vec4}
            { "round" }
        });
        self.add_functions(builtin_material_functions! {
            {fn roundEven_float(v: float) -> float}
            {fn roundEven_vec2(v: vec2) -> vec2}
            {fn roundEven_vec3(v: vec3) -> vec3}
            {fn roundEven_vec4(v: vec4) -> vec4}
            { "roundEven" }
        });
        self.add_functions(builtin_material_functions! {
            {fn fract_float(v: float) -> float}
            {fn fract_vec2(v: vec2) -> vec2}
            {fn fract_vec3(v: vec3) -> vec3}
            {fn fract_vec4(v: vec4) -> vec4}
            { "fract" }
        });
        self.add_functions(builtin_material_functions! {
            {fn mod_float(x: float, y: float) -> float}
            {fn mod_vec2(x: vec2, y: vec2) -> vec2}
            {fn mod_vec3(x: vec3, y: vec3) -> vec3}
            {fn mod_vec4(x: vec4, y: vec4) -> vec4}
            { "mod" }
        });
        self.add_functions(code_material_functions! {
            {fn mod_int(x: int, y: int) -> int}
            {fn mod_ivec2(x: ivec2, y: ivec2) -> ivec2}
            {fn mod_ivec3(x: ivec3, y: ivec3) -> ivec3}
            {fn mod_ivec4(x: ivec4, y: ivec4) -> ivec4}
            { "return x % y;" }
        });
        self.add_functions(builtin_material_functions! {
            {fn min_float(x: float, y: float) -> float}
            {fn min_vec2(x: vec2, y: vec2) -> vec2}
            {fn min_vec3(x: vec3, y: vec3) -> vec3}
            {fn min_vec4(x: vec4, y: vec4) -> vec4}
            { "min" }
        });
        self.add_functions(builtin_material_functions! {
            {fn max_float(x: float, y: float) -> float}
            {fn max_vec2(x: vec2, y: vec2) -> vec2}
            {fn max_vec3(x: vec3, y: vec3) -> vec3}
            {fn max_vec4(x: vec4, y: vec4) -> vec4}
            { "max" }
        });
        self.add_functions(builtin_material_functions! {
            {fn clamp_float(x: float, min: float, max: float) -> float}
            {fn clamp_vec2(x: vec2, min: vec2, max: vec2) -> vec2}
            {fn clamp_vec3(x: vec3, min: vec3, max: vec3) -> vec3}
            {fn clamp_vec4(x: vec4, min: vec4, max: vec4) -> vec4}
            { "clamp" }
        });
        self.add_functions(builtin_material_functions! {
            {fn mix_float(x: float, y: float, alpha: float) -> float}
            {fn mix_vec2(x: vec2, y: vec2, alpha: vec2) -> vec2}
            {fn mix_vec3(x: vec3, y: vec3, alpha: vec3) -> vec3}
            {fn mix_vec4(x: vec4, y: vec4, alpha: vec4) -> vec4}
            { "mix" }
        });
        self.add_functions(builtin_material_functions! {
            {fn step_float(edge: float, x: float) -> float}
            {fn step_vec2(edge: vec2, x: vec2) -> vec2}
            {fn step_vec3(edge: vec3, x: vec3) -> vec3}
            {fn step_vec4(edge: vec4, x: vec4) -> vec4}
            { "step" }
        });
        self.add_functions(builtin_material_functions! {
            {fn smoothstep_float(edge0: float, edge1: float, x: float) -> float}
            {fn smoothstep_vec2(edge0: vec2, edge1: vec2, x: vec2) -> vec2}
            {fn smoothstep_vec3(edge0: vec3, edge1: vec3, x: vec3) -> vec3}
            {fn smoothstep_vec4(edge0: vec4, edge1: vec4, x: vec4) -> vec4}
            { "smoothstep" }
        });
        self
    }

    fn with_vector_functions(mut self) -> Self {
        self.add_functions(builtin_material_functions! {
            {fn length_vec2(x: vec2) -> float}
            {fn length_vec3(x: vec3) -> float}
            {fn length_vec4(x: vec4) -> float}
            { "length" }
        });
        self.add_functions(builtin_material_functions! {
            {fn distance_vec2(x: vec2, y: vec2) -> float}
            {fn distance_vec3(x: vec3, y: vec3) -> float}
            {fn distance_vec4(x: vec4, y: vec4) -> float}
            { "distance" }
        });
        self.add_functions(builtin_material_functions! {
            {fn dot_vec2(x: vec2, y: vec2) -> float}
            {fn dot_vec3(x: vec3, y: vec3) -> float}
            {fn dot_vec4(x: vec4, y: vec4) -> float}
            { "dot" }
        });
        self.add_functions(builtin_material_functions! {
            {fn cross(x: vec3, y: vec3) -> vec3}
            { "cross" }
        });
        self.add_functions(builtin_material_functions! {
            {fn normalize_vec2(x: vec2) -> vec2}
            {fn normalize_vec3(x: vec3) -> vec3}
            {fn normalize_vec4(x: vec4) -> vec4}
            { "normalize" }
        });
        self.add_functions(builtin_material_functions! {
            {fn faceforward_vec2(n: vec2, i: vec2, nref: vec2) -> vec2}
            {fn faceforward_vec3(n: vec3, i: vec3, nref: vec3) -> vec3}
            {fn faceforward_vec4(n: vec4, i: vec4, nref: vec4) -> vec4}
            { "faceforward" }
        });
        self.add_functions(builtin_material_functions! {
            {fn reflect_vec2(i: vec2, n: vec2) -> vec2}
            {fn reflect_vec3(i: vec3, n: vec3) -> vec3}
            {fn reflect_vec4(i: vec4, n: vec4) -> vec4}
            { "reflect" }
        });
        self.add_functions(builtin_material_functions! {
            {fn refract_vec2(i: vec2, n: vec2, eta: float) -> vec2}
            {fn refract_vec3(i: vec3, n: vec3, eta: float) -> vec3}
            {fn refract_vec4(i: vec4, n: vec4, eta: float) -> vec4}
            { "refract" }
        });
        self.add_functions(code_material_functions! {
            {fn aspect_vec2(v: vec2) -> float}
            {fn aspect_ivec2(v: ivec2) -> int}
            { "return v.x / v.y;" }
        });
        self.add_functions(code_material_functions! {
            {fn sum_vec2(v: vec2) -> float}
            {fn sum_ivec2(v: ivec2) -> int}
            { "return v.x + v.y;" }
        });
        self.add_functions(code_material_functions! {
            {fn sum_vec3(v: vec3) -> float}
            {fn sum_ivec3(v: ivec3) -> int}
            { "return v.x + v.y + v.z;" }
        });
        self.add_functions(code_material_functions! {
            {fn sum_vec4(v: vec4) -> float}
            {fn sum_ivec4(v: ivec4) -> int}
            { "return v.x + v.y + v.z + v.w;" }
        });
        self.add_functions(code_material_functions! {
            {fn product_vec2(v: vec2) -> float}
            {fn product_ivec2(v: ivec2) -> int}
            { "return v.x * v.y;" }
        });
        self.add_functions(code_material_functions! {
            {fn product_vec3(v: vec3) -> float}
            {fn product_ivec3(v: ivec3) -> int}
            { "return v.x * v.y * v.z;" }
        });
        self.add_functions(code_material_functions! {
            {fn product_vec4(v: vec4) -> float}
            {fn product_ivec4(v: ivec4) -> int}
            { "return v.x * v.y * v.z * v.w;" }
        });
        self.add_function(code_material_function! {
            fn power_series_vec2(v: float) -> vec2 {
                "return vec2(1.0, v);"
            }
        });
        self.add_function(code_material_function! {
            fn power_series_vec3(v: float) -> vec3 {
                "return vec3(1.0, v, v * v);"
            }
        });
        self.add_function(code_material_function! {
            fn power_series_vec4(v: float) -> vec4 {
                "return vec4(1.0, v, v * v, v * v * v);"
            }
        });
        self
    }

    fn with_matrix_functions(mut self) -> Self {
        self.add_functions(builtin_material_functions! {
            {fn matrixCompMult_bmat2(x: bmat2, y: bmat2) -> bmat2}
            {fn matrixCompMult_bmat3(x: bmat3, y: bmat3) -> bmat3}
            {fn matrixCompMult_bmat4(x: bmat4, y: bmat4) -> bmat4}
            {fn matrixCompMult_mat2(x: mat2, y: mat2) -> mat2}
            {fn matrixCompMult_mat3(x: mat3, y: mat3) -> mat3}
            {fn matrixCompMult_mat4(x: mat4, y: mat4) -> mat4}
            {fn matrixCompMult_imat2(x: imat2, y: imat2) -> imat2}
            {fn matrixCompMult_imat3(x: imat3, y: imat3) -> imat3}
            {fn matrixCompMult_imat4(x: imat4, y: imat4) -> imat4}
            { "matrixCompMult" }
        });
        self.add_functions(builtin_material_functions! {
            {fn inverse_bmat2(x: bmat2) -> bmat2}
            {fn inverse_bmat3(x: bmat3) -> bmat3}
            {fn inverse_bmat4(x: bmat4) -> bmat4}
            {fn inverse_mat2(x: mat2) -> mat2}
            {fn inverse_mat3(x: mat3) -> mat3}
            {fn inverse_mat4(x: mat4) -> mat4}
            {fn inverse_imat2(x: imat2) -> imat2}
            {fn inverse_imat3(x: imat3) -> imat3}
            {fn inverse_imat4(x: imat4) -> imat4}
            { "inverse" }
        });
        self.add_functions(builtin_material_functions! {
            {fn transpose_bmat2(x: bmat2) -> bmat2}
            {fn transpose_bmat3(x: bmat3) -> bmat3}
            {fn transpose_bmat4(x: bmat4) -> bmat4}
            {fn transpose_mat2(x: mat2) -> mat2}
            {fn transpose_mat3(x: mat3) -> mat3}
            {fn transpose_mat4(x: mat4) -> mat4}
            {fn transpose_imat2(x: imat2) -> imat2}
            {fn transpose_imat3(x: imat3) -> imat3}
            {fn transpose_imat4(x: imat4) -> imat4}
            { "transpose" }
        });
        self
    }

    fn with_compare_functions(mut self) -> Self {
        self.add_functions(code_material_functions! {
            {fn lessThan_bool(x: bool, y: bool) -> bool}
            {fn lessThan_float(x: float, y: float) -> bool}
            {fn lessThan_int(x: int, y: int) -> bool}
            { "return x < y;" }
        });
        self.add_functions(builtin_material_functions! {
            {fn lessThan_vec2(x: vec2, y: vec2) -> bvec2}
            {fn lessThan_vec3(x: vec3, y: vec3) -> bvec3}
            {fn lessThan_vec4(x: vec4, y: vec4) -> bvec4}
            {fn lessThan_ivec2(x: ivec2, y: ivec2) -> bvec2}
            {fn lessThan_ivec3(x: ivec3, y: ivec3) -> bvec3}
            {fn lessThan_ivec4(x: ivec4, y: ivec4) -> bvec4}
            { "lessThan" }
        });
        self.add_functions(code_material_functions! {
            {fn lessThanEqual_bool(x: bool, y: bool) -> bool}
            {fn lessThanEqual_float(x: float, y: float) -> bool}
            {fn lessThanEqual_int(x: int, y: int) -> bool}
            { "return x <= y;" }
        });
        self.add_functions(builtin_material_functions! {
            {fn lessThanEqual_vec2(x: vec2, y: vec2) -> bvec2}
            {fn lessThanEqual_vec3(x: vec3, y: vec3) -> bvec3}
            {fn lessThanEqual_vec4(x: vec4, y: vec4) -> bvec4}
            {fn lessThanEqual_ivec2(x: ivec2, y: ivec2) -> bvec2}
            {fn lessThanEqual_ivec3(x: ivec3, y: ivec3) -> bvec3}
            {fn lessThanEqual_ivec4(x: ivec4, y: ivec4) -> bvec4}
            { "lessThanEqual" }
        });
        self.add_functions(code_material_functions! {
            {fn greaterThan_bool(x: bool, y: bool) -> bool}
            {fn greaterThan_float(x: float, y: float) -> bool}
            {fn greaterThan_int(x: int, y: int) -> bool}
            { "return x > y;" }
        });
        self.add_functions(builtin_material_functions! {
            {fn greaterThan_vec2(x: vec2, y: vec2) -> bvec2}
            {fn greaterThan_vec3(x: vec3, y: vec3) -> bvec3}
            {fn greaterThan_vec4(x: vec4, y: vec4) -> bvec4}
            {fn greaterThan_ivec2(x: ivec2, y: ivec2) -> bvec2}
            {fn greaterThan_ivec3(x: ivec3, y: ivec3) -> bvec3}
            {fn greaterThan_ivec4(x: ivec4, y: ivec4) -> bvec4}
            { "greaterThan" }
        });
        self.add_functions(code_material_functions! {
            {fn greaterThanEqual_bool(x: bool, y: bool) -> bool}
            {fn greaterThanEqual_float(x: float, y: float) -> bool}
            {fn greaterThanEqual_int(x: int, y: int) -> bool}
            { "return x >= y;" }
        });
        self.add_functions(builtin_material_functions! {
            {fn greaterThanEqual_vec2(x: vec2, y: vec2) -> bvec2}
            {fn greaterThanEqual_vec3(x: vec3, y: vec3) -> bvec3}
            {fn greaterThanEqual_vec4(x: vec4, y: vec4) -> bvec4}
            {fn greaterThanEqual_ivec2(x: ivec2, y: ivec2) -> bvec2}
            {fn greaterThanEqual_ivec3(x: ivec3, y: ivec3) -> bvec3}
            {fn greaterThanEqual_ivec4(x: ivec4, y: ivec4) -> bvec4}
            { "greaterThanEqual" }
        });
        self.add_functions(code_material_functions! {
            {fn equal_bool(x: bool, y: bool) -> bool}
            {fn equal_float(x: float, y: float) -> bool}
            {fn equal_int(x: int, y: int) -> bool}
            { "return x == y;" }
        });
        self.add_functions(builtin_material_functions! {
            {fn equal_bvec2(x: bvec2, y: bvec2) -> bvec2}
            {fn equal_bvec3(x: bvec3, y: bvec3) -> bvec3}
            {fn equal_bvec4(x: bvec4, y: bvec4) -> bvec4}
            {fn equal_vec2(x: vec2, y: vec2) -> bvec2}
            {fn equal_vec3(x: vec3, y: vec3) -> bvec3}
            {fn equal_vec4(x: vec4, y: vec4) -> bvec4}
            {fn equal_ivec2(x: ivec2, y: ivec2) -> bvec2}
            {fn equal_ivec3(x: ivec3, y: ivec3) -> bvec3}
            {fn equal_ivec4(x: ivec4, y: ivec4) -> bvec4}
            { "equal" }
        });
        self.add_functions(code_material_functions! {
            {fn notEqual_bool(x: bool, y: bool) -> bool}
            {fn notEqual_float(x: float, y: float) -> bool}
            {fn notEqual_int(x: int, y: int) -> bool}
            { "return x != y;" }
        });
        self.add_functions(builtin_material_functions! {
            {fn notEqual_bvec2(x: bvec2, y: bvec2) -> bvec2}
            {fn notEqual_bvec3(x: bvec3, y: bvec3) -> bvec3}
            {fn notEqual_bvec4(x: bvec4, y: bvec4) -> bvec4}
            {fn notEqual_vec2(x: vec2, y: vec2) -> bvec2}
            {fn notEqual_vec3(x: vec3, y: vec3) -> bvec3}
            {fn notEqual_vec4(x: vec4, y: vec4) -> bvec4}
            {fn notEqual_ivec2(x: ivec2, y: ivec2) -> bvec2}
            {fn notEqual_ivec3(x: ivec3, y: ivec3) -> bvec3}
            {fn notEqual_ivec4(x: ivec4, y: ivec4) -> bvec4}
            { "notEqual" }
        });
        self
    }

    fn with_logic_functions(mut self) -> Self {
        self.add_functions(builtin_material_functions! {
            {fn any_bvec2(x: bvec2) -> bool}
            {fn any_bvec3(x: bvec3) -> bool}
            {fn any_bvec4(x: bvec4) -> bool}
            { "any" }
        });
        self.add_functions(builtin_material_functions! {
            {fn all_bvec2(x: bvec2) -> bool}
            {fn all_bvec3(x: bvec3) -> bool}
            {fn all_bvec4(x: bvec4) -> bool}
            { "all" }
        });
        self.add_functions(builtin_material_functions! {
            {fn not_bvec2(x: bvec2) -> bvec2}
            {fn not_bvec3(x: bvec3) -> bvec3}
            {fn not_bvec4(x: bvec4) -> bvec4}
            { "not" }
        });
        self.add_function(code_material_function! {
            fn negate(v: bool) -> bool {
                "return !v;"
            }
        });
        self.add_functions(code_material_functions! {
            {fn if_float(condition: bool, truthy: float, falsy: float) -> float}
            {fn if_vec2(condition: bool, truthy: vec2, falsy: vec2) -> vec2}
            {fn if_vec3(condition: bool, truthy: vec3, falsy: vec3) -> vec3}
            {fn if_vec4(condition: bool, truthy: vec4, falsy: vec4) -> vec4}
            {fn if_int(condition: bool, truthy: int, falsy: int) -> int}
            {fn if_ivec2(condition: bool, truthy: ivec2, falsy: ivec2) -> ivec2}
            {fn if_ivec3(condition: bool, truthy: ivec3, falsy: ivec3) -> ivec3}
            {fn if_ivec4(condition: bool, truthy: ivec4, falsy: ivec4) -> ivec4}
            { "return mix(falsy, truthy, float(condition));" }
        });
        self.add_function(code_material_function! {
            fn discard_test(condition: bool) -> bool {
                "if (condition) discard; return condition;"
            }
        });
        self
    }

    fn with_texture_functions(mut self) -> Self {
        self.add_functions(builtin_material_functions! {
            {fn textureSize2d(sampler: sampler2D, lod: int) -> ivec2}
            {fn textureSize2dArray(sampler: sampler2DArray, lod: int) -> ivec3}
            {fn textureSize3d(sampler: sampler3D, lod: int) -> ivec3}
            { "textureSize" }
        });
        self.add_functions(builtin_material_functions! {
            {fn texture2d(sampler: sampler2D, coord: vec2) -> vec4}
            {fn texture2dArray(sampler: sampler2DArray, coord: vec3) -> vec4}
            {fn texture3d(sampler: sampler3D, coord: vec3) -> vec4}
            { "texture" }
        });
        self.add_functions(builtin_material_functions! {
            {fn textureProj2d_vec3(sampler: sampler2D, coord: vec3) -> vec4}
            {fn textureProj2d_vec4(sampler: sampler2D, coord: vec4) -> vec4}
            {fn textureProj3d(sampler: sampler3D, coord: vec4) -> vec4}
            { "textureProj" }
        });
        self.add_functions(builtin_material_functions! {
            {fn texelFetch2d(sampler: sampler2D, coord: ivec2, lod: int) -> vec4}
            {fn texelFetch2dArray(sampler: sampler2DArray, coord: ivec3, lod: int) -> vec4}
            {fn texelFetch3d(sampler: sampler3D, coord: ivec3, lod: int) -> vec4}
            { "texelFetch" }
        });
        self
    }

    fn with_virtual_texture_function(mut self) -> Self {
        self.add_functions(code_material_functions! {
            {fn virtualTextureCoord2d(coord: vec2, offset: vec2, size: vec2) -> vec2}
            {fn virtualTextureCoord3d(coord: vec3, offset: vec3, size: vec3) -> vec3}
            { "return clamp(mix(offset, offset + size, coord), 0.0, 1.0);" }
        });
        self.add_functions(code_material_functions! {
            {fn virtualTexture2d(sampler: sampler2D, coord: vec2, offset: vec2, size: vec2) -> vec4}
            {fn virtualTexture2dArray(sampler: sampler2DArray, coord: vec3, offset: vec3, size: vec3) -> vec4}
            {fn virtualTexture3d(sampler: sampler3D, coord: vec3, offset: vec3, size: vec3) -> vec4}
            { "return texture(sampler, clamp(mix(offset, offset + size, coord), 0.0, 1.0));" }
        });
        self
    }

    fn with_operator_functions(mut self) -> Self {
        self.add_functions(code_material_functions! {
            {fn add_float(a: float, b: float) -> float}
            {fn add_vec2(a: vec2, b: vec2) -> vec2}
            {fn add_vec3(a: vec3, b: vec3) -> vec3}
            {fn add_vec4(a: vec4, b: vec4) -> vec4}
            {fn add_int(a: int, b: int) -> int}
            {fn add_ivec2(a: ivec2, b: ivec2) -> ivec2}
            {fn add_ivec3(a: ivec3, b: ivec3) -> ivec3}
            {fn add_ivec4(a: ivec4, b: ivec4) -> ivec4}
            { "return a + b;" }
        });
        self.add_functions(code_material_functions! {
            {fn sub_float(a: float, b: float) -> float}
            {fn sub_vec2(a: vec2, b: vec2) -> vec2}
            {fn sub_vec3(a: vec3, b: vec3) -> vec3}
            {fn sub_vec4(a: vec4, b: vec4) -> vec4}
            {fn sub_int(a: int, b: int) -> int}
            {fn sub_ivec2(a: ivec2, b: ivec2) -> ivec2}
            {fn sub_ivec3(a: ivec3, b: ivec3) -> ivec3}
            {fn sub_ivec4(a: ivec4, b: ivec4) -> ivec4}
            { "return a - b;" }
        });
        self.add_functions(code_material_functions! {
            {fn mul_float(a: float, b: float) -> float}
            {fn mul_vec2(a: vec2, b: vec2) -> vec2}
            {fn mul_vec3(a: vec3, b: vec3) -> vec3}
            {fn mul_vec4(a: vec4, b: vec4) -> vec4}
            {fn mul_mat2(a: mat2, b: mat2) -> mat2}
            {fn mul_mat3(a: mat3, b: mat3) -> mat3}
            {fn mul_mat4(a: mat4, b: mat4) -> mat4}
            {fn mul_mat2_vec2(a: mat2, b: vec2) -> vec2}
            {fn mul_mat3_vec3(a: mat3, b: vec3) -> vec3}
            {fn mul_mat4_vec4(a: mat4, b: vec4) -> vec4}
            {fn mul_int(a: int, b: int) -> int}
            {fn mul_ivec2(a: ivec2, b: ivec2) -> ivec2}
            {fn mul_ivec3(a: ivec3, b: ivec3) -> ivec3}
            {fn mul_ivec4(a: ivec4, b: ivec4) -> ivec4}
            {fn mul_imat2(a: imat2, b: imat2) -> imat2}
            {fn mul_imat3(a: imat3, b: imat3) -> imat3}
            {fn mul_imat4(a: imat4, b: imat4) -> imat4}
            {fn mul_imat2_ivec2(a: imat2, b: ivec2) -> ivec2}
            {fn mul_imat3_ivec3(a: imat3, b: ivec3) -> ivec3}
            {fn mul_imat4_ivec4(a: imat4, b: ivec4) -> ivec4}
            { "return a * b;" }
        });
        self.add_functions(code_material_functions! {
            {fn div_float(a: float, b: float) -> float}
            {fn div_vec2(a: vec2, b: vec2) -> vec2}
            {fn div_vec3(a: vec3, b: vec3) -> vec3}
            {fn div_vec4(a: vec4, b: vec4) -> vec4}
            {fn div_int(a: int, b: int) -> int}
            {fn div_ivec2(a: ivec2, b: ivec2) -> ivec2}
            {fn div_ivec3(a: ivec3, b: ivec3) -> ivec3}
            {fn div_ivec4(a: ivec4, b: ivec4) -> ivec4}
            { "return a / b;" }
        });
        self.add_function(code_material_function! {
            fn bitwise_and(a: int, b: int) -> int {
                "return a & b;"
            }
        });
        self.add_function(code_material_function! {
            fn bitwise_or(a: int, b: int) -> int {
                "return a | b;"
            }
        });
        self.add_function(code_material_function! {
            fn bitwise_xor(a: int, b: int) -> int {
                "return a ^ b;"
            }
        });
        self.add_function(code_material_function! {
            fn bitwise_shift_left(v: int, bits: int) -> int {
                "return v << bits;"
            }
        });
        self.add_function(code_material_function! {
            fn bitwise_shift_right(v: int, bits: int) -> int {
                "return v >> bits;"
            }
        });
        self
    }

    fn with_cast_functions(mut self) -> Self {
        self.add_functions(code_material_functions! {
            {fn cast_float_bool(v: float) -> bool}
            {fn cast_int_bool(v: int) -> bool}
            { "return bool(v);" }
        });
        self.add_functions(code_material_functions! {
            {fn cast_bool_float(v: bool) -> float}
            {fn cast_int_float(v: int) -> float}
            { "return float(v);" }
        });
        self.add_functions(code_material_functions! {
            {fn cast_bool_int(v: bool) -> int}
            {fn cast_float_int(v: float) -> int}
            { "return int(v);" }
        });
        self.add_functions(code_material_functions! {
            {fn cast_vec2_bvec2(v: vec2) -> bvec2}
            {fn cast_ivec2_bvec2(v: ivec2) -> bvec2}
            { "return bvec2(v);" }
        });
        self.add_functions(code_material_functions! {
            {fn cast_bvec2_vec2(v: bvec2) -> vec2}
            {fn cast_ivec2_vec2(v: ivec2) -> vec2}
            { "return vec2(v);" }
        });
        self.add_functions(code_material_functions! {
            {fn cast_bvec2_ivec2(v: bvec2) -> ivec2}
            {fn cast_vec2_ivec2(v: vec2) -> ivec2}
            { "return ivec2(v);" }
        });
        self.add_functions(code_material_functions! {
            {fn cast_vec3_bvec3(v: vec3) -> bvec3}
            {fn cast_ivec3_bvec3(v: ivec3) -> bvec3}
            { "return bvec3(v);" }
        });
        self.add_functions(code_material_functions! {
            {fn cast_bvec3_vec3(v: bvec3) -> vec3}
            {fn cast_ivec3_vec3(v: ivec3) -> vec3}
            { "return vec3(v);" }
        });
        self.add_functions(code_material_functions! {
            {fn cast_bvec3_ivec3(v: bvec3) -> ivec3}
            {fn cast_vec3_ivec3(v: vec3) -> ivec3}
            { "return ivec3(v);" }
        });
        self.add_functions(code_material_functions! {
            {fn cast_vec4_bvec4(v: vec4) -> bvec4}
            {fn cast_ivec4_bvec4(v: ivec4) -> bvec4}
            { "return bvec4(v);" }
        });
        self.add_functions(code_material_functions! {
            {fn cast_bvec4_vec4(v: bvec4) -> vec4}
            {fn cast_ivec4_vec4(v: ivec4) -> vec4}
            { "return vec4(v);" }
        });
        self.add_functions(code_material_functions! {
            {fn cast_bvec4_ivec4(v: bvec4) -> ivec4}
            {fn cast_vec4_ivec4(v: vec4) -> ivec4}
            { "return ivec4(v);" }
        });
        self.add_functions(code_material_functions! {
            {fn cast_mat3_mat2(v: mat3) -> mat2}
            {fn cast_mat4_mat2(v: mat4) -> mat2}
            { "return mat2(v);" }
        });
        self.add_functions(code_material_functions! {
            {fn cast_mat2_mat3(v: mat2) -> mat3}
            {fn cast_mat4_mat3(v: mat4) -> mat3}
            { "return mat3(v);" }
        });
        self.add_functions(code_material_functions! {
            {fn cast_mat2_mat4(v: mat2) -> mat4}
            {fn cast_mat3_mat4(v: mat3) -> mat4}
            { "return mat4(v);" }
        });
        self
    }

    fn with_fill_functions(mut self) -> Self {
        self.add_function(code_material_function! {
            fn fill_bvec2(v: bool) -> bvec2 {
                "return bvec2(v, v);"
            }
        });
        self.add_function(code_material_function! {
            fn fill_bvec3(v: bool) -> bvec3 {
                "return bvec3(v, v, v);"
            }
        });
        self.add_function(code_material_function! {
            fn fill_bvec4(v: bool) -> bvec4 {
                "return bvec4(v, v, v, v);"
            }
        });
        self.add_function(code_material_function! {
            fn fill_vec2(v: float) -> vec2 {
                "return vec2(v, v);"
            }
        });
        self.add_function(code_material_function! {
            fn fill_vec3(v: float) -> vec3 {
                "return vec3(v, v, v);"
            }
        });
        self.add_function(code_material_function! {
            fn fill_vec4(v: float) -> vec4 {
                "return vec4(v, v, v, v);"
            }
        });
        self.add_function(code_material_function! {
            fn fill_ivec2(v: int) -> ivec2 {
                "return ivec2(v, v);"
            }
        });
        self.add_function(code_material_function! {
            fn fill_ivec3(v: int) -> ivec3 {
                "return ivec3(v, v, v);"
            }
        });
        self.add_function(code_material_function! {
            fn fill_ivec4(v: int) -> ivec4 {
                "return ivec4(v, v, v, v);"
            }
        });
        self
    }

    fn with_make_functions(mut self) -> Self {
        self.add_function(code_material_function! {
            fn make_bvec2(x: bool, y: bool) -> bvec2 {
                "return bvec2(x, y);"
            }
        });
        self.add_function(code_material_function! {
            fn make_bvec3(x: bool, y: bool, z: bool) -> bvec3 {
                "return bvec3(x, y, z);"
            }
        });
        self.add_function(code_material_function! {
            fn make_bvec4(x: bool, y: bool, z: bool, w: bool) -> bvec4 {
                "return bvec4(x, y, z, w);"
            }
        });
        self.add_function(code_material_function! {
            fn make_vec2(x: float, y: float) -> vec2 {
                "return vec2(x, y);"
            }
        });
        self.add_function(code_material_function! {
            fn make_vec3(x: float, y: float, z: float) -> vec3 {
                "return vec3(x, y, z);"
            }
        });
        self.add_function(code_material_function! {
            fn make_vec4(x: float, y: float, z: float, w: float) -> vec4 {
                "return vec4(x, y, z, w);"
            }
        });
        self.add_function(code_material_function! {
            fn make_mat2_identity() -> mat2 {
                "return mat2(1, 0, 0, 1);"
            }
        });
        self.add_function(code_material_function! {
            fn make_mat2(a: vec2, b: vec2) -> mat2 {
                "return mat2(a, b);"
            }
        });
        self.add_function(code_material_function! {
            fn make_mat3_identity() -> mat3 {
                "return mat3(1, 0, 0, 0, 1, 0, 0, 0, 1);"
            }
        });
        self.add_function(code_material_function! {
            fn make_mat3(a: vec3, b: vec3, c: vec3) -> mat3 {
                "return mat3(a, b, c);"
            }
        });
        self.add_function(code_material_function! {
            fn make_mat4_identity() -> mat4 {
                "return mat4(1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1);"
            }
        });
        self.add_function(code_material_function! {
            fn make_mat4(a: vec4, b: vec4, c: vec4, d: vec4) -> mat4 {
                "return mat4(a, b, c, d);"
            }
        });
        self.add_function(code_material_function! {
            fn make_ivec2(x: int, y: int) -> ivec2 {
                "return ivec2(x, y);"
            }
        });
        self.add_function(code_material_function! {
            fn make_ivec3(x: int, y: int, z: int) -> ivec3 {
                "return ivec3(x, y, z);"
            }
        });
        self.add_function(code_material_function! {
            fn make_ivec4(x: int, y: int, z: int, w: int) -> ivec4 {
                "return ivec4(x, y, z, w);"
            }
        });
        self.add_function(code_material_function! {
            fn make_imat2_identity() -> imat2 {
                "return imat2(1, 0, 0, 1);"
            }
        });
        self.add_function(code_material_function! {
            fn make_imat2(a: ivec2, b: ivec2) -> imat2 {
                "return imat2(a, b);"
            }
        });
        self.add_function(code_material_function! {
            fn make_imat3_identity() -> imat3 {
                "return imat3(1, 0, 0, 0, 1, 0, 0, 0, 1);"
            }
        });
        self.add_function(code_material_function! {
            fn make_imat3(a: ivec3, b: ivec3, c: ivec3) -> imat3 {
                "return imat3(a, b, c);"
            }
        });
        self.add_function(code_material_function! {
            fn make_imat4_identity() -> imat4 {
                "return imat4(1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1);"
            }
        });
        self.add_function(code_material_function! {
            fn make_imat4(a: ivec4, b: ivec4, c: ivec4) -> imat4 {
                "return imat4(a, b, c, d);"
            }
        });
        self
    }

    fn with_mask_functions(mut self) -> Self {
        self.add_functions(code_material_functions! {
            {fn maskX_bvec2(v: bvec2) -> bool}
            {fn maskX_bvec3(v: bvec3) -> bool}
            {fn maskX_bvec4(v: bvec4) -> bool}
            {fn maskX_vec2(v: vec2) -> float}
            {fn maskX_vec3(v: vec3) -> float}
            {fn maskX_vec4(v: vec4) -> float}
            {fn maskX_ivec2(v: ivec2) -> int}
            {fn maskX_ivec3(v: ivec3) -> int}
            {fn maskX_ivec4(v: ivec4) -> int}
            { "return v.x;" }
        });
        self.add_functions(code_material_functions! {
            {fn maskY_bvec2(v: bvec2) -> bool}
            {fn maskY_bvec3(v: bvec3) -> bool}
            {fn maskY_bvec4(v: bvec4) -> bool}
            {fn maskY_vec2(v: vec2) -> float}
            {fn maskY_vec3(v: vec3) -> float}
            {fn maskY_vec4(v: vec4) -> float}
            {fn maskY_ivec2(v: ivec2) -> int}
            {fn maskY_ivec3(v: ivec3) -> int}
            {fn maskY_ivec4(v: ivec4) -> int}
            { "return v.y;" }
        });
        self.add_functions(code_material_functions! {
            {fn maskZ_bvec3(v: bvec3) -> bool}
            {fn maskZ_bvec4(v: bvec4) -> bool}
            {fn maskZ_vec3(v: vec3) -> float}
            {fn maskZ_vec4(v: vec4) -> float}
            {fn maskZ_ivec3(v: ivec3) -> int}
            {fn maskZ_ivec4(v: ivec4) -> int}
            { "return v.z;" }
        });
        self.add_functions(code_material_functions! {
            {fn maskW_bvec4(v: bvec4) -> bool}
            {fn maskW_vec4(v: vec4) -> float}
            {fn maskW_ivec4(v: ivec4) -> int}
            { "return v.w;" }
        });
        self.add_functions(code_material_functions! {
            {fn maskXX_bvec3(v: bvec3) -> bvec2}
            {fn maskXX_bvec4(v: bvec4) -> bvec2}
            {fn maskXX_vec3(v: vec3) -> vec2}
            {fn maskXX_vec4(v: vec4) -> vec2}
            {fn maskXX_ivec3(v: ivec3) -> ivec2}
            {fn maskXX_ivec4(v: ivec4) -> ivec2}
            { "return v.xx;" }
        });
        self.add_functions(code_material_functions! {
            {fn maskXY_bvec3(v: bvec3) -> bvec2}
            {fn maskXY_bvec4(v: bvec4) -> bvec2}
            {fn maskXY_vec3(v: vec3) -> vec2}
            {fn maskXY_vec4(v: vec4) -> vec2}
            {fn maskXY_ivec3(v: ivec3) -> ivec2}
            {fn maskXY_ivec4(v: ivec4) -> ivec2}
            { "return v.xy;" }
        });
        self.add_functions(code_material_functions! {
            {fn maskXZ_bvec3(v: bvec3) -> bvec2}
            {fn maskXZ_bvec4(v: bvec4) -> bvec2}
            {fn maskXZ_vec3(v: vec3) -> vec2}
            {fn maskXZ_vec4(v: vec4) -> vec2}
            {fn maskXZ_ivec3(v: ivec3) -> ivec2}
            {fn maskXZ_ivec4(v: ivec4) -> ivec2}
            { "return v.xz;" }
        });
        self.add_functions(code_material_functions! {
            {fn maskYX_bvec3(v: bvec3) -> bvec2}
            {fn maskYX_bvec4(v: bvec4) -> bvec2}
            {fn maskYX_vec3(v: vec3) -> vec2}
            {fn maskYX_vec4(v: vec4) -> vec2}
            {fn maskYX_ivec3(v: ivec3) -> ivec2}
            {fn maskYX_ivec4(v: ivec4) -> ivec2}
            { "return v.yx;" }
        });
        self.add_functions(code_material_functions! {
            {fn maskYY_bvec3(v: bvec3) -> bvec2}
            {fn maskYY_bvec4(v: bvec4) -> bvec2}
            {fn maskYY_vec3(v: vec3) -> vec2}
            {fn maskYY_vec4(v: vec4) -> vec2}
            {fn maskYY_ivec3(v: ivec3) -> ivec2}
            {fn maskYY_ivec4(v: ivec4) -> ivec2}
            { "return v.yy;" }
        });
        self.add_functions(code_material_functions! {
            {fn maskYZ_bvec3(v: bvec3) -> bvec2}
            {fn maskYZ_bvec4(v: bvec4) -> bvec2}
            {fn maskYZ_vec3(v: vec3) -> vec2}
            {fn maskYZ_vec4(v: vec4) -> vec2}
            {fn maskYZ_ivec3(v: ivec3) -> ivec2}
            {fn maskYZ_ivec4(v: ivec4) -> ivec2}
            { "return v.yz;" }
        });
        self.add_functions(code_material_functions! {
            {fn maskZX_bvec3(v: bvec3) -> bvec2}
            {fn maskZX_bvec4(v: bvec4) -> bvec2}
            {fn maskZX_vec3(v: vec3) -> vec2}
            {fn maskZX_vec4(v: vec4) -> vec2}
            {fn maskZX_ivec3(v: ivec3) -> ivec2}
            {fn maskZX_ivec4(v: ivec4) -> ivec2}
            { "return v.zx;" }
        });
        self.add_functions(code_material_functions! {
            {fn maskZY_bvec3(v: bvec3) -> bvec2}
            {fn maskZY_bvec4(v: bvec4) -> bvec2}
            {fn maskZY_vec3(v: vec3) -> vec2}
            {fn maskZY_vec4(v: vec4) -> vec2}
            {fn maskZY_ivec3(v: ivec3) -> ivec2}
            {fn maskZY_ivec4(v: ivec4) -> ivec2}
            { "return v.zy;" }
        });
        self.add_functions(code_material_functions! {
            {fn maskZZ_bvec3(v: bvec3) -> bvec2}
            {fn maskZZ_bvec4(v: bvec4) -> bvec2}
            {fn maskZZ_vec3(v: vec3) -> vec2}
            {fn maskZZ_vec4(v: vec4) -> vec2}
            {fn maskZZ_ivec3(v: ivec3) -> ivec2}
            {fn maskZZ_ivec4(v: ivec4) -> ivec2}
            { "return v.zz;" }
        });
        self.add_functions(code_material_functions! {
            {fn maskXXX_bvec4(v: bvec4) -> bvec3}
            {fn maskXXX_vec4(v: vec4) -> vec3}
            {fn maskXXX_ivec4(v: ivec4) -> ivec3}
            { "return v.xxx;" }
        });
        self.add_functions(code_material_functions! {
            {fn maskXXY_bvec4(v: bvec4) -> bvec3}
            {fn maskXXY_vec4(v: vec4) -> vec3}
            {fn maskXXY_ivec4(v: ivec4) -> ivec3}
            { "return v.xxy;" }
        });
        self.add_functions(code_material_functions! {
            {fn maskXXZ_bvec4(v: bvec4) -> bvec3}
            {fn maskXXZ_vec4(v: vec4) -> vec3}
            {fn maskXXZ_ivec4(v: ivec4) -> ivec3}
            { "return v.xxz;" }
        });
        self.add_functions(code_material_functions! {
            {fn maskXYX_bvec4(v: bvec4) -> bvec3}
            {fn maskXYX_vec4(v: vec4) -> vec3}
            {fn maskXYX_ivec4(v: ivec4) -> ivec3}
            { "return v.xyx;" }
        });
        self.add_functions(code_material_functions! {
            {fn maskXYY_bvec4(v: bvec4) -> bvec3}
            {fn maskXYY_vec4(v: vec4) -> vec3}
            {fn maskXYY_ivec4(v: ivec4) -> ivec3}
            { "return v.xyy;" }
        });
        self.add_functions(code_material_functions! {
            {fn maskXYZ_bvec4(v: bvec4) -> bvec3}
            {fn maskXYZ_vec4(v: vec4) -> vec3}
            {fn maskXYZ_ivec4(v: ivec4) -> ivec3}
            { "return v.xyz;" }
        });
        self.add_functions(code_material_functions! {
            {fn maskXZX_bvec4(v: bvec4) -> bvec3}
            {fn maskXZX_vec4(v: vec4) -> vec3}
            {fn maskXZX_ivec4(v: ivec4) -> ivec3}
            { "return v.xzx;" }
        });
        self.add_functions(code_material_functions! {
            {fn maskXZY_bvec4(v: bvec4) -> bvec3}
            {fn maskXZY_vec4(v: vec4) -> vec3}
            {fn maskXZY_ivec4(v: ivec4) -> ivec3}
            { "return v.xzy;" }
        });
        self.add_functions(code_material_functions! {
            {fn maskXZZ_bvec4(v: bvec4) -> bvec3}
            {fn maskXZZ_vec4(v: vec4) -> vec3}
            {fn maskXZZ_ivec4(v: ivec4) -> ivec3}
            { "return v.xzz;" }
        });
        self.add_functions(code_material_functions! {
            {fn maskYXX_bvec4(v: bvec4) -> bvec3}
            {fn maskYXX_vec4(v: vec4) -> vec3}
            {fn maskYXX_ivec4(v: ivec4) -> ivec3}
            { "return v.yxx;" }
        });
        self.add_functions(code_material_functions! {
            {fn maskYXY_bvec4(v: bvec4) -> bvec3}
            {fn maskYXY_vec4(v: vec4) -> vec3}
            {fn maskYXY_ivec4(v: ivec4) -> ivec3}
            { "return v.yxy;" }
        });
        self.add_functions(code_material_functions! {
            {fn maskYXZ_bvec4(v: bvec4) -> bvec3}
            {fn maskYXZ_vec4(v: vec4) -> vec3}
            {fn maskYXZ_ivec4(v: ivec4) -> ivec3}
            { "return v.yxz;" }
        });
        self.add_functions(code_material_functions! {
            {fn maskYYX_bvec4(v: bvec4) -> bvec3}
            {fn maskYYX_vec4(v: vec4) -> vec3}
            {fn maskYYX_ivec4(v: ivec4) -> ivec3}
            { "return v.yyx;" }
        });
        self.add_functions(code_material_functions! {
            {fn maskYYY_bvec4(v: bvec4) -> bvec3}
            {fn maskYYY_vec4(v: vec4) -> vec3}
            {fn maskYYY_ivec4(v: ivec4) -> ivec3}
            { "return v.yyy;" }
        });
        self.add_functions(code_material_functions! {
            {fn maskYYZ_bvec4(v: bvec4) -> bvec3}
            {fn maskYYZ_vec4(v: vec4) -> vec3}
            {fn maskYYZ_ivec4(v: ivec4) -> ivec3}
            { "return v.yyz;" }
        });
        self.add_functions(code_material_functions! {
            {fn maskYZX_bvec4(v: bvec4) -> bvec3}
            {fn maskYZX_vec4(v: vec4) -> vec3}
            {fn maskYZX_ivec4(v: ivec4) -> ivec3}
            { "return v.yzx;" }
        });
        self.add_functions(code_material_functions! {
            {fn maskYZY_bvec4(v: bvec4) -> bvec3}
            {fn maskYZY_vec4(v: vec4) -> vec3}
            {fn maskYZY_ivec4(v: ivec4) -> ivec3}
            { "return v.yzy;" }
        });
        self.add_functions(code_material_functions! {
            {fn maskYZZ_bvec4(v: bvec4) -> bvec3}
            {fn maskYZZ_vec4(v: vec4) -> vec3}
            {fn maskYZZ_ivec4(v: ivec4) -> ivec3}
            { "return v.yzz;" }
        });
        self.add_functions(code_material_functions! {
            {fn maskZXX_bvec4(v: bvec4) -> bvec3}
            {fn maskZXX_vec4(v: vec4) -> vec3}
            {fn maskZXX_ivec4(v: ivec4) -> ivec3}
            { "return v.zxx;" }
        });
        self.add_functions(code_material_functions! {
            {fn maskZXY_bvec4(v: bvec4) -> bvec3}
            {fn maskZXY_vec4(v: vec4) -> vec3}
            {fn maskZXY_ivec4(v: ivec4) -> ivec3}
            { "return v.zxy;" }
        });
        self.add_functions(code_material_functions! {
            {fn maskZXZ_bvec4(v: bvec4) -> bvec3}
            {fn maskZXZ_vec4(v: vec4) -> vec3}
            {fn maskZXZ_ivec4(v: ivec4) -> ivec3}
            { "return v.zxz;" }
        });
        self.add_functions(code_material_functions! {
            {fn maskZYX_bvec4(v: bvec4) -> bvec3}
            {fn maskZYX_vec4(v: vec4) -> vec3}
            {fn maskZYX_ivec4(v: ivec4) -> ivec3}
            { "return v.zyx;" }
        });
        self.add_functions(code_material_functions! {
            {fn maskZYY_bvec4(v: bvec4) -> bvec3}
            {fn maskZYY_vec4(v: vec4) -> vec3}
            {fn maskZYY_ivec4(v: ivec4) -> ivec3}
            { "return v.zyy;" }
        });
        self.add_functions(code_material_functions! {
            {fn maskZYZ_bvec4(v: bvec4) -> bvec3}
            {fn maskZYZ_vec4(v: vec4) -> vec3}
            {fn maskZYZ_ivec4(v: ivec4) -> ivec3}
            { "return v.zyz;" }
        });
        self.add_functions(code_material_functions! {
            {fn maskZZX_bvec4(v: bvec4) -> bvec3}
            {fn maskZZX_vec4(v: vec4) -> vec3}
            {fn maskZZX_ivec4(v: ivec4) -> ivec3}
            { "return v.zzx;" }
        });
        self.add_functions(code_material_functions! {
            {fn maskZZY_bvec4(v: bvec4) -> bvec3}
            {fn maskZZY_vec4(v: vec4) -> vec3}
            {fn maskZZY_ivec4(v: ivec4) -> ivec3}
            { "return v.zzy;" }
        });
        self.add_functions(code_material_functions! {
            {fn maskZZZ_bvec4(v: bvec4) -> bvec3}
            {fn maskZZZ_vec4(v: vec4) -> vec3}
            {fn maskZZZ_ivec4(v: ivec4) -> ivec3}
            { "return v.zzz;" }
        });
        self
    }

    fn with_append_functions(mut self) -> Self {
        self.add_function(code_material_function! {
            fn append_bvec2(a: bool, b: bool) -> bvec2 {
                "return bvec2(a, b);"
            }
        });
        self.add_function(code_material_function! {
            fn append_bvec3(a: bvec2, b: bool) -> bvec3 {
                "return bvec3(a.xy, b);"
            }
        });
        self.add_function(code_material_function! {
            fn append_bvec4(a: bvec3, b: bool) -> bvec4 {
                "return bvec4(a.xyz, b);"
            }
        });
        self.add_function(code_material_function! {
            fn append_vec2(a: float, b: float) -> vec2 {
                "return vec2(a, b);"
            }
        });
        self.add_function(code_material_function! {
            fn append_vec3(a: vec2, b: float) -> vec3 {
                "return vec3(a.xy, b);"
            }
        });
        self.add_function(code_material_function! {
            fn append_vec4(a: vec3, b: float) -> vec4 {
                "return vec4(a.xyz, b);"
            }
        });
        self.add_function(code_material_function! {
            fn append_ivec2(a: int, b: int) -> ivec2 {
                "return ivec2(a, b);"
            }
        });
        self.add_function(code_material_function! {
            fn append_ivec3(a: ivec2, b: int) -> ivec3 {
                "return ivec3(a.xy, b);"
            }
        });
        self.add_function(code_material_function! {
            fn append_ivec4(a: ivec3, b: int) -> ivec4 {
                "return ivec4(a.xyz, b);"
            }
        });
        self
    }

    fn with_truncate_functions(mut self) -> Self {
        self.add_functions(code_material_functions! {
            {fn truncate_bvec2(v: bvec2) -> bool}
            {fn truncate_vec2(v: vec2) -> float}
            {fn truncate_ivec2(v: ivec2) -> int}
            { "return v.x;" }
        });
        self.add_functions(code_material_functions! {
            {fn truncate_bvec3(v: bvec3) -> bvec2}
            {fn truncate_vec3(v: vec3) -> vec2}
            {fn truncate_ivec3(v: ivec3) -> ivec2}
            { "return v.xy;" }
        });
        self.add_functions(code_material_functions! {
            {fn truncate_bvec4(v: bvec4) -> bvec3}
            {fn truncate_vec4(v: vec4) -> vec3}
            {fn truncate_ivec4(v: ivec4) -> ivec3}
            { "return v.xyz;" }
        });
        self
    }

    fn with_dithering(mut self) -> Self {
        self.add_function(graph_material_function! {
            fn dither_threshold(texture: sampler2D, coord: ivec2) -> float {
                [size = (textureSize2d, sampler: texture, lod: {0})]
                [wrapped_coord = (mod_ivec2, x: coord, y: size)]
                [return (maskX_vec4,
                    v: (texelFetch2d, sampler: texture, coord: wrapped_coord, lod: {0})
                )]
            }
        });
        self.add_function(graph_material_function! {
            fn dither_mask(texture: sampler2D, coord: ivec2, value: float) -> bool {
                [threshold = (dither_threshold, texture: texture, coord: coord)]
                [return (greaterThan_float, x: value, y: threshold)]
            }
        });
        self.add_function(graph_material_function! {
            fn temporal_dither_threshold(texture: sampler2D, coord: ivec2, time: float) -> float {
                [threshold = (dither_threshold, texture: texture, coord: coord)]
                [return (fract_float, v: (add_float, a: threshold, b: time))]
            }
        });
        self.add_function(graph_material_function! {
            fn temporal_dither_mask(texture: sampler2D, coord: ivec2, time: float, value: float) -> bool {
                [threshold = (temporal_dither_threshold, texture: texture, coord: coord, time: time)]
                [return (greaterThan_float, x: value, y: threshold)]
            }
        });
        self
    }

    fn with_vertanim_middleware(mut self) -> Self {
        self.add_middleware(
            "vertanim".to_owned(),
            material_graph! {
                inputs {
                    [vertex] in position as in_position: vec3 = {vec3(0.0, 0.0, 0.0)};
                    [vertex] in animationColumn: float = {0.0};

                    [vertex] uniform animationFrames: sampler2D;
                    [vertex] uniform animationRow: float;
                }

                outputs {
                    [vertex] out position as out_position: vec3;
                }

                [row = (fract_float, v: animationRow)]
                [coord = (make_vec2, x: animationColumn, y: row)]
                [data = (texture2d, sampler: animationFrames, coord: coord)]
                [offset = (truncate_vec4, v: data)]
                [enabled = (step_float, edge: {0.5}, x: (maskW_vec4, v: data))]
                [offset := (mul_vec3, a: offset, b: (fill_vec3, v: enabled))]
                [(add_vec3, a: in_position, b: offset) -> out_position]
            },
        );
        self
    }

    fn with_skinning_middleware(mut self) -> Self {
        self.add_function(graph_material_function! {
            fn skinning_fetch_bone_matrix(texture: sampler2D, index: int) -> mat4 {
                [return (make_mat4,
                    a: (texelFetch2d, sampler: texture, coord: (make_ivec2, x: {0}, y: index), lod: {0}),
                    b: (texelFetch2d, sampler: texture, coord: (make_ivec2, x: {1}, y: index), lod: {0}),
                    c: (texelFetch2d, sampler: texture, coord: (make_ivec2, x: {2}, y: index), lod: {0}),
                    d: (texelFetch2d, sampler: texture, coord: (make_ivec2, x: {3}, y: index), lod: {0})
                )]
            }
        });
        self.add_function(graph_material_function! {
            fn skinning_weight_position(bone_matrix: mat4, position: vec4, weight: float) -> vec4 {
                [return (mul_mat4_vec4,
                    a: bone_matrix,
                    b: (mul_vec4, a: position, b: (fill_vec4, v: weight))
                )]
            }
        });
        self.add_middleware(
            "skinning".to_owned(),
            material_graph! {
                inputs {
                    [vertex] in position as in_position: vec3 = {vec3(0.0, 0.0, 0.0)};
                    [vertex] in boneIndices: int = {0};
                    [vertex] in boneWeights: vec4 = {vec4(0.0, 0.0, 0.0, 0.0)};

                    [vertex] uniform boneMatrices: sampler2D;
                }

                outputs {
                    [vertex] out position as out_position: vec3;
                }

                [pos = (append_vec4, a: in_position, b: {1.0})]
                [index_a = (bitwise_and, a: boneIndices, b: {0xFF})]
                [index_b = (bitwise_and, a: (bitwise_shift_right, v: boneIndices, bits: {8}), b: {0xFF})]
                [index_c = (bitwise_and, a: (bitwise_shift_right, v: boneIndices, bits: {16}), b: {0xFF})]
                [index_d = (bitwise_and, a: (bitwise_shift_right, v: boneIndices, bits: {24}), b: {0xFF})]
                [result = (skinning_weight_position,
                    bone_matrix: (skinning_fetch_bone_matrix, texture: boneMatrices, index: index_a),
                    position: pos,
                    weight: (maskX_vec4, v: boneWeights)
                )]
                [weighted = (skinning_weight_position,
                    bone_matrix: (skinning_fetch_bone_matrix, texture: boneMatrices, index: index_b),
                    position: pos,
                    weight: (maskY_vec4, v: boneWeights)
                )]
                [result := (add_vec4, a: result, b: weighted)]
                [weighted := (skinning_weight_position,
                    bone_matrix: (skinning_fetch_bone_matrix, texture: boneMatrices, index: index_c),
                    position: pos,
                    weight: (maskZ_vec4, v: boneWeights)
                )]
                [result := (add_vec4, a: result, b: weighted)]
                [weighted := (skinning_weight_position,
                    bone_matrix: (skinning_fetch_bone_matrix, texture: boneMatrices, index: index_d),
                    position: pos,
                    weight: (maskW_vec4, v: boneWeights)
                )]
                [result := (add_vec4, a: result, b: weighted)]
                [(truncate_vec4, v: result) -> out_position]
            },
        );
        self
    }

    fn with_deformer_middleware(mut self) -> Self {
        self.add_function(code_material_function! {
            fn deformer_sample_curve(bezier_matrix: mat4, t: float, control_points_x: vec4, control_points_y: vec4) -> vec2 {
                "vec4 _power_series = vec4(1.0, t, t * t, t * t * t);
                vec4 _result_x = _power_series * bezier_matrix * control_points_x;
                vec4 _result_y = _power_series * bezier_matrix * control_points_y;
                return vec2(
                    _result_x.x + _result_x.y + _result_x.z + _result_x.w,
                    _result_y.x + _result_y.y + _result_y.z + _result_y.w
                );"
            }
        });
        self.add_middleware(
            "deformer".to_owned(),
            material_graph! {
                inputs {
                    [vertex] in position as in_position: vec3 = {vec3(0.0, 0.0, 0.0)};
                    [vertex] in curvesIndex: int = {0};

                    [vertex] uniform bezierCurves: sampler2D;
                    [vertex] in bezierMatrix: mat4 = {mat4([
                        [1.0, -3.0,  3.0, -1.0],
                        [0.0,  3.0, -6.0,  3.0],
                        [0.0,  0.0,  3.0, -3.0],
                        [0.0,  0.0,  0.0,  1.0],
                    ])};
                }

                outputs {
                    [vertex] out position as out_position: vec3;
                }

                [curve_top_x = (texelFetch2d,
                    sampler: bezierCurves,
                    coord: (make_ivec2, x: {0}, y: curvesIndex),
                    lod: {0}
                )]
                [curve_top_y = (texelFetch2d,
                    sampler: bezierCurves,
                    coord: (make_ivec2, x: {1}, y: curvesIndex),
                    lod: {0}
                )]
                [curve_bottom_x = (texelFetch2d,
                    sampler: bezierCurves,
                    coord: (make_ivec2, x: {2}, y: curvesIndex),
                    lod: {0}
                )]
                [curve_bottom_y = (texelFetch2d,
                    sampler: bezierCurves,
                    coord: (make_ivec2, x: {3}, y: curvesIndex),
                    lod: {0}
                )]
                [curve_left_x = (texelFetch2d,
                    sampler: bezierCurves,
                    coord: (make_ivec2, x: {4}, y: curvesIndex),
                    lod: {0}
                )]
                [curve_left_y = (texelFetch2d,
                    sampler: bezierCurves,
                    coord: (make_ivec2, x: {5}, y: curvesIndex),
                    lod: {0}
                )]
                [curve_right_x = (texelFetch2d,
                    sampler: bezierCurves,
                    coord: (make_ivec2, x: {6}, y: curvesIndex),
                    lod: {0}
                )]
                [curve_right_y = (texelFetch2d,
                    sampler: bezierCurves,
                    coord: (make_ivec2, x: {7}, y: curvesIndex),
                    lod: {0}
                )]
                [top = (deformer_sample_curve,
                    bezier_matrix: bezierMatrix,
                    t: (maskX_vec3, v: in_position),
                    control_points_x: curve_top_x,
                    control_points_y: curve_top_y
                )]
                [bottom = (deformer_sample_curve,
                    bezier_matrix: bezierMatrix,
                    t: (maskX_vec3, v: in_position),
                    control_points_x: curve_bottom_x,
                    control_points_y: curve_bottom_y
                )]
                [left = (deformer_sample_curve,
                    bezier_matrix: bezierMatrix,
                    t: (maskY_vec3, v: in_position),
                    control_points_x: curve_left_x,
                    control_points_y: curve_left_y
                )]
                [right = (deformer_sample_curve,
                    bezier_matrix: bezierMatrix,
                    t: (maskY_vec3, v: in_position),
                    control_points_x: curve_right_x,
                    control_points_y: curve_right_y
                )]
                [vertical = (mix_vec2,
                    x: top,
                    y: bottom,
                    alpha: (fill_vec2, v: (maskY_vec3, v: in_position))
                )]
                [horizontal = (mix_vec2,
                    x: left,
                    y: right,
                    alpha: (fill_vec2, v: (maskX_vec3, v: in_position))
                )]

                [(append_vec3,
                    a: (mix_vec2, x: vertical, y: horizontal, alpha: {vec2(0.5, 0.5)}),
                    b: (maskZ_vec3, v: in_position)
                ) -> out_position]
            },
        );
        self
    }
}

impl Default for MaterialLibrary {
    fn default() -> Self {
        Self {
            functions: Default::default(),
            domains: Default::default(),
            middlewares: Default::default(),
        }
        .with_angle_functions()
        .with_single_functions()
        .with_vector_functions()
        .with_matrix_functions()
        .with_compare_functions()
        .with_logic_functions()
        .with_texture_functions()
        .with_virtual_texture_function()
        .with_operator_functions()
        .with_cast_functions()
        .with_fill_functions()
        .with_make_functions()
        .with_mask_functions()
        .with_append_functions()
        .with_truncate_functions()
        .with_dithering()
        .with_vertanim_middleware()
        .with_skinning_middleware()
        .with_deformer_middleware()
    }
}
