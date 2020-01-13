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
use crate::{component::WebScriptComponent, interface::WebScriptInterface};
use core::app::AppBuilder;

pub fn bundle_installer<'a, 'b, WSS>(builder: &mut AppBuilder<'a, 'b>, mut web_script_setup: WSS)
where
    WSS: FnMut(&mut WebScriptInterface),
{
    builder.install_component::<WebScriptComponent>();
    WebScriptInterface::with(|interface| {
        #[cfg(feature = "composite-renderer")]
        {
            use crate::integration::composite_renderer::*;
            use oxygengine_composite_renderer::component::*;
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
        }
        web_script_setup(interface);
    });
}
