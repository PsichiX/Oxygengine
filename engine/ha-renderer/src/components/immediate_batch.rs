use crate::material::domains::surface::{immediate::SurfaceImmediateFactory, SurfaceDomain};
use core::{
    prefab::{Prefab, PrefabComponent},
    Ignite,
};
use serde::{Deserialize, Serialize};

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct HaImmediateBatch<V>
where
    V: SurfaceDomain + Default + Copy + Send + Sync + 'static,
{
    #[serde(skip)]
    #[ignite(ignore)]
    pub factory: SurfaceImmediateFactory<V>,
}

impl<V> Prefab for HaImmediateBatch<V> where
    V: SurfaceDomain + Default + Copy + Send + Sync + 'static
{
}
impl<V> PrefabComponent for HaImmediateBatch<V> where
    V: SurfaceDomain + Default + Copy + Send + Sync + 'static
{
}
