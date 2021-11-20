use crate::utils::spinbot::*;
use oxygengine::user_interface::raui::core::prelude::*;

#[derive(Debug, Default)]
pub struct UiBridge {
    battle: Option<UiBridgeBattle>,
}

#[derive(Debug, Default)]
pub struct UiBridgeBattle(Vec<UiBridgeBattlePlayer>);

#[derive(Debug, Default)]
pub struct UiBridgeBattlePlayer {
    pub health: DataBinding<Scalar>,
    pub power: DataBinding<SpinBotPower>,
}
