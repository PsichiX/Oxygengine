# Hardware Accelerated rendering

Here we will take a deeper look at how Oxygengine's hardware Accelerated rendering model works.

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

Material domain and material graphs always work in tandem, material domain job is to preprocess vertex and uniform data and send it to material graph (via specific interface that given domain defines) for it to post process that data and send it back for material domain to store it properly in target outputs.

The reason for them being separate units is because HA renderer aims for in case
of user wanting to do some different visuals than default ones provided by the
engine, to focus only on the effect and don't bother writing additional logic just
to meet specialized vertex format and render target requirements. Another benefit
we get from this approach is that now user can make his frontend material once and
renderer will bake at runtime all variants needed all pairs of domain and graph
materials. This basically means that user can now get his material graph working
without any additional work with any vertex format and target format that renderer
find compatible at runtime. In even simpler words: imagine you have a material
graph that has to add outlines to the image, in that case no matter if you render
your entity for example in forward or deferred renderer, it will work for both by
default as long as both use material domains which provide domain node that your
outline material uses.

**IMPORTANT:** all shader variants for given material are considered unique as
long as they have different Material Signatures:
- Material Signature is defined by Material Mesh Signature + Material Render
  Target Signature + Domain name
- Material Mesh Signature is defined by unique Vertex Layout (vertex layouts are
  defined by meshes, to be more precise by the vertex format given mesh data uses)
- Material Render Target Signature is defined by set of render target output
  names.

## 1 picture say more than 1000 words

Let's take a look at the simplest material domain (this is code-side
representation of the material domain/graph):
```rust,ignore
material_graph! {
    inputs {
        [fragment] domain BaseColor: vec4 = {vec4(1.0, 1.0, 1.0, 1.0)};

        [vertex] uniform model: mat4;
        [vertex] uniform view: mat4;
        [vertex] uniform projection: mat4;

        [vertex] in position: vec3 = vec3(0.0, 0.0, 0.0);
        [vertex] in color: vec4 = vec4(1.0, 1.0, 1.0, 1.0);
    }

    outputs {
        [vertex] domain TintColor: vec4;
        [vertex] domain ScreenPosition: vec4;

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

In this snippet we can see that this particular material domain expects model +
view + projection uniforms, as well as position + color vertex inputs, and it
writes data to gl_Position vertex output and finalColor target output. This
basically means that this material domain will work with stage that writes to
finalColor target output and position + color vertex format. It also will bake
shader variants for any material graph that **might read** TintColor and/or
ScreenPosition domain input, and **might write** BaseColor domain output.

Consider domain input/outputs to be purely an optional interface between material
domain and material graph. You might ask now: "why domain interface is
optional?" - well, this is where this approach shines too: you see, when material
domain  gets combined with material graph, it will bake shader only from nodes
that leads  directly from target outputs to vertex inputs, with all required
nodes along the way.

Now let's take a look at the simplest material graph:
```rust,ignore
material_graph! {
    inputs {
        [vertex] domain TintColor: vec4 = {vec4(1.0, 1.0, 1.0, 1.0)};
    }

    outputs {
        [fragment] domain BaseColor: vec4;
    }

    [[TintColor => vColor] -> BaseColor]
}
```

Here in this material graph we can see we only use domain interface and just move input color from vertex shader stage to fragment shader stage and send it back to material domain to let it store properly for its render target outputs - when user is making material graph, he doesn't have to care about how to write to targets, he can only care how to process domain inputs into domain outputs and domain takes care of properly storing data into target outputs.

Now imagine user wants to create material graph that do not use TintColor at all,
rather converts ScreenPosition into BaseColor:

```rust,ignore
material_graph! {
    inputs {
        [vertex] domain ScreenPosition: vec4 = {vec4(0.0, 0.0, 0.0, 0.0)};
    }

    outputs {
        [fragment] domain BaseColor: vec4;
    }

    [[ScreenPosition => vColor] -> BaseColor]
}
```

This material graph when combined with our previously defined material domain,
will bake shader with nodes that only use screen position calculated in domain
graph and not include color vertex data in the shader at all since this shader
variant does not use it. Now, do you also see the benefits of this over usual
`#ifdef`-ed raw shaders?

---

As you can see, all of this moves away a burden of careful producing of all
shader code from user to the engine. With materials there is no more need for any
tedious, boilerplate-y and unnecessary `#ifdef`-ed shader code - we have reduced
the complexity of shader creation and management.
