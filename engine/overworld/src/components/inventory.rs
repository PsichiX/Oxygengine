use crate::resources::market::*;
use oxygengine_core::{
    prefab::{Prefab, PrefabComponent},
    Ignite, Scalar,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum InventoryError {
    InventoryFull,
    EmptyItem(MarketItemId),
    ItemDoesNotExists(MarketItemId),
    /// (item id, owned item count, removing item count)
    RemovingTooMuchItems(MarketItemId, usize, usize),
    /// (item id, item count)
    CannotGiveItem(MarketItemId, usize),
    /// (item id, item count)
    CannotReceiveItem(MarketItemId, usize),
}

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct Inventory {
    capacity_weight: Option<Scalar>,
    items: HashMap<MarketItemId, usize>,
    items_resize: usize,
}

impl Inventory {
    pub fn new(capacity_weight: Option<Scalar>) -> Self {
        Self::with_resize(capacity_weight, 10)
    }

    pub fn with_resize(capacity_weight: Option<Scalar>, items_resize: usize) -> Self {
        Self {
            capacity_weight,
            items: HashMap::with_capacity(items_resize),
            items_resize,
        }
    }

    pub fn items(&self) -> impl Iterator<Item = (MarketItemId, usize)> + '_ {
        self.items.iter().map(|(id, count)| (*id, *count))
    }

    pub fn contains(&self, id: MarketItemId) -> Option<usize> {
        self.items.get(&id).copied()
    }

    pub fn can_add<T, V>(
        &self,
        id: MarketItemId,
        count: usize,
        database: &MarketDatabase<T, V>,
    ) -> bool
    where
        T: std::fmt::Debug + Clone + Send + Sync,
        V: Currency + std::fmt::Debug + Clone + Send + Sync,
    {
        count != 0
            && database
                .item(id)
                .map(|item| {
                    self.capacity_weight
                        .map(|capacity_weight| item.weight * count as Scalar <= capacity_weight)
                        .unwrap_or(true)
                })
                .unwrap_or_default()
    }

    pub fn add<T, V>(
        &mut self,
        id: MarketItemId,
        count: usize,
        database: &MarketDatabase<T, V>,
    ) -> Result<(), InventoryError>
    where
        T: std::fmt::Debug + Clone + Send + Sync,
        V: Currency + std::fmt::Debug + Clone + Send + Sync,
    {
        if count == 0 {
            return Err(InventoryError::EmptyItem(id));
        }
        let item = match database.item(id) {
            Some(item) => item,
            None => return Err(InventoryError::ItemDoesNotExists(id)),
        };
        if let Some(capacity_weight) = self.capacity_weight {
            if item.weight * count as Scalar > capacity_weight {
                return Err(InventoryError::InventoryFull);
            }
        }
        *self.items.entry(id).or_default() += count;
        Ok(())
    }

    pub fn can_remove(&self, id: MarketItemId, count: usize) -> bool {
        count != 0
            && self
                .items
                .get(&id)
                .copied()
                .and_then(|c| c.checked_sub(count))
                .is_some()
    }

    pub fn remove(&mut self, id: MarketItemId, count: usize) -> Result<(), InventoryError> {
        if count == 0 {
            return Err(InventoryError::EmptyItem(id));
        }
        if let Some(c) = self.items.get(&id).copied() {
            if let Some(c) = c.checked_sub(count) {
                if c == 0 {
                    self.items.remove(&id);
                    return Ok(());
                }
                self.items.insert(id, c);
                return Ok(());
            }
            return Err(InventoryError::RemovingTooMuchItems(id, c, count));
        }
        Err(InventoryError::ItemDoesNotExists(id))
    }

    pub fn transfer<T, V>(
        &mut self,
        receiver: &mut Self,
        id: MarketItemId,
        count: usize,
        database: &MarketDatabase<T, V>,
    ) -> Result<(), InventoryError>
    where
        T: std::fmt::Debug + Clone + Send + Sync,
        V: Currency + std::fmt::Debug + Clone + Send + Sync,
    {
        if !self.can_remove(id, count) {
            return Err(InventoryError::CannotGiveItem(id, count));
        }
        if !receiver.can_add(id, count, database) {
            return Err(InventoryError::CannotReceiveItem(id, count));
        }
        self.remove(id, count).unwrap();
        receiver.add(id, count, database).unwrap();
        Ok(())
    }
}

impl Prefab for Inventory {}

impl PrefabComponent for Inventory {}
