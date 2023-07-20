# Hardware Accelerated rendering

Here we will take a deeper look at how Oxygengine's hardware Accelerated
rendering model works.

## Overview

HA renderer uses completely data-driven, material-graph based rendering approach.
The most important unit in HA renderer is Render Pipeline.

- Render Pipelines are containers for Render Targets and Render Stages.
- Render Targets are a GPU storage, more precisely dynamic textures that all
  rendered information gets stored into.
- Render Stages are containers for Render Queues and are used by dedicated render
  systems which define how entities (or any custom data in mater of fact) gets
  rendered into Render Target that Render Stage points to.
- Render systems work on cameras and record Render Commands into Render Queue
  assigned to given Render Stage.
- Render Stage also defines Material Domain (which is basically a shader backend)
  that will be linked with Material Graphs (which are shader frontends) of
  entities that gets rendered (more about how Materials works later on this page).

## Material-graph based rendering

While most game engines expose raw shaders to the users to make them tell exactly
how anything should be rendered, Oxygengine took another path and do not expose
any shader code, these are considered engine internals that user should never be
forced to write on their own. More than that, shaders gets baked at runtime (or
build-time, both material graphs and baked materials are assets) only when engine
finds that certain pair of material domain (backend) and material graph (frontend)
are gonna be used in rendering.

Material domain and material graphs always work in tandem, material domain job is
to preprocess vertex and uniform data and send it to material graph (via specific
interface that given domain defines) for it to postprocess that data and send it
back for material domain to store it properly in target outputs.

The reason for them being separate units is because HA renderer aims for in case
of user wanting to do some different visuals than default ones provided by the
engine, to focus only on the effect and don't bother writing additional logic just
to meet specialized vertex format and render target requirements. Another benefit
we get from this approach is that now user can make his frontend material once and
renderer will bake at runtime all variants needed by all pairs of domain and graph
materials.

This basically means that user can now get his material graph working without any
additional work with any vertex format and target format that renderer find
compatible at runtime. In even simpler words: imagine you have a material graph
that has to add outlines to the image, in that case no matter if you render your
entity for example in forward or deferred renderer, it will work for both by default
as long as both use material domains which provide domain node that your outline
material uses.

**IMPORTANT:** All shader variants for given material are considered unique as
long as they have different Material Signatures:
- Material Signature is defined by Material Mesh Signature + Material Render Target
  Signature + Domain name + Vertex Middlewares used.
- Material Mesh Signature is defined by unique Vertex Layout (vertex layouts are
  defined by meshes, to be more precise by the vertex format given mesh data uses).
- Material Render Target Signature is defined by set of render target output names.

## 1 picture say more than 1000 words

Let's take a look at the simplest material domain (this is code-side
representation of the material domain/graph):
```rust,ignore
material_graph! {
    inputs {
        [fragment] inout BaseColor: vec4 = {vec4(1.0, 1.0, 1.0, 1.0)};

        [vertex] uniform model: mat4;
        [vertex] uniform view: mat4;
        [vertex] uniform projection: mat4;

        [vertex] in position: vec3 = vec3(0.0, 0.0, 0.0);
        [vertex] in color: vec4 = vec4(1.0, 1.0, 1.0, 1.0);
    }

    outputs {
        [vertex] inout TintColor: vec4;
        [vertex] inout ScreenPosition: vec4;

        [vertex] builtin gl_Position: vec4;
        [fragment] out finalColor: vec4;
    }

    [model_view_projection = (mul_mat4,
      a: projection,
      b: (mul_mat4,
        a: view,
        b: model
      )
    )]
    [pos = (append_vec4, a: position, b: {1.0})]
    [screen_position = (mul_mat4_vec4, a: model_view_projection, b: pos)]

    [color -> TintColor]
    [screen_position -> ScreenPosition]
    [screen_position -> gl_Position]
    [BaseColor -> finalColor]
}
```

In this snippet we can see that this particular material domain expects `model` +
`view` + `projection` uniforms, as well as `position` + `color` vertex inputs,
and it writes data to `gl_Position` vertex output and `finalColor` target output.
This basically means that this material domain will work with stage that writes
to `finalColor` target output and `position` + `color` vertex format. It also
will bake shader variants for any material graph that **might read** `TintColor`
and/or `ScreenPosition` domain input, and **might write** `BaseColor` domain output.

Consider domain input/outputs to be purely an optional interface between material
domain and material graph. You might ask now: "why domain interface is optional?"
well, this is where this approach shines: you see, when material domain gets
combined with material graph, it will bake shader only from nodes that leads
directly from target outputs to vertex inputs, with all required nodes along the
way, every node not used in that path won't get compiled into shader variant.

Now let's take a look at the simplest material graph:
```rust,ignore
material_graph! {
    inputs {
        [vertex] inout TintColor: vec4 = {vec4(1.0, 1.0, 1.0, 1.0)};
    }

    outputs {
        [fragment] inout BaseColor: vec4;
    }

    [[TintColor => vColor] -> BaseColor]
}
```

Here in this material graph we can see we only use domain interface and just move
input color from vertex shader stage to fragment shader stage and send it back to
material domain to let it store properly for its render target outputs - when user
is making material graph, (s)he doesn't have to care about how to write to targets,
(s)he can only care how to process domain inputs into domain outputs and domain
takes care of properly storing data into target outputs.

Now imagine user wants to create material graph that do not use TintColor at all,
rather converts ScreenPosition into BaseColor:

```rust,ignore
material_graph! {
    inputs {
        [vertex] inout ScreenPosition: vec4 = {vec4(0.0, 0.0, 0.0, 0.0)};
    }

    outputs {
        [fragment] inout BaseColor: vec4;
    }

    [[ScreenPosition => vColor] -> BaseColor]
}
```

This material graph when combined with our previously defined material domain,
will bake shader with nodes that only use screen position calculated in domain
graph and not include color vertex data in the shader at all since this shader
variant does not use it. Now, do you also see the benefits of this over usual
`#ifdef`-ed raw shaders? You can focus on what effect you want to achieve
without caring about engine internals you work with, trying to define or even
limit yourself with effects.

## Material middlewares

Another concept used with material graphs is material middlewares - an ergonomic
way to "inject" other material graphs as material input preprocessor.

Consider you have a vertex format like this:
```rust,ignore
vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain, SurfaceTexturedDomain)
    pub struct SurfaceVertexPT {
        #[serde(default = "default_position")]
        pub position: vec3 = position(0, bounds),
        #[serde(default = "default_texture_coord")]
        pub texture_coord: vec3 = textureCoord(0),
    }
}
```
It is used to render for example regular entity sprites. Now you want to make
these sprites animated with skinning. Normally you would need to duplicate all
material domains that has to work with skinning which makes future changes and
general material maintanance subjective to being out of sync for starters - we
can avoid that problem entirely using material middlewares!

You start with defining skinned vertex format with skinning data only:
```rust,ignore
vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @middlewares(skinning)
    pub struct SurfaceSkinningFragment {
        #[serde(default = "default_bone_indices")]
        pub bone_indices: int = boneIndices(0),
        #[serde(default = "default_bone_weights")]
        pub bone_weights: vec4 = boneWeights(0),
    }
}
```
As you can see, we have marked this vertex format to use skinning middleware.
We also define compound vertex format that will make that sprite vertex format
mixed with skinned vertex format:
```rust,ignore
compound_vertex_type! {
    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    @tags(SurfaceDomain, SurfaceSkinnedDomain, SurfaceTexturedDomain)
    pub struct SurfaceVertexSPT {
        #[serde(default)]
        pub vertex: SurfaceVertexPT,
        #[serde(default)]
        pub skinning: SurfaceSkinningFragment,
    }
}
```

Now we have to add skinning middleware material graph to Material Library resource:
```rust,ignore
#library.add_function(graph_material_function! {
#    fn skinning_fetch_bone_matrix(texture: sampler2D, index: int) -> mat4 {
#        [return (make_mat4,
#            a: (texelFetch2d, sampler: texture, coord: (make_ivec2, x: {0}, y: index), lod: {0}),
#            b: (texelFetch2d, sampler: texture, coord: (make_ivec2, x: {1}, y: index), lod: {0}),
#            c: (texelFetch2d, sampler: texture, coord: (make_ivec2, x: {2}, y: index), lod: {0}),
#            d: (texelFetch2d, sampler: texture, coord: (make_ivec2, x: {3}, y: index), lod: {0})
#        )]
#    }
#});
#library.add_function(graph_material_function! {
#    fn skinning_weight_position(bone_matrix: mat4, position: vec4, weight: float) -> vec4 {
#        [return (mul_mat4_vec4,
#            a: bone_matrix,
#            b: (mul_vec4, a: position, b: (fill_vec4, v: weight))
#        )]
#    }
#});
library.add_middleware(
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
#
#        [pos = (append_vec4, a: in_position, b: {1.0})]
#        [index_a = (bitwise_and, a: boneIndices, b: {0xFF})]
#        [index_b = (bitwise_and, a: (bitwise_shift_right, v: boneIndices, bits: {8}), b: {0xFF})]
#        [index_c = (bitwise_and, a: (bitwise_shift_right, v: boneIndices, bits: {16}), b: {0xFF})]
#        [index_d = (bitwise_and, a: (bitwise_shift_right, v: boneIndices, bits: {24}), b: {0xFF})]
#        [result = (skinning_weight_position,
#            bone_matrix: (skinning_fetch_bone_matrix, texture: boneMatrices, index: index_a),
#            position: pos,
#            weight: (maskX_vec4, v: boneWeights)
#        )]
#        [weighted = (skinning_weight_position,
#            bone_matrix: (skinning_fetch_bone_matrix, texture: boneMatrices, index: index_b),
#            position: pos,
#            weight: (maskY_vec4, v: boneWeights)
#        )]
#        [result := (add_vec4, a: result, b: weighted)]
#        [weighted := (skinning_weight_position,
#            bone_matrix: (skinning_fetch_bone_matrix, texture: boneMatrices, index: index_c),
#            position: pos,
#            weight: (maskZ_vec4, v: boneWeights)
#        )]
#        [result := (add_vec4, a: result, b: weighted)]
#        [weighted := (skinning_weight_position,
#            bone_matrix: (skinning_fetch_bone_matrix, texture: boneMatrices, index: index_d),
#            position: pos,
#            weight: (maskW_vec4, v: boneWeights)
#        )]
#        [result := (add_vec4, a: result, b: weighted)]
#        [(truncate_vec4, v: result) -> out_position]
    },
);
```
What is important there, material middlewares has to define in and out pins that
it injects in between, so in case of skinning we essentially tell material
compiler we want to inject skinning before vertex position data gets passed to
actual material - we are making skinning middleware as vertex input preprocessor:
```rust,ignore
inputs {
  [vertex] in position as in_position: vec3 = {vec3(0.0, 0.0, 0.0)};
}
outputs {
  [vertex] out position as out_position: vec3;
}
```

Now whenever we want to render mesh with `SurfaceVertexSPT` vertex format (skinned
position texcoord), for every material that has to use it, there will be compiled
shader variant with skinning injected - no more duplicating materials with extra
features, when we can just inject these features (middlewares) directly into
materials that use them!

---

As you can see, all of this moves away burden of careful producing of all shader
code from user to the engine. With materials there is no more need for any
tedious, boilerplate-y and unnecessary `#ifdef`-ed shader code - we have reduced
the complexity of shader creation and management to bare minimum.
