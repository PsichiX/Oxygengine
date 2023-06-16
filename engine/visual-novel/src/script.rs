use crate::{dialogue::Dialogue, Color, Position, Scale};
use core::{prefab::Prefab, Scalar};
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum LogType {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    None,
    // (log type, message)
    Log(LogType, String),
    Label(String),
    Wait(Scalar),
    GoToScene(String),
    EndScene,
    ChangeSceneBackground(String),
    ShowCharacter(String),
    HideCharacter(String),
    HideAllCharacters,
    /// (character name, visibility percentage)
    ChangeCharacterVisibility(String, Scalar),
    /// (character name, color)
    ChangeCharacterNameColor(String, Color),
    /// (character name, position percentage)
    ChangeCharacterPosition(String, Position),
    /// (character name, alignment percentage)
    ChangeCharacterAlignment(String, Position),
    /// (character name, rotation percentage)
    ChangeCharacterRotation(String, Scalar),
    /// (character name, scale percentage)
    ChangeCharacterScale(String, Scale),
    /// (character name, style name)
    ChangeCharacterStyle(String, String),
    ChangeCameraPosition(Position),
    ChangeCameraRotation(Scalar),
    GoToLabel(String),
    GoToChapter(String),
    Parallel(Vec<Action>),
    ShowDialogue(Dialogue),
    HideDialogue,
}

impl Default for Action {
    fn default() -> Self {
        Self::None
    }
}

impl Prefab for Action {}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub name: String,
    pub actions: Vec<Action>,
}

impl Prefab for Chapter {}
