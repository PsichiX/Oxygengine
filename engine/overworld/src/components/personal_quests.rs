use crate::resources::{market::Currency, quests::*};
use oxygengine_core::{
    prefab::{Prefab, PrefabComponent},
    Scalar,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub enum PersonalQuestsError {
    QuestAlreadyTaken(QuestId),
    QuestAlreadyCompleted(QuestId),
    QuestDoesNotExists(QuestId),
    QuestHasNoObjectives(QuestId),
    ObjectiveDoesNotExists(ObjectiveId),
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PersonalQuests {
    /// {quest id: {objective id: progress score}}
    active_quests: HashMap<QuestId, HashMap<ObjectiveId, Scalar>>,
    completed_quests: HashSet<QuestId>,
}

impl PersonalQuests {
    pub fn take<Q, B, V>(
        &mut self,
        id: QuestId,
        database: &QuestsDatabase<Q, B, V>,
    ) -> Result<(), PersonalQuestsError>
    where
        Q: std::fmt::Debug + Clone + Send + Sync,
        B: std::fmt::Debug + Clone + Send + Sync,
        V: Currency + std::fmt::Debug + Clone + Send + Sync,
    {
        if self.active_quests.contains_key(&id) {
            return Err(PersonalQuestsError::QuestAlreadyTaken(id));
        }
        if self.completed_quests.contains(&id) {
            return Err(PersonalQuestsError::QuestAlreadyCompleted(id));
        }
        let quest = match database.quest(id) {
            Some(quest) => quest,
            None => return Err(PersonalQuestsError::QuestDoesNotExists(id)),
        };
        if quest.objectives.is_empty() {
            return Err(PersonalQuestsError::QuestHasNoObjectives(id));
        }
        let objectives = quest.objectives.iter().map(|id| (*id, 0.0)).collect();
        self.active_quests.insert(id, objectives);
        Ok(())
    }

    pub fn leave(&mut self, id: QuestId) -> Result<(), PersonalQuestsError> {
        if self.active_quests.remove(&id).is_some() {
            Ok(())
        } else {
            Err(PersonalQuestsError::QuestDoesNotExists(id))
        }
    }

    pub fn active_quests(&self) -> impl Iterator<Item = QuestId> + '_ {
        self.active_quests.keys().copied()
    }

    pub fn completed_quests(&self) -> impl Iterator<Item = QuestId> + '_ {
        self.completed_quests.iter().copied()
    }

    pub fn progress(&self, id: QuestId) -> Result<Scalar, PersonalQuestsError> {
        match self.active_quests.get(&id) {
            Some(objectives) => {
                Ok(objectives.values().fold(0.0, |a, v| a + *v) / objectives.len() as Scalar)
            }
            None => Err(PersonalQuestsError::QuestDoesNotExists(id)),
        }
    }

    pub fn progress_objectives(
        &self,
        id: QuestId,
    ) -> Result<impl Iterator<Item = (ObjectiveId, Scalar)> + '_, PersonalQuestsError> {
        match self.active_quests.get(&id) {
            Some(objectives) => Ok(objectives.iter().map(|(id, score)| (*id, *score))),
            None => Err(PersonalQuestsError::QuestDoesNotExists(id)),
        }
    }

    pub fn complete(
        &mut self,
        id: ObjectiveId,
        score: Option<Scalar>,
    ) -> Result<(), PersonalQuestsError> {
        for objectives in self.active_quests.values_mut() {
            if let Some(s) = objectives.get_mut(&id) {
                *s = (*s + score.unwrap_or(1.0)).max(0.0).min(1.0);
            }
        }
        let to_remove = self
            .active_quests
            .iter()
            .filter(|(_, objectives)| objectives.values().all(|score| *score >= 1.0))
            .map(|(id, _)| *id)
            .collect::<Vec<_>>();
        for id in to_remove {
            self.active_quests.remove(&id);
            self.completed_quests.insert(id);
        }
        Ok(())
    }

    pub fn reset(&mut self, id: QuestId) -> Result<(), PersonalQuestsError> {
        match self.active_quests.get_mut(&id) {
            Some(objectives) => {
                for score in objectives.values_mut() {
                    *score = 0.0;
                }
                Ok(())
            }
            None => Err(PersonalQuestsError::QuestDoesNotExists(id)),
        }
    }

    pub fn transfer(
        &mut self,
        receiver: &mut Self,
        id: QuestId,
    ) -> Result<(), PersonalQuestsError> {
        if receiver.active_quests.contains_key(&id) {
            return Err(PersonalQuestsError::QuestAlreadyTaken(id));
        }
        if receiver.completed_quests.contains(&id) {
            return Err(PersonalQuestsError::QuestAlreadyCompleted(id));
        }
        if !self.active_quests.contains_key(&id) {
            return Err(PersonalQuestsError::QuestDoesNotExists(id));
        }
        let objectives = self.active_quests.remove(&id).unwrap();
        receiver.active_quests.insert(id, objectives);
        Ok(())
    }
}

impl Prefab for PersonalQuests {}
impl PrefabComponent for PersonalQuests {}
