use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Tag(String);

impl Tag {
    pub fn new(value: String) -> Result<Self> {
        let trimmed = value.trim().to_lowercase();

        if trimmed.is_empty() {
            return Err(Error::InvalidTagFormat { tag: value });
        }

        if trimmed.len() > 50 {
            return Err(Error::TagTooLong {
                tag: value,
                max_length: 50,
            });
        }

        if !trimmed
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(Error::InvalidTagFormat { tag: value });
        }

        Ok(Self(trimmed))
    }

    pub fn value(&self) -> &str {
        &self.0
    }

    pub fn work() -> Self {
        Self("work".to_string())
    }

    pub fn personal() -> Self {
        Self("personal".to_string())
    }

    pub fn learning() -> Self {
        Self("learning".to_string())
    }

    pub fn health() -> Self {
        Self("health".to_string())
    }
}

impl Display for Tag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for Tag {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        Self::new(value)
    }
}

impl TryFrom<&str> for Tag {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        Self::new(value.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagCollection(Vec<Tag>);

impl TagCollection {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn from_vec(tags: Vec<Tag>) -> Self {
        let mut unique_tags = Vec::new();
        for tag in tags {
            if !unique_tags.contains(&tag) {
                unique_tags.push(tag);
            }
        }
        Self(unique_tags)
    }

    pub fn from_strings(strings: Vec<String>) -> Result<Self> {
        let mut tags = Vec::new();
        for s in strings {
            tags.push(Tag::new(s)?);
        }
        Ok(Self::from_vec(tags))
    }

    pub fn add(&mut self, tag: Tag) {
        if !self.0.contains(&tag) {
            self.0.push(tag);
        }
    }

    pub fn remove(&mut self, tag: &Tag) -> bool {
        if let Some(pos) = self.0.iter().position(|t| t == tag) {
            self.0.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn contains(&self, tag: &Tag) -> bool {
        self.0.contains(tag)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Tag> {
        self.0.iter()
    }

    pub fn to_strings(&self) -> Vec<String> {
        self.0.iter().map(|tag| tag.value().to_string()).collect()
    }
}

impl Default for TagCollection {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoIterator for TagCollection {
    type Item = Tag;
    type IntoIter = std::vec::IntoIter<Tag>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_valid_tag() {
        let tag = Tag::new("work".to_string()).unwrap();
        assert_eq!(tag.value(), "work");
    }

    #[test]
    fn should_normalize_tag_case() {
        let tag = Tag::new("WORK".to_string()).unwrap();
        assert_eq!(tag.value(), "work");
    }

    #[test]
    fn should_trim_whitespace() {
        let tag = Tag::new("  work  ".to_string()).unwrap();
        assert_eq!(tag.value(), "work");
    }

    #[test]
    fn should_reject_empty_tag() {
        let result = Tag::new("".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn should_reject_too_long_tag() {
        let long_tag = "a".repeat(51);
        let result = Tag::new(long_tag);
        assert!(result.is_err());
    }

    #[test]
    fn should_reject_invalid_characters() {
        let result = Tag::new("work@home".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn should_allow_valid_characters() {
        let tag = Tag::new("work-project_1".to_string()).unwrap();
        assert_eq!(tag.value(), "work-project_1");
    }

    #[test]
    fn should_create_predefined_tags() {
        assert_eq!(Tag::work().value(), "work");
        assert_eq!(Tag::personal().value(), "personal");
        assert_eq!(Tag::learning().value(), "learning");
        assert_eq!(Tag::health().value(), "health");
    }

    #[test]
    fn should_create_tag_collection() {
        let mut collection = TagCollection::new();
        collection.add(Tag::work());
        collection.add(Tag::personal());

        assert_eq!(collection.len(), 2);
        assert!(collection.contains(&Tag::work()));
    }

    #[test]
    fn should_not_add_duplicate_tags() {
        let mut collection = TagCollection::new();
        collection.add(Tag::work());
        collection.add(Tag::work());

        assert_eq!(collection.len(), 1);
    }

    #[test]
    fn should_remove_tags() {
        let mut collection = TagCollection::new();
        collection.add(Tag::work());

        let removed = collection.remove(&Tag::work());
        assert!(removed);
        assert!(collection.is_empty());
    }
}
