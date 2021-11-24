extern crate oxygengine_core as core;
extern crate oxygengine_input as input;

pub mod component;
pub mod resource;
pub mod system;
pub mod ui_theme_asset_protocol;

// reexport macros.
pub use raui_core::{
    post_hooks, pre_hooks,
    prelude::{MessageData, Prefab, PropsData},
    unpack_named_slots, widget,
};

pub mod prelude {
    pub use crate::{component::*, resource::*, system::*, ui_theme_asset_protocol::*};
}
pub mod raui {
    pub mod core {
        pub use raui_core::*;
    }
    pub mod material {
        pub use raui_material::*;
    }
}

use crate::{
    component::UserInterfaceView,
    raui::core::application::ProcessContext,
    resource::UserInterface,
    system::{user_interface_system, UserInterfaceSystemCache, UserInterfaceSystemResources},
};
use core::{
    app::AppBuilder,
    assets::database::AssetsDatabase,
    ecs::{
        pipeline::{PipelineBuilder, PipelineBuilderError},
        AccessType, ResQuery, ResRead, ResWrite,
    },
    prefab::PrefabManager,
};

pub fn bundle_installer<PB, Q>(
    builder: &mut AppBuilder<PB>,
    user_interface: UserInterface,
) -> Result<(), PipelineBuilderError>
where
    PB: PipelineBuilder,
    Q: AccessType + ResQuery + 'static,
    <Q as ResQuery>::Fetch: FeedProcessContext,
{
    builder.install_resource(user_interface);
    builder.install_resource(UserInterfaceSystemCache::default());
    builder.install_system::<UserInterfaceSystemResources<Q>>(
        "user-interface",
        user_interface_system::<Q>,
        &[],
    )?;
    Ok(())
}

pub fn prefabs_installer(prefabs: &mut PrefabManager) {
    prefabs.register_component_factory::<UserInterfaceView>("UserInterfaceView");
}

pub fn protocols_installer(database: &mut AssetsDatabase) {
    database.register(ui_theme_asset_protocol::UiThemeAssetProtocol);
}

pub trait FeedProcessContext
where
    Self: Sized,
{
    fn feed_process_context(self, _context: &mut ProcessContext) {}
}

impl FeedProcessContext for () {}

impl<T> FeedProcessContext for ResRead<T>
where
    T: 'static,
{
    fn feed_process_context(self, context: &mut ProcessContext) {
        context.insert_owned(self);
    }
}

impl<T> FeedProcessContext for ResWrite<T>
where
    T: 'static,
{
    fn feed_process_context(self, context: &mut ProcessContext) {
        context.insert_owned(self);
    }
}

impl<T> FeedProcessContext for Option<ResRead<T>>
where
    T: 'static,
{
    fn feed_process_context(self, context: &mut ProcessContext) {
        if let Some(resource) = self {
            context.insert_owned(resource);
        }
    }
}

impl<T> FeedProcessContext for Option<ResWrite<T>>
where
    T: 'static,
{
    fn feed_process_context(self, context: &mut ProcessContext) {
        if let Some(resource) = self {
            context.insert_owned(resource);
        }
    }
}

macro_rules! impl_feed_process_context {
    ( $( $ty:ident ),+ ) => {
        impl<$( $ty ),+> FeedProcessContext for ( $( $ty, )+ ) where $( $ty: FeedProcessContext ),+ {
            fn feed_process_context(self, context: &mut ProcessContext) {
                #[allow(non_snake_case)]
                let ( $( $ty, )+ ) = self;
                $( $ty.feed_process_context(context) );+
            }
        }
    }
}

impl_feed_process_context!(A);
impl_feed_process_context!(A, B);
impl_feed_process_context!(A, B, C);
impl_feed_process_context!(A, B, C, D);
impl_feed_process_context!(A, B, C, D, E);
impl_feed_process_context!(A, B, C, D, E, F);
impl_feed_process_context!(A, B, C, D, E, F, G);
impl_feed_process_context!(A, B, C, D, E, F, G, H);
impl_feed_process_context!(A, B, C, D, E, F, G, H, I);
impl_feed_process_context!(A, B, C, D, E, F, G, H, I, J);
impl_feed_process_context!(A, B, C, D, E, F, G, H, I, J, K);
impl_feed_process_context!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_feed_process_context!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_feed_process_context!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_feed_process_context!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
