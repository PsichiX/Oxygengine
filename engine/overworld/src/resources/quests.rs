use crate::resources::market::{Currency, MarketItemId};
use oxygengine_core::id::ID;
use std::collections::{HashMap, HashSet};

pub type ObjectiveId = ID<Objective<()>>;
pub type QuestId = ID<Quest<(), ()>>;

#[derive(Debug, Clone)]
pub struct Objective<T>
where
    T: std::fmt::Debug + Clone + Send + Sync,
{
    pub data: T,
}

impl<T> Default for Objective<T>
where
    T: std::fmt::Debug + Default + Clone + Send + Sync,
{
    fn default() -> Self {
        Self {
            data: Default::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum QuestReward<V>
where
    V: Currency + std::fmt::Debug + Clone + Send + Sync,
{
    MarketItem(MarketItemId),
    Currency(V),
}

#[derive(Debug, Clone)]
pub struct Quest<T, V>
where
    T: std::fmt::Debug + Clone + Send + Sync,
    V: Currency + std::fmt::Debug + Clone + Send + Sync,
{
    pub data: T,
    pub objectives: HashSet<ObjectiveId>,
    pub rewards: Vec<QuestReward<V>>,
}

impl<T, V> Default for Quest<T, V>
where
    T: std::fmt::Debug + Default + Clone + Send + Sync,
    V: Currency + std::fmt::Debug + Clone + Send + Sync,
{
    fn default() -> Self {
        Self {
            data: Default::default(),
            objectives: Default::default(),
            rewards: Default::default(),
        }
    }
}

impl<T, V> Quest<T, V>
where
    T: std::fmt::Debug + Clone + Send + Sync,
    V: Currency + std::fmt::Debug + Clone + Send + Sync,
{
    pub fn new(data: T) -> Self {
        Self {
            data,
            objectives: Default::default(),
            rewards: Default::default(),
        }
    }

    pub fn with_objective(mut self, id: ObjectiveId) -> Self {
        self.objectives.insert(id);
        self
    }

    pub fn with_reward(mut self, reward: QuestReward<V>) -> Self {
        self.rewards.push(reward);
        self
    }
}

#[derive(Debug, Clone)]
pub struct QuestsDatabase<Q, B, V>
where
    Q: std::fmt::Debug + Clone + Send + Sync,
    B: std::fmt::Debug + Clone + Send + Sync,
    V: Currency + std::fmt::Debug + Clone + Send + Sync,
{
    objectives: HashMap<ObjectiveId, Objective<B>>,
    quests: HashMap<QuestId, Quest<Q, V>>,
}

impl<Q, B, V> Default for QuestsDatabase<Q, B, V>
where
    Q: std::fmt::Debug + Clone + Send + Sync,
    B: std::fmt::Debug + Clone + Send + Sync,
    V: Currency + std::fmt::Debug + Clone + Send + Sync,
{
    fn default() -> Self {
        Self {
            objectives: Default::default(),
            quests: Default::default(),
        }
    }
}

impl<Q, B, V> QuestsDatabase<Q, B, V>
where
    Q: std::fmt::Debug + Clone + Send + Sync,
    B: std::fmt::Debug + Clone + Send + Sync,
    V: Currency + std::fmt::Debug + Clone + Send + Sync,
{
    pub fn register_objective(&mut self, item: Objective<B>) -> ObjectiveId {
        let id = ObjectiveId::new();
        self.objectives.insert(id, item);
        id
    }

    pub fn register_many_objectives<'a>(
        &'a mut self,
        iter: impl Iterator<Item = Objective<B>> + 'a,
    ) -> impl Iterator<Item = ObjectiveId> + 'a {
        let reserve = iter.size_hint();
        let reserve = reserve.1.unwrap_or(reserve.0);
        self.objectives.reserve(reserve);
        iter.map(|item| self.register_objective(item))
    }

    pub fn unregister_objective(&mut self, id: ObjectiveId) -> Option<Objective<B>> {
        if let Some(objective) = self.objectives.remove(&id) {
            for quest in self.quests.values_mut() {
                quest.objectives.remove(&id);
            }
            return Some(objective);
        }
        None
    }

    pub fn unregister_many_objectives<'a>(
        &'a mut self,
        iter: impl Iterator<Item = ObjectiveId> + 'a,
    ) -> impl Iterator<Item = Objective<B>> + 'a {
        iter.filter_map(|id| self.unregister_objective(id))
    }

    pub fn objectives(&self) -> impl Iterator<Item = (ObjectiveId, &Objective<B>)> {
        self.objectives
            .iter()
            .map(|(id, objective)| (*id, objective))
    }

    pub fn objective(&self, id: ObjectiveId) -> Option<&Objective<B>> {
        self.objectives.get(&id)
    }

    pub fn objective_mut(&mut self, id: ObjectiveId) -> Option<&mut Objective<B>> {
        self.objectives.get_mut(&id)
    }

    pub fn find_objectives(
        &self,
        iter: impl Iterator<Item = ObjectiveId>,
    ) -> impl Iterator<Item = (ObjectiveId, &Objective<B>)> {
        iter.filter_map(|id| self.objective(id).map(|objective| (id, objective)))
    }

    pub fn contains_objective(&self, id: ObjectiveId) -> bool {
        self.objectives.contains_key(&id)
    }

    pub fn objective_id(&self, data: &B) -> Option<ObjectiveId>
    where
        B: PartialEq,
    {
        self.objectives
            .iter()
            .find(|(_, objective)| &objective.data == data)
            .map(|(id, _)| *id)
    }

    pub fn register_quest(&mut self, item: Quest<Q, V>) -> QuestId {
        let id = QuestId::new();
        self.quests.insert(id, item);
        id
    }

    pub fn register_many_quests<'a>(
        &'a mut self,
        iter: impl Iterator<Item = Quest<Q, V>> + 'a,
    ) -> impl Iterator<Item = QuestId> + 'a {
        let reserve = iter.size_hint();
        let reserve = reserve.1.unwrap_or(reserve.0);
        self.quests.reserve(reserve);
        iter.map(|item| self.register_quest(item))
    }

    pub fn unregister_quest(&mut self, id: QuestId) -> Option<Quest<Q, V>> {
        self.quests.remove(&id)
    }

    pub fn unregister_many_quests<'a>(
        &'a mut self,
        iter: impl Iterator<Item = QuestId> + 'a,
    ) -> impl Iterator<Item = Quest<Q, V>> + 'a {
        iter.filter_map(|id| self.unregister_quest(id))
    }

    pub fn quests(&self) -> impl Iterator<Item = (QuestId, &Quest<Q, V>)> {
        self.quests.iter().map(|(id, quest)| (*id, quest))
    }

    pub fn quest(&self, id: QuestId) -> Option<&Quest<Q, V>> {
        self.quests.get(&id)
    }

    pub fn quest_mut(&mut self, id: QuestId) -> Option<&mut Quest<Q, V>> {
        self.quests.get_mut(&id)
    }

    pub fn find_quests(
        &self,
        iter: impl Iterator<Item = QuestId>,
    ) -> impl Iterator<Item = (QuestId, &Quest<Q, V>)> {
        iter.filter_map(|id| self.quest(id).map(|quest| (id, quest)))
    }

    pub fn contains_quest(&self, id: QuestId) -> bool {
        self.quests.contains_key(&id)
    }

    pub fn quest_id(&self, data: &Q) -> Option<QuestId>
    where
        Q: PartialEq,
    {
        self.quests
            .iter()
            .find(|(_, quest)| &quest.data == data)
            .map(|(id, _)| *id)
    }
}
