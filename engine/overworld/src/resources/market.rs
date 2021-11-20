use oxygengine_core::{id::ID, Scalar};
use std::collections::HashMap;

pub type MarketItemId = ID<MarketItem<(), ()>>;

pub trait Currency: Copy + Eq + Ord {
    type Error;

    /// (value left, value taken)!
    fn take(self, value: Self) -> Result<(Self, Self), Self::Error>;

    fn accumulate(self, value: Self) -> Result<Self, Self::Error>;

    /// (self value, other value)!
    fn exchange(self, other: Self, value: Self) -> Result<(Self, Self), Self::Error> {
        let (a, b) = self.take(value)?;
        let c = other.accumulate(b)?;
        Ok((a, c))
    }
}

#[derive(Debug, Clone)]
pub enum GenericCurrencyError<T>
where
    T: Currency,
{
    CouldNotTake(T),
    CouldNotAccumulate(T),
}

impl Currency for () {
    type Error = GenericCurrencyError<()>;

    fn take(self, _: Self) -> Result<(Self, Self), Self::Error> {
        Err(Self::Error::CouldNotTake(()))
    }

    fn accumulate(self, _: Self) -> Result<Self, Self::Error> {
        Err(Self::Error::CouldNotAccumulate(()))
    }
}

impl Currency for usize {
    type Error = GenericCurrencyError<usize>;

    fn take(self, value: Self) -> Result<(Self, Self), Self::Error> {
        match self.checked_sub(value) {
            Some(v) => Ok((v, value)),
            None => Err(Self::Error::CouldNotTake(value)),
        }
    }

    fn accumulate(self, value: Self) -> Result<Self, Self::Error> {
        match self.checked_add(value) {
            Some(v) => Ok(v),
            None => Err(Self::Error::CouldNotAccumulate(value)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MarketItem<T, V>
where
    T: std::fmt::Debug + Clone + Send + Sync,
    V: Currency + std::fmt::Debug + Clone + Send + Sync,
{
    pub data: T,
    pub value: V,
    pub weight: Scalar,
}

impl<T, V> Default for MarketItem<T, V>
where
    T: std::fmt::Debug + Default + Clone + Send + Sync,
    V: Currency + std::fmt::Debug + Default + Clone + Send + Sync,
{
    fn default() -> Self {
        Self {
            data: Default::default(),
            value: Default::default(),
            weight: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MarketDatabase<T, V>
where
    T: std::fmt::Debug + Clone + Send + Sync,
    V: Currency + std::fmt::Debug + Clone + Send + Sync,
{
    items: HashMap<MarketItemId, MarketItem<T, V>>,
}

impl<T, V> Default for MarketDatabase<T, V>
where
    T: std::fmt::Debug + Clone + Send + Sync,
    V: Currency + std::fmt::Debug + Clone + Send + Sync,
{
    fn default() -> Self {
        Self {
            items: Default::default(),
        }
    }
}

impl<T, V> MarketDatabase<T, V>
where
    T: std::fmt::Debug + Clone + Send + Sync,
    V: Currency + std::fmt::Debug + Clone + Send + Sync,
{
    pub fn register(&mut self, item: MarketItem<T, V>) -> MarketItemId {
        let id = MarketItemId::new();
        self.items.insert(id, item);
        id
    }

    pub fn register_many<'a>(
        &'a mut self,
        iter: impl Iterator<Item = MarketItem<T, V>> + 'a,
    ) -> impl Iterator<Item = MarketItemId> + 'a {
        let reserve = iter.size_hint();
        let reserve = reserve.1.unwrap_or(reserve.0);
        self.items.reserve(reserve);
        iter.map(|item| self.register(item))
    }

    pub fn unregister(&mut self, id: MarketItemId) -> Option<MarketItem<T, V>> {
        self.items.remove(&id)
    }

    pub fn unregister_many<'a>(
        &'a mut self,
        iter: impl Iterator<Item = MarketItemId> + 'a,
    ) -> impl Iterator<Item = MarketItem<T, V>> + 'a {
        iter.filter_map(|id| self.unregister(id))
    }

    pub fn items(&self) -> impl Iterator<Item = (MarketItemId, &MarketItem<T, V>)> {
        self.items.iter().map(|(id, item)| (*id, item))
    }

    pub fn item(&self, id: MarketItemId) -> Option<&MarketItem<T, V>> {
        self.items.get(&id)
    }

    pub fn item_mut(&mut self, id: MarketItemId) -> Option<&mut MarketItem<T, V>> {
        self.items.get_mut(&id)
    }

    pub fn find_items(
        &self,
        iter: impl Iterator<Item = MarketItemId>,
    ) -> impl Iterator<Item = (MarketItemId, &MarketItem<T, V>)> {
        iter.filter_map(|id| self.item(id).map(|item| (id, item)))
    }

    pub fn contains(&self, id: MarketItemId) -> bool {
        self.items.contains_key(&id)
    }

    pub fn item_id(&self, data: &T) -> Option<MarketItemId>
    where
        T: PartialEq,
    {
        self.items
            .iter()
            .find(|(_, item)| &item.data == data)
            .map(|(id, _)| *id)
    }
}
