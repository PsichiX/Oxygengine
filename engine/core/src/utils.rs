use crate::Ignite;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, io::Write};

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "StringSequenceDef")]
#[serde(into = "StringSequenceDef")]
pub struct StringSequence(String);

impl StringSequence {
    pub fn new(items: &[String]) -> Self {
        Self::from_iter(
            items
                .iter()
                .filter(|item| !item.is_empty())
                .map(|item| item.as_str()),
        )
    }

    pub fn append(&mut self, item: &str) {
        if !item.is_empty() {
            if self.0.is_empty() {
                self.0.push_str(item);
            } else {
                self.0.reserve(1 + item.len());
                self.0.push('|');
                self.0.push_str(item);
            }
        }
    }

    pub fn with(mut self, item: &str) -> Self {
        self.append(item);
        self
    }

    pub fn parts(&self) -> impl Iterator<Item = &str> {
        self.0.split('|').filter(|chunk| !chunk.is_empty())
    }

    pub fn rparts(&self) -> impl Iterator<Item = &str> {
        self.0.rsplit('|').filter(|chunk| !chunk.is_empty())
    }

    pub fn as_slice(&self) -> StrSequence<'_> {
        StrSequence(self.0.as_str())
    }
}

impl From<&[String]> for StringSequence {
    fn from(items: &[String]) -> Self {
        Self::new(items)
    }
}

impl<'a> FromIterator<&'a str> for StringSequence {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = &'a str>,
    {
        Self(
            iter.into_iter()
                .filter(|chunk| !chunk.is_empty())
                .flat_map(|item| std::iter::once('|').chain(item.chars()))
                .skip(1)
                .collect::<String>(),
        )
    }
}

impl From<StringSequenceDef> for StringSequence {
    fn from(def: StringSequenceDef) -> Self {
        def.0.iter().map(|item| item.as_str()).collect()
    }
}

#[derive(Serialize, Deserialize)]
#[serde(transparent)]
struct StringSequenceDef(Vec<String>);

impl From<StringSequence> for StringSequenceDef {
    fn from(seq: StringSequence) -> Self {
        Self(seq.parts().map(|item| item.to_owned()).collect())
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct StrSequence<'a>(&'a str);

impl<'a> StrSequence<'a> {
    pub fn parts(&self) -> impl Iterator<Item = &str> {
        self.0.split('|').filter(|chunk| !chunk.is_empty())
    }

    pub fn rparts(&self) -> impl Iterator<Item = &str> {
        self.0.rsplit('|').filter(|chunk| !chunk.is_empty())
    }

    pub fn to_owned(&self) -> StringSequence {
        StringSequence(self.0.to_owned())
    }
}

#[derive(Ignite, Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagFilters {
    #[serde(default)]
    inclusive: bool,
    #[serde(default)]
    tags: HashSet<String>,
}

impl TagFilters {
    pub fn inclusive() -> Self {
        Self {
            inclusive: true,
            tags: Default::default(),
        }
    }

    pub fn exclusive() -> Self {
        Self {
            inclusive: false,
            tags: Default::default(),
        }
    }

    pub fn none() -> Self {
        Self::inclusive()
    }

    pub fn all() -> Self {
        Self::exclusive()
    }

    pub fn include(mut self, tag: impl ToString) -> Self {
        if self.inclusive {
            self.tags.insert(tag.to_string());
        } else {
            self.tags.remove(&tag.to_string());
        }
        self
    }

    pub fn include_range(mut self, tags: impl Iterator<Item = impl ToString>) -> Self {
        for tag in tags {
            self = self.include(tag.to_string());
        }
        self
    }

    pub fn exclude(mut self, tag: impl ToString) -> Self {
        if self.inclusive {
            self.tags.remove(&tag.to_string());
        } else {
            self.tags.insert(tag.to_string());
        }
        self
    }

    pub fn exclude_range(mut self, tags: impl Iterator<Item = impl ToString>) -> Self {
        for tag in tags {
            self = self.exclude(tag.to_string());
        }
        self
    }

    pub fn combine(mut self, other: &Self) -> Self {
        if self.inclusive == other.inclusive {
            self.tags = self.tags.union(&other.tags).cloned().collect();
        } else {
            self.tags = self.tags.difference(&other.tags).cloned().collect();
        }
        self
    }

    pub fn validate_tag(&self, tag: &str) -> bool {
        if self.inclusive {
            self.tags.contains(tag)
        } else {
            !self.tags.contains(tag)
        }
    }
}

#[derive(Clone)]
pub struct StringBuffer {
    buffer: Vec<u8>,
    level: usize,
    pub indent: usize,
    pub resize: usize,
}

impl Default for StringBuffer {
    fn default() -> Self {
        Self {
            buffer: Default::default(),
            level: 0,
            indent: 2,
            resize: 1024,
        }
    }
}

impl StringBuffer {
    pub fn push_level(&mut self) {
        self.level += 1;
    }

    pub fn pop_level(&mut self) {
        if self.level > 0 {
            self.level -= 1;
        }
    }

    pub fn level(&self) -> usize {
        self.level
    }

    pub fn write_indent(&mut self) -> std::io::Result<()> {
        if self.level > 0 && self.indent > 0 {
            let count = self.level * self.indent;
            write!(&mut self.buffer, "{:indent$}", "", indent = count)
        } else {
            Ok(())
        }
    }

    pub fn write_indented_lines<S>(&mut self, s: S) -> std::io::Result<()>
    where
        S: AsRef<str>,
    {
        for line in s.as_ref().lines() {
            if !line.is_empty() {
                self.write_new_line()?;
                self.write_str(line.trim())?;
            }
        }
        Ok(())
    }

    pub fn write_str<S>(&mut self, s: S) -> std::io::Result<()>
    where
        S: AsRef<str>,
    {
        write!(&mut self.buffer, "{}", s.as_ref())
    }

    pub fn write_new_line(&mut self) -> std::io::Result<()> {
        writeln!(&mut self.buffer)?;
        self.write_indent()
    }

    pub fn write_space(&mut self) -> std::io::Result<()> {
        write!(&mut self.buffer, " ")
    }
}

impl Write for StringBuffer {
    fn flush(&mut self) -> std::io::Result<()> {
        self.buffer.flush()
    }

    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if self.resize > 0 && self.buffer.len() + buf.len() > self.buffer.capacity() {
            let count = buf.len() / self.resize + 1;
            self.buffer.reserve(self.resize * count);
        }
        self.buffer.write(buf)
    }
}

impl From<StringBuffer> for std::io::Result<String> {
    fn from(buffer: StringBuffer) -> Self {
        match String::from_utf8(buffer.buffer) {
            Ok(result) => Ok(result),
            Err(error) => Err(std::io::Error::new(std::io::ErrorKind::Other, error)),
        }
    }
}
