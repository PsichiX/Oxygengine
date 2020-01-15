extern crate oxygengine_core as core;
#[macro_use]
extern crate lazy_static;

mod component;
pub mod integration;
pub mod interface;
pub mod scriptable;
pub mod state;
pub mod web_api;

pub mod prelude {
    pub use crate::{integration::*, interface::*, scriptable::*, state::*};
}
use crate::{
    component::WebScriptComponent, integration::core::AppLifeCycleScripted,
    interface::WebScriptInterface,
};
use core::{
    app::{AppBuilder, AppLifeCycle},
    hierarchy::{Name, NonPersistent, Tag},
};

pub fn bundle_installer<'a, 'b, WSS>(builder: &mut AppBuilder<'a, 'b>, mut web_script_setup: WSS)
where
    WSS: FnMut(&mut WebScriptInterface),
{
    builder.install_component::<WebScriptComponent>();
    WebScriptInterface::with(|interface| {
        interface.register_component_bridge("Name", Name::default());
        interface.register_component_bridge("Tag", Tag::default());
        interface.register_component_bridge("NonPersistent", NonPersistent::default());
        interface.register_resource_bridge::<AppLifeCycle, AppLifeCycleScripted>("AppLifeCycle");
        #[cfg(feature = "composite-renderer")]
        {
            use crate::integration::composite_renderer::*;
            use oxygengine_composite_renderer::component::*;
            use oxygengine_composite_renderer_backend_web::WebCompositeRenderer;
            interface
                .register_component_bridge("CompositeVisibility", CompositeVisibility::default());
            interface.register_component_bridge::<_, CompositeSurfaceCacheScripted>(
                "CompositeSurfaceCache",
                CompositeSurfaceCache::default(),
            );
            interface.register_component_bridge::<_, CompositeRenderable>(
                "CompositeRenderable",
                CompositeRenderable::default(),
            );
            interface.register_component_bridge(
                "CompositeRenderableStroke",
                CompositeRenderableStroke::default(),
            );
            interface.register_component_bridge::<_, CompositeTransformScripted>(
                "CompositeTransform",
                CompositeTransform::default(),
            );
            interface
                .register_component_bridge("CompositeRenderDepth", CompositeRenderDepth::default());
            interface
                .register_component_bridge("CompositeRenderAlpha", CompositeRenderAlpha::default());
            interface.register_component_bridge(
                "CompositeCameraAlignment",
                CompositeCameraAlignment::default(),
            );
            interface.register_component_bridge("CompositeEffect", CompositeEffect::default());
            interface.register_component_bridge("CompositeCamera", CompositeCamera::default());
            interface.register_component_bridge::<_, CompositeSpriteScripted>(
                "CompositeSprite",
                CompositeSprite::default(),
            );
            interface.register_component_bridge::<_, CompositeSpriteAnimationScripted>(
                "CompositeSpriteAnimation",
                CompositeSpriteAnimation::default(),
            );
            interface.register_component_bridge::<_, CompositeTilemapScripted>(
                "CompositeTilemap",
                CompositeTilemap::default(),
            );
            interface.register_component_bridge::<_, CompositeTilemapAnimationScripted>(
                "CompositeTilemapAnimation",
                CompositeTilemapAnimation::default(),
            );
            interface.register_component_bridge::<_, CompositeMapChunkScripted>(
                "CompositeMapChunk",
                CompositeMapChunk::default(),
            );
            interface
                .register_resource_bridge::<WebCompositeRenderer, WebCompositeRendererScripted>(
                    "WebCompositeRenderer",
                );
        }
        #[cfg(feature = "audio")]
        {
            use crate::integration::audio::*;
            use oxygengine_audio::component::*;
            interface.register_component_bridge::<_, AudioSourceScripted>(
                "AudioSource",
                AudioSource::default(),
            );
        }
        #[cfg(feature = "input")]
        {
            use crate::integration::input::*;
            use oxygengine_input::resource::*;
            interface.register_resource_bridge::<InputController, InputControllerMappingsScripted>(
                "InputControllerMappings",
            );
            interface.register_resource_bridge::<InputController, InputControllerStateScripted>(
                "InputControllerState",
            );
        }
        web_script_setup(interface);
    });
}
