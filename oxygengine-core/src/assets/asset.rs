use crate::id::ID;
use std::{any::Any, mem::replace};

pub type AssetID = ID<()>;

pub struct Asset {
    id: AssetID,
    protocol: String,
    path: String,
    data: Box<dyn Any + Send + Sync>,
}

impl Asset {
    pub fn new(protocol: &str, path: &str, data: Box<dyn Any + Send + Sync>) -> Self {
        Self {
            id: AssetID::new(),
            protocol: protocol.to_owned(),
            path: path.to_owned(),
            data,
        }
    }

    pub fn id(&self) -> AssetID {
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
