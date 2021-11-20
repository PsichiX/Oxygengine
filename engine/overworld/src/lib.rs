pub mod components;
pub mod resources;
pub mod systems;

pub mod prelude {
    pub use crate::{
        components::{board_avatar::*, inventory::*, personal_quests::*, wallet::*},
        resources::{bank::*, board::*, market::*, quests::*},
        systems::{bank::*, board::*},
    };
}

use crate::{
    components::{
        board_avatar::BoardAvatar, inventory::Inventory, personal_quests::PersonalQuests,
        wallet::Wallet,
    },
    resources::{
        bank::Bank,
        board::Board,
        market::{Currency, MarketDatabase},
        quests::QuestsDatabase,
    },
    systems::{
        bank::{bank_system, BankSystemCache, BankSystemResources},
        board::{board_system, BoardSystemCache, BoardSystemResources},
    },
};
use oxygengine_core::{
    app::AppBuilder,
    ecs::pipeline::{PipelineBuilder, PipelineBuilderError},
    prefab::PrefabManager,
};

pub fn bundle_installer<I, V, Q, B, F, PB>(
    builder: &mut AppBuilder<PB>,
    mut f: F,
) -> Result<(), PipelineBuilderError>
where
    I: std::fmt::Debug + Clone + Send + Sync + 'static,
    V: Currency + std::fmt::Debug + Default + Clone + Send + Sync + 'static,
    Q: std::fmt::Debug + Clone + Send + Sync + 'static,
    B: std::fmt::Debug + Clone + Send + Sync + 'static,
    F: FnMut(&mut Bank<V>, &mut MarketDatabase<I, V>, &mut QuestsDatabase<Q, B, V>) -> Board,
    PB: PipelineBuilder,
{
    let mut bank = Bank::<V>::default();
    let mut market = MarketDatabase::<I, V>::default();
    let mut quests = QuestsDatabase::<Q, B, V>::default();
    let board = f(&mut bank, &mut market, &mut quests);

    builder.install_resource(board);
    builder.install_resource(bank);
    builder.install_resource(market);
    builder.install_resource(quests);
    builder.install_resource(BoardSystemCache::default());
    builder.install_resource(BankSystemCache::default());

    builder.install_system::<BoardSystemResources>("board", board_system, &[])?;
    builder.install_system::<BankSystemResources<V>>("bank", bank_system::<V>, &[])?;

    Ok(())
}

pub fn prefabs_installer(prefabs: &mut PrefabManager) {
    prefabs.register_component_factory::<BoardAvatar>("BoardAvatar");
    prefabs.register_component_factory::<Inventory>("Inventory");
    prefabs.register_component_factory::<PersonalQuests>("PersonalQuests");
    prefabs.register_component_factory::<Wallet>("Wallet");
}

#[cfg(test)]
mod tests {
    use crate::{
        components::{inventory::*, personal_quests::*, wallet::*},
        resources::{bank::*, board::*, market::*, quests::*},
    };

    #[test]
    fn test_send_sync() {
        fn foo<T>()
        where
            T: Send + Sync,
        {
            println!("{} is Send + Sync", std::any::type_name::<T>());
        }

        foo::<Inventory>();
        foo::<PersonalQuests>();
        foo::<Wallet>();
        foo::<Bank<()>>();
        foo::<Board>();
        foo::<MarketDatabase<(), ()>>();
        foo::<QuestsDatabase<(), (), ()>>();
    }
}
