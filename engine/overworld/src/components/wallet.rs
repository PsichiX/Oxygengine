use crate::resources::bank::*;
use oxygengine_core::{
    prefab::{Prefab, PrefabComponent},
    Ignite,
};
use serde::{Deserialize, Serialize};

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct Wallet {
    #[serde(skip)]
    #[ignite(ignore)]
    pub(crate) account: Option<BankAccountId>,
}

impl Wallet {
    pub fn account(&self) -> Option<BankAccountId> {
        self.account
    }
}

impl Prefab for Wallet {}

impl PrefabComponent for Wallet {}
