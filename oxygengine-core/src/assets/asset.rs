use crate::id::ID;
use std::any::Any;

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

    pub fn set(&mut self, data: Box<dyn Any + Send + Sync>) {
        self.data = data;
    }
}
