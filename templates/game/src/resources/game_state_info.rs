#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct GameStatePlayerInfo {
    pub health: usize,
    pub health_capacity: usize,
    pub weapons: usize,
    pub weapons_capacity: usize,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct GameStateCombatInfo {
    pub health: usize,
    pub weapons: usize,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct GameStateInfo {
    pub player: GameStatePlayerInfo,
    pub combat: Option<GameStateCombatInfo>,
    pub area: Option<String>,
}
