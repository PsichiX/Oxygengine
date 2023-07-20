# Render Pipeline

## Overview

HA renderer requires user to define render pipelines on game app setup phase.
Render pipelines describe how camera that uses given render pipeline, should
render world entities.

Let's take a look at typical renderer setup first:
```rust,ignore
HaRenderer::new(WebPlatformInterface::with_canvas_id(
    "screen",
    WebContextOptions::default(),
)?)
.with_stage::<RenderForwardStage>("forward")
.with_stage::<RenderGizmoStage>("gizmos")
.with_stage::<RenderUiStage>("ui")
.with_pipeline(
    "default",
    PipelineDescriptor::default()
        .render_target("main", RenderTargetDescriptor::Main)
        .stage(
            StageDescriptor::new("forward")
                .render_target("main")
                .domain("@material/domain/surface/flat")
                .clear_settings(ClearSettings {
                    color: Some(Rgba::gray(0.2)),
                    depth: false,
                    stencil: false,
                }),
        )
        .debug_stage(
            StageDescriptor::new("gizmos")
                .render_target("main")
                .domain("@material/domain/gizmo"),
        )
        .stage(
            StageDescriptor::new("ui")
                .render_target("main")
                .domain("@material/domain/surface/flat"),
        ),
)
```

From that code snippet we can tell than render pipelines contains:
- set of render targets used to render into.
- set of render stages that tells how to render geometry to given render target.

What is important here is that for render stages we are required to provide its
render target name for stage to know where to store all information it produces,
as well as domain graph name which is the shader backend for all shader frontends
(material graphs) used by entities in the world.

## How it works

#### Recording to Render Queue

At first, renderer searches for new camera components, for each camera it creates its own instance of render pipeline that camera component points at.

Then all render stage systems goes through all cameras that contain render stages
with stage type given system provides, here is a brief snippet example:
```rust,ignore
pub fn ha_render_gizmo_stage_system(universe: &mut Universe) {
#    type V = GizmoVertex;
#
    let (
        world,
        mut renderer,
        lifecycle,
        mut gizmos,
        material_mapping,
        image_mapping,
        mut cache,
        ..,
    ) = universe.query_resources::<HaRenderGizmoStageSystemResources>();

#    if gizmos.factory.is_empty() {
#        return;
#    }
#
#    let layout = match V::vertex_layout() {
#        Ok(layout) => layout,
#        Err(_) => return,
#    };
#
#    let mesh_id = match cache.mesh {
#        Some(mesh_id) => mesh_id,
#        None => {
#            let mut m = Mesh::new(layout.to_owned());
#            m.set_regenerate_bounds(false);
#            m.set_vertex_storage_all(BufferStorage::Dynamic);
#            m.set_index_storage(BufferStorage::Dynamic);
#            match renderer.add_mesh(m) {
#                Ok(mesh_id) => {
#                    cache.mesh = Some(mesh_id);
#                    mesh_id
#                }
#                Err(_) => return,
#            }
#        }
#    };
#    match renderer.mesh_mut(mesh_id) {
#        Some(mesh) => match gizmos.factory.factory() {
#            Ok(factory) => {
#                if factory.write_into(mesh).is_err() {
#                    return;
#                }
#            }
#            Err(_) => return,
#        },
#        None => return,
#    }
#
#    gizmos
#        .material
#        .update_references(&material_mapping, &image_mapping);
#    let material_id = match gizmos.material.reference.id().copied() {
#        Some(material_id) => material_id,
#        None => return,
#    };
#    let time = vec4(
#        lifecycle.time_seconds(),
#        lifecycle.delta_time_seconds(),
#        lifecycle.time_seconds().fract(),
#        0.0,
#    );
#
    for (_, (visibility, camera, transform)) in world
        .query::<(Option<&HaVisibility>, &HaCamera, &HaTransform)>()
        .iter()
    {
        if !visibility.map(|v| v.0).unwrap_or(true) {
            continue;
        }
        let iter = match camera.record_to_pipeline_stage::<RenderGizmoStage>(&renderer, transform) {
            Some(iter) => iter,
            None => continue,
        };
        for (info, render_queue) in iter {
            let mut render_queue = match render_queue.write() {
                Ok(render_queue) => render_queue,
                Err(_) => continue,
            };
            render_queue.clear();
            let mut recorder = render_queue.auto_recorder(None);
#
            let _ = recorder.record(RenderCommand::ActivateMesh(mesh_id));
#            let signature = info.make_material_signature(&layout);
            let _ = recorder.record(RenderCommand::ActivateMaterial(
                material_id,
                signature.to_owned(),
            ));
#            let _ = recorder.record(RenderCommand::OverrideUniform(
#                MODEL_MATRIX_NAME.into(),
#                Mat4::identity().into(),
#            ));
#            let _ = recorder.record(RenderCommand::OverrideUniform(
#                VIEW_MATRIX_NAME.into(),
#                info.view_matrix.into(),
#            ));
            let _ = recorder.record(RenderCommand::OverrideUniform(
                PROJECTION_MATRIX_NAME.into(),
                info.projection_matrix.into(),
            ));
#            let _ = recorder.record(RenderCommand::OverrideUniform(
#                TIME_NAME.into(),
#                time.into(),
#            ));
#            for (key, value) in &gizmos.material.values {
#                let _ = recorder.record(RenderCommand::OverrideUniform(
#                    key.to_owned().into(),
#                    value.to_owned(),
#                ));
#            }
#            if let Some(draw_options) = &gizmos.material.override_draw_options {
#                let _ = recorder.record(RenderCommand::ApplyDrawOptions(draw_options.to_owned()));
#            }
            let _ = recorder.record(RenderCommand::DrawMesh(MeshDrawRange::All));
#            let _ = recorder.record(RenderCommand::ResetUniforms);
            let _ = recorder.record(RenderCommand::SortingBarrier);
        }
    }
#
#    gizmos.factory.clear();
}
```

*You can toggle full code reveal to see what is actually happening there.*

As you can see, when we get an iterator over requested render stages for cameras,
all we do next is to get access to render queue, create auto recorder (to ease
writing ordered render commands) and start recording commands. Although Gizmo
render system renders already batched gizmo geometry in other render systems, you
can get the idea that all what recording phase cares about is to just record
render commands into render queue of given camera render pipeline, it doesn't
really matter where we get data from, what matters is what gets into render
queue. We could also just iterate over world entities and record their render
commands to the queue - in matter of fact, this is how Render Forward Stage does
that, here we show Render Gizmo Stage for the sake of simplified explanation.

#### Execution of render queues

After all render stage systems complete recording commands into queues, renderer
is now ready to go through all active render pipelines and execute their render
queues full of previously made records.

You can remember that when we were talking about recording to Render Queues, we
have been mentioning auto ordered recordings of commands - but what does that
means? Well, sometimes your render stage system might require its commands to be
ordered by for example some kind of depth value. For this case, to not require
user to collect entities to sort them and then record them in proper order, we
just encode order information in render command group index and enable optional
render queue sorting in stage descriptor. That way we do not break unspecified
order of entities iteration and just sort render commands itself. This obviously
has its own cost so it's just an optional step and you should definitely
benchmark to decide which of either render commands sorting or manual entity
sorting approach will benefit more your stage rendering.

Another thing worth mentioning about render queues is that they are only data
containers so you can for example create your own render queues separately from
what render pipeline provides, for example as a way of caching queues and reusing
them with multiple pipelines by flushing your custom render queue into one provided
by the render pipeline. Yet another use of render queues is to instead of recording
them in your application, you can send them via network socket to render it on
client application that will reflect your camera setup, but instead of recording
world itself, it will render what server sends - similar use case could be for
making both game and editor worlds embeded in one application host and game world
sending its recorded queues to editor world which then renders game view in its
rendering context.
