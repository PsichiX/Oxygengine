//! Standarized Interprocess Messaging Protocol.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpMessageId {
    pub id: String,
    pub version: u32,
}

impl SimpMessageId {
    pub fn new(id: impl ToString, version: u32) -> Self {
        Self {
            id: id.to_string(),
            version,
        }
    }
}

impl std::fmt::Display for SimpMessageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} v{}", self.id, self.version)
    }
}

#[derive(Debug, Clone)]
pub struct SimpMessage {
    pub id: SimpMessageId,
    pub text_data: Option<String>,
    pub binary_data: Option<Vec<u8>>,
}

impl SimpMessage {
    pub fn new(id: SimpMessageId, text_data: String, binary_data: Vec<u8>) -> Self {
        Self {
            id,
            text_data: Some(text_data),
            binary_data: Some(binary_data),
        }
    }

    pub fn text(id: SimpMessageId, text_data: String) -> Self {
        Self {
            id,
            text_data: Some(text_data),
            binary_data: None,
        }
    }

    pub fn binary(id: SimpMessageId, binary_data: Vec<u8>) -> Self {
        Self {
            id,
            text_data: None,
            binary_data: Some(binary_data),
        }
    }

    pub fn empty(id: SimpMessageId) -> Self {
        Self {
            id,
            text_data: None,
            binary_data: None,
        }
    }
}

impl std::fmt::Display for SimpMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "* Message: {}", self.id)?;
        if let Some(text) = &self.text_data {
            write!(f, "- Text data:\n{}", text)?;
        }
        if let Some(binary) = &self.binary_data {
            write!(f, "- Binary data: {} bytes", binary.len())?;
        }
        Ok(())
    }
}

pub trait SimpSender
where
    Self: Sized,
{
    type Error;

    fn write(&mut self, message: SimpMessage) -> Result<(), Self::Error>;

    fn write_iter<I>(&mut self, iter: I) -> Result<(), Self::Error>
    where
        I: Iterator<Item = SimpMessage>,
    {
        for message in iter {
            self.write(message)?;
        }
        Ok(())
    }
}

impl SimpSender for () {
    type Error = ();

    fn write(&mut self, _: SimpMessage) -> Result<(), Self::Error> {
        Ok(())
    }
}

pub trait SimpReceiver
where
    Self: Sized,
{
    type Error;

    fn read(&mut self) -> Option<Result<SimpMessage, Self::Error>>;

    fn read_iter(&mut self) -> SimpReadIter<Self> {
        SimpReadIter(self)
    }
}

impl SimpReceiver for () {
    type Error = ();

    fn read(&mut self) -> Option<Result<SimpMessage, Self::Error>> {
        None
    }
}

pub struct SimpReadIter<'a, R>(&'a mut R)
where
    R: SimpReceiver;

impl<'a, R> Iterator for SimpReadIter<'a, R>
where
    R: SimpReceiver,
{
    type Item = Result<SimpMessage, R::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.read()
    }
}

pub trait SimpChannel: SimpSender + SimpReceiver {}
