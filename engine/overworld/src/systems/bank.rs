use crate::{
    components::wallet::*,
    resources::{bank::*, market::*},
};
use oxygengine_core::ecs::{life_cycle::EntityChanges, Comp, Entity, Universe, WorldRef};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct BankSystemCache {
    wallets: HashMap<Entity, BankAccountId>,
}

pub type BankSystemResources<'a, T> = (
    WorldRef,
    &'a EntityChanges,
    &'a mut Bank<T>,
    &'a mut BankSystemCache,
    Comp<&'a mut Wallet>,
);

pub fn bank_system<T>(universe: &mut Universe)
where
    T: Currency + std::fmt::Debug + Default + Send + Sync + 'static,
{
    let (world, changes, mut bank, mut cache, ..) =
        universe.query_resources::<BankSystemResources<T>>();

    for entity in changes.despawned() {
        if let Some(id) = cache.wallets.remove(&entity) {
            let _ = bank.remove_account(id);
        }
    }

    for (entity, wallet) in world.query::<&mut Wallet>().iter() {
        if wallet.account.is_none() {
            let id = bank.create_account();
            wallet.account = Some(id);
            cache.wallets.insert(entity, id);
        }
    }
}
