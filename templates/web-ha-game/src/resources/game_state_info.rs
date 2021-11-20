#[derive(Debug, Default)]
pub struct GameStatePlayerInfo {
    pub health: usize,
    pub strength: usize,
    pub level: usize,
}

#[derive(Debug, Default)]
pub struct GameStateCombatInfo {
    pub health: usize,
    pub level: usize,
}

#[derive(Debug, Default)]
pub struct GameStateInfo {
    pub player: GameStatePlayerInfo,
    pub combat: Option<GameStateCombatInfo>,
    pub area: Option<String>,
}
