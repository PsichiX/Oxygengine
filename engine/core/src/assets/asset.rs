use crate::id::ID;
use std::{any::Any, mem::replace};

pub type AssetId = ID<Asset>;

pub struct Asset {
    id: AssetId,
    protocol: String,
    path: String,
    data: Box<dyn Any + Send + Sync>,
}

impl Asset {
    pub fn new<T>(protocol: &str, path: &str, data: T) -> Self
    where
        T: Any + Send + Sync + 'static,
    {
        Self::new_boxed(protocol, path, Box::new(data))
    }

    pub fn new_boxed(protocol: &str, path: &str, data: Box<dyn Any + Send + Sync>) -> Self {
        Self {
            id: AssetId::new(),
            protocol: protocol.to_owned(),
            path: path.to_owned(),
            data,
        }
    }

    pub fn id(&self) -> AssetId {
        self.id
    }

    pub fn protocol(&self) -> &str {
        &self.protocol
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn to_full_path(&self) -> String {
        format!("{}://{}", self.protocol, self.path)
    }

    pub fn is<T>(&self) -> bool
    where
        T: Any + Send + Sync,
    {
        self.data.is::<T>()
    }

    pub fn get<T>(&self) -> Option<&T>
    where
        T: Any + Send + Sync,
    {
        self.data.downcast_ref()
    }

    pub fn get_mut<T>(&mut self) -> Option<&mut T>
    where
        T: Any + Send + Sync,
    {
        self.data.downcast_mut()
    }

    pub fn set<T>(&mut self, data: T) -> Box<dyn Any + Send + Sync>
    where
        T: Any + Send + Sync,
    {
        replace(&mut self.data, Box::new(data))
    }
}
