use crate::resources::bank::*;
use oxygengine_core::prefab::{Prefab, PrefabComponent};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Wallet {
    #[serde(skip)]
    pub(crate) account: Option<BankAccountId>,
}

impl Wallet {
    pub fn account(&self) -> Option<BankAccountId> {
        self.account
    }
}

impl Prefab for Wallet {}
impl PrefabComponent for Wallet {}
