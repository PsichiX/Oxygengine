use crate::{
    background::Background,
    character::Character,
    dialogue::{ActiveDialogue, Dialogue, DialogueAction},
    scene::{ActiveScene, Scene},
    script::{Action, Chapter, LogType},
};
use core::{error, info, prefab::Prefab, warn, Ignite, Scalar};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Debug};

#[derive(Debug, Clone)]
pub enum StoryError {
    SceneDoesNotExists(String),
    ChapterDoesNotExists(String),
    ChapterHasNoActions(String),
    /// (chapter name, label name)
    ChapterLabelDoesNotExists(String, String),
    BackgroundDoesNotExists(String),
    CharacterDoesNotExists(String),
    ThereIsNoActiveScene,
    ThereIsNoActiveChapter,
    ThereIsNoDialogOptionToSelect,
    ThereIsNoDialogOptionToFocus,
    /// (option index, options count)
    TryingToSelectDialogueOptionWithWrongIndex(usize, usize),
    /// (option index, options count)
    TryingToFocusDialogueOptionWithWrongIndex(usize, usize),
}

#[derive(Debug, Clone)]
pub struct StoryDebugState {
    pub current_chapter: Option<(String, Action)>,
    pub active_scene: Option<String>,
    pub active_dialogue: Option<Dialogue>,
    pub in_progress: bool,
    pub characters_in_progress: bool,
    pub scenes_in_progress: bool,
    pub active_scene_in_progress: bool,
    pub active_dialogue_in_progress: bool,
    pub is_waiting_for_dialogue_option_selection: bool,
    pub dialogue_action_selected: Option<DialogueAction>,
    pub wait: Scalar,
    pub is_complete: bool,
}

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct Story {
    #[serde(default)]
    active_scene: ActiveScene,
    #[serde(default)]
    active_dialogue: ActiveDialogue,
    #[serde(default)]
    scenes: HashMap<String, Scene>,
    #[serde(default)]
    backgrounds: HashMap<String, Background>,
    #[serde(default)]
    characters: HashMap<String, Character>,
    #[serde(default)]
    chapters: HashMap<String, Chapter>,
    /// (chapter name, chapter action index)
    #[serde(default)]
    current_chapter: Option<(String, usize)>,
    #[serde(skip)]
    dialogue_action_selected: Option<DialogueAction>,
    #[serde(default)]
    wait: Scalar,
    #[serde(default)]
    paused: bool,
}

impl Prefab for Story {}

impl Story {
    pub fn initialize(&mut self) {
        self.active_scene.end();
        self.active_dialogue.end();
        for scene in self.scenes.values_mut() {
            scene.initialize();
        }
        for character in self.characters.values_mut() {
            character.initialize();
        }
    }

    pub fn register_scene(&mut self, mut scene: Scene) {
        scene.initialize();
        self.scenes.insert(scene.name.clone(), scene);
    }

    pub fn unregister_scene(&mut self, name: &str) -> Option<Scene> {
        self.scenes.remove(name)
    }

    pub fn register_background(&mut self, background: Background) {
        self.backgrounds.insert(background.name.clone(), background);
    }

    pub fn unregister_background(&mut self, name: &str) -> Option<Background> {
        self.backgrounds.remove(name)
    }

    pub fn register_character(&mut self, mut character: Character) {
        character.initialize();
        self.characters
            .insert(character.name().to_owned(), character);
    }

    pub fn unregister_character(&mut self, name: &str) -> Option<Character> {
        self.characters.remove(name)
    }

    pub fn register_chapter(&mut self, chapter: Chapter) {
        self.chapters.insert(chapter.name.clone(), chapter);
    }

    pub fn unregister_chapter(&mut self, name: &str) -> Option<Chapter> {
        self.chapters.remove(name)
    }

    pub fn backgrounds(&self) -> impl Iterator<Item = (&str, &Background)> {
        self.backgrounds.iter().map(|(k, v)| (k.as_str(), v))
    }

    pub fn background(&self, name: &str) -> Option<&Background> {
        self.backgrounds.get(name)
    }

    pub fn scenes(&self) -> impl Iterator<Item = (&str, &Scene)> {
        self.scenes.iter().map(|(k, v)| (k.as_str(), v))
    }

    pub fn scene(&self, name: &str) -> Option<&Scene> {
        self.scenes.get(name)
    }

    pub fn characters(&self) -> impl Iterator<Item = (&str, &Character)> {
        self.characters.iter().map(|(k, v)| (k.as_str(), v))
    }

    pub fn character(&self, name: &str) -> Option<&Character> {
        self.characters.get(name)
    }

    pub fn character_mut(&mut self, name: &str) -> Option<&mut Character> {
        self.characters.get_mut(name)
    }

    pub fn chapter(&self, name: &str) -> Option<&Chapter> {
        self.chapters.get(name)
    }

    pub fn active_scene(&self) -> &ActiveScene {
        &self.active_scene
    }

    pub fn active_dialogue(&self) -> &ActiveDialogue {
        &self.active_dialogue
    }

    pub fn current_chapter(&self) -> Option<(&str, usize)> {
        self.current_chapter.as_ref().map(|(n, i)| (n.as_str(), *i))
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn set_paused(&mut self, value: bool) {
        self.paused = value;
    }

    pub fn is_waiting_for_dialogue_option_selection(&self) -> bool {
        if let Some(dialogue) = self.active_dialogue.to() {
            !dialogue.options.is_empty()
        } else {
            false
        }
    }

    pub fn focus_dialogue_option(&mut self, index: Option<usize>) -> Result<(), StoryError> {
        if let Some(dialogue) = self.active_dialogue.to_mut() {
            if let Some(index) = index {
                if index >= dialogue.options.len() {
                    return Err(StoryError::TryingToFocusDialogueOptionWithWrongIndex(
                        index,
                        dialogue.options.len(),
                    ));
                }
                for (i, option) in dialogue.options.iter_mut().enumerate() {
                    option.focused.set(i == index);
                    option.focused.playing = true;
                }
            } else {
                for option in &mut dialogue.options {
                    option.focused.set(false);
                    option.focused.playing = true;
                }
            }
            Ok(())
        } else {
            Err(StoryError::ThereIsNoDialogOptionToFocus)
        }
    }

    pub fn select_dialogue_option(&mut self, index: usize) -> Result<(), StoryError> {
        if let Some(dialogue) = self.active_dialogue.to() {
            if let Some(option) = dialogue.options.get(index) {
                self.dialogue_action_selected = Some(option.action.clone());
                Ok(())
            } else {
                Err(StoryError::TryingToSelectDialogueOptionWithWrongIndex(
                    index,
                    dialogue.options.len(),
                ))
            }
        } else {
            Err(StoryError::ThereIsNoDialogOptionToSelect)
        }
    }

    pub fn unselect_dialogue_option(&mut self) {
        self.dialogue_action_selected = None;
    }

    pub fn go_to_scene(&mut self, name: &str) -> Result<(), StoryError> {
        if self.scenes.contains_key(name) {
            self.active_scene.set(Some(name.to_owned()));
            self.active_scene.playing = true;
            Ok(())
        } else {
            Err(StoryError::SceneDoesNotExists(name.to_owned()))
        }
    }

    pub fn end_scene(&mut self) {
        self.active_scene.set(None);
        self.active_scene.playing = true;
    }

    pub fn change_scene_background(&mut self, name: &str) -> Result<(), StoryError> {
        if self.backgrounds.contains_key(name) {
            if let Some(scene_name) = self.active_scene.to() {
                if let Some(scene) = self.scenes.get_mut(scene_name) {
                    scene.background_style.set(name.to_owned());
                    scene.background_style.playing = true;
                    Ok(())
                } else {
                    Err(StoryError::SceneDoesNotExists(scene_name.to_owned()))
                }
            } else {
                Err(StoryError::ThereIsNoActiveScene)
            }
        } else {
            Err(StoryError::BackgroundDoesNotExists(name.to_owned()))
        }
    }

    fn wait(&mut self, seconds: Scalar) {
        self.wait = seconds;
    }

    fn go_to_label(&self, name: &str) -> Result<usize, StoryError> {
        if let Some((chapter_name, _)) = &self.current_chapter {
            if let Some(chapter) = self.chapters.get(chapter_name) {
                if let Some(i) = chapter.actions.iter().position(|a| {
                    if let Action::Label(n) = a {
                        n == name
                    } else {
                        false
                    }
                }) {
                    Ok(i)
                } else {
                    Err(StoryError::ChapterLabelDoesNotExists(
                        chapter_name.to_owned(),
                        name.to_owned(),
                    ))
                }
            } else {
                Err(StoryError::ChapterDoesNotExists(chapter_name.to_owned()))
            }
        } else {
            Err(StoryError::ThereIsNoActiveChapter)
        }
    }

    pub fn jump_to_label(&mut self, name: &str) -> Result<(), StoryError> {
        if let Some((chapter_name, index)) = &mut self.current_chapter {
            if let Some(chapter) = self.chapters.get(chapter_name) {
                if let Some(i) = chapter.actions.iter().position(|a| {
                    if let Action::Label(n) = a {
                        n == name
                    } else {
                        false
                    }
                }) {
                    *index = i;
                    Ok(())
                } else {
                    Err(StoryError::ChapterLabelDoesNotExists(
                        chapter_name.to_owned(),
                        name.to_owned(),
                    ))
                }
            } else {
                Err(StoryError::ChapterDoesNotExists(chapter_name.to_owned()))
            }
        } else {
            Err(StoryError::ThereIsNoActiveChapter)
        }
    }

    pub fn run_chapter(&mut self, name: &str) -> Result<(), StoryError> {
        if let Some(chapter) = self.chapters.get(name) {
            if !chapter.actions.is_empty() {
                self.current_chapter = Some((name.to_owned(), 0));
                Ok(())
            } else {
                Err(StoryError::ChapterHasNoActions(name.to_owned()))
            }
        } else {
            Err(StoryError::ChapterDoesNotExists(name.to_owned()))
        }
    }

    pub fn in_progress(&self) -> bool {
        self.characters.values().any(|c| c.is_dirty())
            || self.scenes.values().any(|s| s.in_progress())
            || self.active_scene.in_progress()
            || self.active_dialogue.in_progress()
            || self.is_waiting_for_dialogue_option_selection()
            || self.wait > 0.0
    }

    pub fn is_dirty(&self) -> bool {
        self.in_progress()
            || self
                .active_dialogue
                .to()
                .as_ref()
                .map_or(false, |d| d.is_dirty())
    }

    pub fn is_complete(&self) -> bool {
        self.current_chapter.is_none()
    }

    pub fn debug_state(&self) -> StoryDebugState {
        StoryDebugState {
            current_chapter: if let Some((name, index)) = &self.current_chapter {
                if let Some(chapter) = self.chapters.get(name) {
                    chapter
                        .actions
                        .get(*index)
                        .cloned()
                        .map(|action| (name.to_owned(), action))
                } else {
                    None
                }
            } else {
                None
            },
            active_scene: self.active_scene.to().clone(),
            active_dialogue: self.active_dialogue.to().clone(),
            in_progress: self.in_progress(),
            characters_in_progress: self.characters.values().any(|c| c.is_dirty()),
            scenes_in_progress: self.scenes.values().any(|s| s.in_progress()),
            active_scene_in_progress: self.active_scene.in_progress(),
            active_dialogue_in_progress: self.active_dialogue.in_progress(),
            is_waiting_for_dialogue_option_selection: self
                .is_waiting_for_dialogue_option_selection(),
            dialogue_action_selected: self.dialogue_action_selected.clone(),
            wait: self.wait,
            is_complete: self.is_complete(),
        }
    }

    pub fn process(&mut self, delta_time: Scalar) -> Result<(), StoryError> {
        if self.paused {
            return Ok(());
        }

        for character in self.characters.values_mut() {
            character.process(delta_time);
        }
        for scene in self.scenes.values_mut() {
            scene.process(delta_time);
        }
        if self.wait > 0.0 {
            self.wait -= delta_time;
        }
        self.active_scene.process(delta_time);
        self.active_dialogue.process(delta_time);
        if let Some(dialogue) = self.active_dialogue.from_mut() {
            dialogue.process(delta_time);
        }
        if let Some(dialogue) = self.active_dialogue.to_mut() {
            dialogue.process(delta_time);
        }
        if let Some(action) = std::mem::replace(&mut self.dialogue_action_selected, None) {
            match action {
                DialogueAction::JumpToLabel(name) => self.jump_to_label(&name)?,
                DialogueAction::JumpToChapter(name) => self.run_chapter(&name)?,
                _ => {}
            }
            self.active_dialogue.set(None);
            self.active_dialogue.playing = true;
        }
        if !self.in_progress() {
            if let Some((chapter_name, index)) = std::mem::replace(&mut self.current_chapter, None)
            {
                let meta = if let Some(chapter) = self.chapters.get(&chapter_name) {
                    chapter
                        .actions
                        .get(index)
                        .cloned()
                        .map(|a| (a, chapter.actions.len()))
                } else {
                    None
                };
                self.current_chapter = if let Some((action, count)) = meta {
                    self.run_action(action, count, chapter_name, index)?
                } else {
                    None
                };
            }
        }
        Ok(())
    }

    fn run_action(
        &mut self,
        action: Action,
        count: usize,
        chapter_name: String,
        mut index: usize,
    ) -> Result<Option<(String, usize)>, StoryError> {
        Ok(match action {
            Action::None | Action::Label(_) => {
                index += 1;
                if index < count {
                    Some((chapter_name, index))
                } else {
                    None
                }
            }
            Action::Log(log_type, message) => {
                match log_type {
                    LogType::Info => info!("[VN STORY] {}", message),
                    LogType::Warning => warn!("[VN STORY] {}", message),
                    LogType::Error => error!("[VN STORY] {}", message),
                }
                index += 1;
                if index < count {
                    Some((chapter_name, index))
                } else {
                    None
                }
            }
            Action::Wait(seconds) => {
                self.wait(seconds);
                index += 1;
                if index < count {
                    Some((chapter_name, index))
                } else {
                    None
                }
            }
            Action::GoToScene(name) => {
                self.go_to_scene(&name)?;
                index += 1;
                if index < count {
                    Some((chapter_name, index))
                } else {
                    None
                }
            }
            Action::EndScene => {
                self.end_scene();
                index += 1;
                if index < count {
                    Some((chapter_name, index))
                } else {
                    None
                }
            }
            Action::ChangeSceneBackground(name) => {
                self.change_scene_background(&name)?;
                index += 1;
                if index < count {
                    Some((chapter_name, index))
                } else {
                    None
                }
            }
            Action::ShowCharacter(name) => {
                if let Some(character) = self.characters.get_mut(&name) {
                    character.set_visibility(1.0);
                } else {
                    return Err(StoryError::CharacterDoesNotExists(name));
                }
                index += 1;
                if index < count {
                    Some((chapter_name, index))
                } else {
                    None
                }
            }
            Action::HideCharacter(name) => {
                if let Some(character) = self.characters.get_mut(&name) {
                    character.set_visibility(0.0);
                } else {
                    return Err(StoryError::CharacterDoesNotExists(name));
                }
                index += 1;
                if index < count {
                    Some((chapter_name, index))
                } else {
                    None
                }
            }
            Action::HideAllCharacters => {
                for character in self.characters.values_mut() {
                    character.set_visibility(0.0);
                }
                index += 1;
                if index < count {
                    Some((chapter_name, index))
                } else {
                    None
                }
            }
            Action::ChangeCharacterVisibility(name, value) => {
                if let Some(character) = self.characters.get_mut(&name) {
                    character.set_visibility(value);
                } else {
                    return Err(StoryError::CharacterDoesNotExists(name));
                }
                index += 1;
                if index < count {
                    Some((chapter_name, index))
                } else {
                    None
                }
            }
            Action::ChangeCharacterNameColor(name, value) => {
                if let Some(character) = self.characters.get_mut(&name) {
                    character.set_name_color(value);
                } else {
                    return Err(StoryError::CharacterDoesNotExists(name));
                }
                index += 1;
                if index < count {
                    Some((chapter_name, index))
                } else {
                    None
                }
            }
            Action::ChangeCharacterPosition(name, value) => {
                if let Some(character) = self.characters.get_mut(&name) {
                    character.set_position(value);
                } else {
                    return Err(StoryError::CharacterDoesNotExists(name));
                }
                index += 1;
                if index < count {
                    Some((chapter_name, index))
                } else {
                    None
                }
            }
            Action::ChangeCharacterAlignment(name, value) => {
                if let Some(character) = self.characters.get_mut(&name) {
                    character.set_alignment(value);
                } else {
                    return Err(StoryError::CharacterDoesNotExists(name));
                }
                index += 1;
                if index < count {
                    Some((chapter_name, index))
                } else {
                    None
                }
            }
            Action::ChangeCharacterRotation(name, value) => {
                if let Some(character) = self.characters.get_mut(&name) {
                    character.set_rotation(value);
                } else {
                    return Err(StoryError::CharacterDoesNotExists(name));
                }
                index += 1;
                if index < count {
                    Some((chapter_name, index))
                } else {
                    None
                }
            }
            Action::ChangeCharacterScale(name, value) => {
                if let Some(character) = self.characters.get_mut(&name) {
                    character.set_scale(value);
                } else {
                    return Err(StoryError::CharacterDoesNotExists(name));
                }
                index += 1;
                if index < count {
                    Some((chapter_name, index))
                } else {
                    None
                }
            }
            Action::ChangeCharacterStyle(name, value) => {
                if let Some(character) = self.characters.get_mut(&name) {
                    character.set_style(value);
                } else {
                    return Err(StoryError::CharacterDoesNotExists(name));
                }
                index += 1;
                if index < count {
                    Some((chapter_name, index))
                } else {
                    None
                }
            }
            Action::ChangeCameraPosition(value) => {
                if let Some(scene_name) = self.active_scene.to() {
                    if let Some(scene) = self.scenes.get_mut(scene_name) {
                        scene.camera_position.set(value);
                        scene.camera_position.playing = true;
                    } else {
                        return Err(StoryError::SceneDoesNotExists(scene_name.to_owned()));
                    }
                } else {
                    return Err(StoryError::ThereIsNoActiveScene);
                }
                index += 1;
                if index < count {
                    Some((chapter_name, index))
                } else {
                    None
                }
            }
            Action::ChangeCameraRotation(value) => {
                if let Some(scene_name) = self.active_scene.to() {
                    if let Some(scene) = self.scenes.get_mut(scene_name) {
                        scene.camera_rotation.set(value);
                        scene.camera_rotation.playing = true;
                    } else {
                        return Err(StoryError::SceneDoesNotExists(scene_name.to_owned()));
                    }
                } else {
                    return Err(StoryError::ThereIsNoActiveScene);
                }
                index += 1;
                if index < count {
                    Some((chapter_name, index))
                } else {
                    None
                }
            }
            Action::GoToLabel(name) => {
                index = self.go_to_label(&name)?;
                Some((chapter_name, index))
            }
            Action::GoToChapter(name) => {
                if let Some(chapter) = self.chapters.get(&name) {
                    if !chapter.actions.is_empty() {
                        Some((name, 0))
                    } else {
                        return Err(StoryError::ChapterHasNoActions(name));
                    }
                } else {
                    return Err(StoryError::ChapterDoesNotExists(name));
                }
            }
            Action::Parallel(actions) => {
                for action in actions {
                    self.run_action(action, count, chapter_name.clone(), index)?;
                }
                index += 1;
                if index < count {
                    Some((chapter_name, index))
                } else {
                    None
                }
            }
            Action::ShowDialogue(dialogue) => {
                self.active_dialogue.set(Some(dialogue));
                self.active_dialogue.playing = true;
                index += 1;
                if index < count {
                    Some((chapter_name, index))
                } else {
                    None
                }
            }
            Action::HideDialogue => {
                self.active_dialogue.set(None);
                self.active_dialogue.playing = true;
                index += 1;
                if index < count {
                    Some((chapter_name, index))
                } else {
                    None
                }
            }
        })
    }
}
