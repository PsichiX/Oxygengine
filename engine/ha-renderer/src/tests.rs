#![cfg(test)]

use crate::{
    graph_material_function,
    material::{common::*, domains::surface::*},
    material_graph,
    math::*,
    mesh::vertex_factory::*,
    render_target::*,
    resources::material_library::*,
};

macro_rules! material_signature {
    (
        mesh( $( $mesh_name:literal ),* )
        render_target( $( $render_target_name:literal ),* )
        $( domain( $domain_name:literal ) )?
        $( middlewares( $( $middleware:literal ),* ) )?
    ) => {
        unsafe {
            #[allow(unused_mut)]
            #[allow(unused_assignments)]
            let mut mesh = vec![];
            $(
                mesh.push(($mesh_name.to_string(), mesh.len()));
            )*
            let render_target = vec![ $( $render_target_name.to_string() ),* ];
            #[allow(unused_mut)]
            #[allow(unused_assignments)]
            let mut domain = None;
            $(
                domain = Some($domain_name.to_string());
            )?
            #[allow(unused_mut)]
            #[allow(unused_assignments)]
            let mut middlewares = core::utils::StringSequence::default();
            $(
                $(
                    middlewares.append($middleware);
                )*
            )?
            MaterialSignature::from_raw(mesh, render_target, domain, middlewares)
        }
    };
}

#[test]
fn test_material_graph() {
    let mut library = MaterialLibrary::default();
    let domain = surface_flat_domain_graph();
    println!("* domain graph: {:#?}", domain);
    library.add_domain("forward".to_owned(), domain.to_owned());
    {
        let graph = default_surface_flat_color_material_graph();
        println!("* material graph color: {:#?}", graph);
        let signature = material_signature! {
            mesh("position", "color")
            render_target("finalColor")
            domain("forward")
        };
        println!("* material graph color signature: {:#?}", signature);
        let baked = graph
            .bake(&signature, Some(&domain), &library, true)
            .unwrap()
            .unwrap();
        println!("* compiled vertex material graph color:\n{}", baked.vertex);
        println!(
            "* compiled fragment material graph color:\n{}",
            baked.fragment
        );
    }
    {
        let graph = default_surface_flat_texture_2d_material_graph();
        println!("* material graph texture: {:#?}", graph);
        let signature = material_signature! {
            mesh("position", "textureCoord", "color")
            render_target("finalColor")
            domain("forward")
        };
        println!("* material graph texture signature: {:#?}", signature);
        let baked = graph
            .bake(&signature, Some(&domain), &library, true)
            .unwrap()
            .unwrap();
        println!(
            "* compiled vertex material graph texture:\n{}",
            baked.vertex
        );
        println!(
            "* compiled fragment material graph texture:\n{}",
            baked.fragment
        );
    }
    {
        let graph = default_surface_flat_text_material_graph();
        println!("* material graph text: {:#?}", graph);
        let signature = material_signature! {
            mesh("position", "textureCoord", "color", "outline", "page", "thickness")
            render_target("finalColor")
            domain("forward")
        };
        println!("* material graph text signature: {:#?}", signature);
        let baked = graph
            .bake(&signature, Some(&domain), &library, true)
            .unwrap()
            .unwrap();
        println!("* compiled vertex material graph text:\n{}", baked.vertex);
        println!(
            "* compiled fragment material graph text:\n{}",
            baked.fragment
        );
    }
    {
        let graph = material_graph! {
            inputs {
                [vertex] inout ScreenPosition: vec3 = {vec3(0.0, 0.0, 0.0)};
            }

            outputs {
                [fragment] inout BaseColor: vec4;
            }

            [(append_vec4, a: [ScreenPosition => vScreenPos], b: {1.0}) -> BaseColor]
        };
        println!("* material graph: {:#?}", graph);
        let signature = material_signature! {
            mesh("position")
            render_target("finalColor")
            domain("forward")
        };
        println!("* material graph signature: {:#?}", signature);
        let baked = graph
            .bake(&signature, Some(&domain), &library, true)
            .unwrap()
            .unwrap();
        println!("* compiled vertex material graph:\n{}", baked.vertex);
        println!("* compiled fragment material graph:\n{}", baked.fragment);
    }
    {
        let graph = material_graph! {
            inputs {
                [fragment] builtin gl_FragCoord: vec4;
                [fragment] uniform ditherImage: sampler2D;
            }

            outputs {
                [fragment] inout BaseColor: vec4;
            }

            [coord = (maskXY_vec4, v: gl_FragCoord)]
            [int_coord = (cast_vec2_ivec2, v: coord)]
            [threshold = (dither_threshold, texture: ditherImage, coord: int_coord)]
            [(fill_vec4, v: threshold) -> BaseColor]
        };
        println!("* material graph: {:#?}", graph);
        let signature = material_signature! {
            mesh("position")
            render_target("finalColor")
            domain("forward")
        };
        println!("* material graph signature: {:#?}", signature);
        let baked = graph
            .bake(&signature, Some(&domain), &library, true)
            .unwrap()
            .unwrap();
        println!("* compiled vertex material graph:\n{}", baked.vertex);
        println!("* compiled fragment material graph:\n{}", baked.fragment);
    }
}

#[test]
fn test_graph_variants() {
    let library = MaterialLibrary::default();
    let graph = material_graph! {
        inputs {}

        outputs {
            [vertex] builtin gl_Position: vec4;
            [fragment] out outputA: vec4;
            [fragment] out outputB: vec4;
        }

        [{vec4(1.0, 0.0, 0.0, 1.0)} -> gl_Position]
        [{vec4(0.0, 1.0, 0.0, 1.0)} -> outputA]
        [{vec4(0.0, 0.0, 1.0, 1.0)} -> outputB]
    };
    println!("* material graph: {:#?}", graph);
    {
        let signature = material_signature! {
            mesh()
            render_target("outputA")
        };
        let baked = graph
            .bake(&signature, None, &library, true)
            .unwrap()
            .unwrap();
        println!("* VS variant A:\n{}", baked.vertex);
        println!("* FS variant A:\n{}", baked.fragment);
    }
    {
        let signature = material_signature! {
            mesh()
            render_target("outputB")
        };
        let baked = graph
            .bake(&signature, None, &library, true)
            .unwrap()
            .unwrap();
        println!("* VS variant B:\n{}", baked.vertex);
        println!("* FS variant B:\n{}", baked.fragment);
    }
}

#[test]
fn test_graph_function() {
    let library = MaterialLibrary::default();
    let add = graph_material_function! {
        fn add(a: vec3, b: vec3) -> vec3 {
            [return (add_vec3, a: a, b: b)]
        }
    };
    add.validate(&library).unwrap();
    println!("* `add` material function: {:#?}", add);
    let times_two = graph_material_function! {
        fn times_two(a: vec3) -> vec3 {
            [return (mul_vec3, a: a, b: {vec3(2.0, 2.0, 2.0)})]
        }
    };
    times_two.validate(&library).unwrap();
    println!("* `times_two` material function: {:#?}", times_two);
}

#[test]
fn test_material_middlewares() {
    let mut library = MaterialLibrary::default();

    let middleware_a = material_graph! {
        inputs {
            [vertex] in position as in_position: vec4 = {vec4(0.0, 0.0, 0.0, 0.0)};
        }

        outputs {
            [vertex] out position as out_position: vec4;
        }

        [(add_vec4, a: in_position, b: {vec4(1.0, 1.0, 1.0, 0.0)}) -> out_position]
    };
    println!("* middleware A graph: {:#?}", middleware_a);
    middleware_a.validate(&library).unwrap();
    library.add_middleware("a".to_owned(), middleware_a);

    let middleware_b = material_graph! {
        inputs {
            [vertex] in position as in_position: vec4 = {vec4(0.0, 0.0, 0.0, 0.0)};
        }

        outputs {
            [vertex] out position as out_position: vec4;
        }

        [(mul_vec4, a: in_position, b: {vec4(2.0, 2.0, 2.0, 0.0)}) -> out_position]
    };
    println!("* middleware B graph: {:#?}", middleware_b);
    middleware_b.validate(&library).unwrap();
    library.add_middleware("b".to_owned(), middleware_b);

    let domain = material_graph! {
        inputs {
            [vertex] in position: vec4 = {vec4(0.0, 0.0, 0.0, 0.0)};
            [vertex] inout Color: vec4 = {vec4(1.0, 1.0, 1.0, 1.0)};
        }

        outputs {
            [vertex] builtin gl_Position: vec4;
            [fragment] out output: vec4;
        }

        [position -> gl_Position]
        [[Color => vColor] -> output]
    };
    println!("* domain graph: {:#?}", domain);

    let graph = material_graph! {
        inputs {
            [vertex] in color: vec4 = {vec4(4.0, 3.0, 2.0, 1.0)};
        }

        outputs {
            [vertex] inout Color: vec4;
        }

        [color -> Color]
    };
    println!("* material graph: {:#?}", graph);

    let signature = material_signature! {
        mesh("position", "color")
        render_target("output")
        middlewares("a", "b")
    };
    let baked = graph
        .bake(&signature, Some(&domain), &library, true)
        .unwrap()
        .unwrap();
    println!("* VS variant A:\n{}", baked.vertex);
    println!("* FS variant A:\n{}", baked.fragment);

    MaterialLibrary::assert_material_compilation(
        &SurfaceVertexSP::vertex_layout().unwrap(),
        RenderTargetDescriptor::Main,
        &surface_flat_domain_graph(),
        &default_surface_flat_color_material_graph(),
    );
}

#[test]
fn test_compound_vertex_type() {
    println!(
        "* SurfaceVertexPT vertex layout: {:#?}",
        SurfaceVertexPT::vertex_layout()
    );
    println!(
        "* SurfaceVertexAPT vertex layout: {:#?}",
        SurfaceVertexAPT::vertex_layout()
    );
    println!(
        "* SurfaceVertexSPT vertex layout: {:#?}",
        SurfaceVertexSPT::vertex_layout()
    );
    println!(
        "* SurfaceVertexASPT vertex layout: {:#?}",
        SurfaceVertexASPT::vertex_layout()
    );
}
