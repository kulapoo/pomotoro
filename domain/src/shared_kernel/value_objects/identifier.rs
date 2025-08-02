//! Generic entity identifier value object.
//!
//! This module provides a generic, type-safe identifier system that can be used
//! across all domain bounded contexts to uniquely identify entities while
//! maintaining type safety and preventing accidental mixing of different
//! entity types.
//!
//! # Design Principles
//!
//! - **Type Safety**: Different entity types cannot be confused
//! - **Zero-Cost Abstractions**: Compiles to pure UUID operations
//! - **Domain Agnostic**: Can be used by any bounded context
//! - **Globally Unique**: Uses UUID v4 for collision-free identifiers
//! - **Serializable**: Can be persisted and transmitted over APIs
//!
//! # Examples
//!
//! ```rust
//! use domain::shared_kernel::{EntityId, EntityMarker};
//!
//! // Define entity markers for type safety
//! pub struct NoteMarker;
//! impl EntityMarker for NoteMarker {
//!     const TYPE_NAME: &'static str = "Note";
//! }
//!
//! pub struct LinkMarker;
//! impl EntityMarker for LinkMarker {
//!     const TYPE_NAME: &'static str = "Link";
//! }
//!
//! // Create type aliases for each entity
//! pub type NoteId = EntityId<NoteMarker>;
//! pub type LinkId = EntityId<LinkMarker>;
//!
//! // Use the identifiers
//! let note_id = NoteId::new();
//! let link_id = LinkId::new();
//!
//! // Type safety prevents mixing different entity types
//! // let mixed: NoteId = link_id; // This won't compile!
//! ```

use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use uuid::Uuid;

/// Marker trait for entity types.
///
/// This trait is used to create type-safe identifiers that cannot be
/// accidentally mixed between different entity types. Each entity
/// should define its own marker type that implements this trait.
///
/// # Examples
///
/// ```rust
/// use domain::shared_kernel::EntityMarker;
///
/// pub struct NoteMarker;
/// impl EntityMarker for NoteMarker {
///     const TYPE_NAME: &'static str = "Note";
/// }
///
/// pub struct LinkMarker;
/// impl EntityMarker for LinkMarker {
///     const TYPE_NAME: &'static str = "Link";
/// }
/// ```
pub trait EntityMarker: 'static {
    /// Human-readable name for this entity type.
    /// Used for debugging and error messages.
    const TYPE_NAME: &'static str;
}

/// Generic entity identifier that provides type safety across domains.
///
/// This generic identifier can be used by any domain to create strongly-typed
/// entity identifiers that cannot be accidentally mixed up. It wraps a UUID
/// but carries the entity type information at compile time.
///
/// # Type Parameters
///
/// * `T` - The entity marker type that identifies what kind of entity this ID represents
///
/// # Design Benefits
///
/// - **Compile-Time Safety**: Cannot mix different entity types
/// - **Zero Runtime Cost**: Compiles down to pure UUID operations
/// - **Domain Independent**: Shared across all bounded contexts
/// - **Standard Format**: Uses well-established UUID format
///
/// # Examples
///
/// ```rust
/// use domain::shared_kernel::{EntityId, EntityMarker};
///
/// // Define entity markers
/// pub struct NoteMarker;
/// impl EntityMarker for NoteMarker {
///     const TYPE_NAME: &'static str = "Note";
/// }
///
/// // Create type alias
/// pub type NoteId = EntityId<NoteMarker>;
///
/// // Generate new identifier
/// let note_id = NoteId::new();
/// println!("Generated note ID: {}", note_id);
///
/// // Parse from string
/// let parsed_id = NoteId::from_string("550e8400-e29b-41d4-a716-446655440000")?;
///
/// // Convert to string
/// let id_string = note_id.as_str();
/// # Ok::<(), uuid::Error>(())
/// ```
#[derive(Serialize, Deserialize)]
pub struct EntityId<T: EntityMarker> {
    uuid: Uuid,
    #[serde(skip)]
    _marker: PhantomData<T>,
}

impl<T: EntityMarker> EntityId<T> {
    /// Generate a new unique entity identifier.
    ///
    /// Uses UUID v4 (random) to ensure global uniqueness without coordination.
    ///
    /// # Returns
    ///
    /// A new unique identifier for the entity type
    ///
    /// # Examples
    ///
    /// ```rust
    /// use domain::shared_kernel::{EntityId, EntityMarker};
    ///
    /// pub struct NoteMarker;
    /// impl EntityMarker for NoteMarker {
    ///     const TYPE_NAME: &'static str = "Note";
    /// }
    ///
    /// pub type NoteId = EntityId<NoteMarker>;
    ///
    /// let note_id = NoteId::new();
    /// println!("Generated note ID: {}", note_id);
    /// ```
    pub fn new() -> Self {
        Self {
            uuid: Uuid::new_v4(),
            _marker: PhantomData,
        }
    }

    /// Create an entity ID from an existing UUID.
    ///
    /// This is useful when you have a UUID from another source and need to
    /// convert it to a strongly-typed entity identifier.
    ///
    /// # Arguments
    ///
    /// * `uuid` - The UUID to wrap as an entity identifier
    ///
    /// # Returns
    ///
    /// An entity identifier wrapping the provided UUID
    ///
    /// # Examples
    ///
    /// ```rust
    /// use domain::shared_kernel::{EntityId, EntityMarker};
    /// use uuid::Uuid;
    ///
    /// pub struct NoteMarker;
    /// impl EntityMarker for NoteMarker {
    ///     const TYPE_NAME: &'static str = "Note";
    /// }
    ///
    /// pub type NoteId = EntityId<NoteMarker>;
    ///
    /// let uuid = Uuid::new_v4();
    /// let note_id = NoteId::from_uuid(uuid);
    /// ```
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self {
            uuid,
            _marker: PhantomData,
        }
    }

    /// Parse an entity ID from a string representation.
    ///
    /// This is commonly used when receiving entity IDs from APIs, databases,
    /// or user input that needs to be converted back to a typed identifier.
    ///
    /// # Arguments
    ///
    /// * `s` - String representation of a UUID
    ///
    /// # Returns
    ///
    /// * `Ok(EntityId<T>)` - Successfully parsed entity ID
    /// * `Err(uuid::Error)` - Invalid UUID format
    ///
    /// # Examples
    ///
    /// ```rust
    /// use domain::shared_kernel::{EntityId, EntityMarker};
    ///
    /// pub struct NoteMarker;
    /// impl EntityMarker for NoteMarker {
    ///     const TYPE_NAME: &'static str = "Note";
    /// }
    ///
    /// pub type NoteId = EntityId<NoteMarker>;
    ///
    /// // Valid UUID string
    /// let note_id = NoteId::from_string("550e8400-e29b-41d4-a716-446655440000")?;
    ///
    /// // Invalid format returns error
    /// let result = NoteId::from_string("invalid-uuid");
    /// assert!(result.is_err());
    /// # Ok::<(), uuid::Error>(())
    /// ```
    pub fn from_string(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self {
            uuid: Uuid::parse_str(s)?,
            _marker: PhantomData,
        })
    }

    /// Convert the entity ID to its string representation.
    ///
    /// Returns the standard UUID string format, suitable for display,
    /// storage, or transmission over APIs.
    ///
    /// # Returns
    ///
    /// String representation of the UUID in standard format
    ///
    /// # Examples
    ///
    /// ```rust
    /// use domain::shared_kernel::{EntityId, EntityMarker};
    ///
    /// pub struct NoteMarker;
    /// impl EntityMarker for NoteMarker {
    ///     const TYPE_NAME: &'static str = "Note";
    /// }
    ///
    /// pub type NoteId = EntityId<NoteMarker>;
    ///
    /// let note_id = NoteId::new();
    /// let id_string = note_id.as_str();
    /// println!("Note ID: {}", id_string);
    /// ```
    pub fn as_str(&self) -> String {
        self.uuid.to_string()
    }

    /// Get a reference to the underlying UUID.
    ///
    /// This provides access to the raw UUID for cases where you need to
    /// interact with APIs that expect a UUID directly.
    ///
    /// # Returns
    ///
    /// Reference to the underlying UUID
    ///
    /// # Examples
    ///
    /// ```rust
    /// use domain::shared_kernel::{EntityId, EntityMarker};
    ///
    /// pub struct NoteMarker;
    /// impl EntityMarker for NoteMarker {
    ///     const TYPE_NAME: &'static str = "Note";
    /// }
    ///
    /// pub type NoteId = EntityId<NoteMarker>;
    ///
    /// let note_id = NoteId::new();
    /// let uuid_ref = note_id.inner();
    /// ```
    pub fn inner(&self) -> &Uuid {
        &self.uuid
    }

    /// Get the entity type name for debugging purposes.
    ///
    /// Returns the type name defined in the EntityMarker implementation.
    ///
    /// # Returns
    ///
    /// Static string containing the entity type name
    ///
    /// # Examples
    ///
    /// ```rust
    /// use domain::shared_kernel::{EntityId, EntityMarker};
    ///
    /// pub struct NoteMarker;
    /// impl EntityMarker for NoteMarker {
    ///     const TYPE_NAME: &'static str = "Note";
    /// }
    ///
    /// pub type NoteId = EntityId<NoteMarker>;
    ///
    /// let note_id = NoteId::new();
    /// assert_eq!(note_id.type_name(), "Note");
    /// ```
    pub fn type_name(&self) -> &'static str {
        T::TYPE_NAME
    }
}

// Manual implementations to avoid requiring T to implement these traits

impl<T: EntityMarker> Clone for EntityId<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: EntityMarker> Copy for EntityId<T> {}

impl<T: EntityMarker> Debug for EntityId<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(&format!("EntityId<{}>", T::TYPE_NAME))
            .field("uuid", &self.uuid)
            .finish()
    }
}

impl<T: EntityMarker> PartialEq for EntityId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

impl<T: EntityMarker> Eq for EntityId<T> {}

impl<T: EntityMarker> Hash for EntityId<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.uuid.hash(state);
    }
}

impl<T: EntityMarker> Display for EntityId<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.uuid)
    }
}

impl<T: EntityMarker> Default for EntityId<T> {
    fn default() -> Self {
        Self::new()
    }
}

// Conversion traits

impl<T: EntityMarker> TryFrom<String> for EntityId<T> {
    type Error = uuid::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_string(&value)
    }
}

impl<T: EntityMarker> TryFrom<&str> for EntityId<T> {
    type Error = uuid::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_string(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test entity markers
    struct TestTaskMarker;
    impl EntityMarker for TestTaskMarker {
        const TYPE_NAME: &'static str = "TestNote";
    }

    struct TestLinkMarker;
    impl EntityMarker for TestLinkMarker {
        const TYPE_NAME: &'static str = "TestLink";
    }

    type TestTaskId = EntityId<TestTaskMarker>;
    type TestLinkId = EntityId<TestLinkMarker>;

    #[test]
    fn test_entity_id_creation() {
        let note_id = TestTaskId::new();
        let link_id = TestLinkId::new();

        // IDs should be different
        assert_ne!(note_id.as_str(), link_id.as_str());
    }

    #[test]
    fn test_entity_id_from_uuid() {
        let uuid = Uuid::new_v4();
        let note_id = TestTaskId::from_uuid(uuid);
        assert_eq!(note_id.inner(), &uuid);
    }

    #[test]
    fn test_entity_id_from_string() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let note_id = TestTaskId::from_string(uuid_str).unwrap();
        assert_eq!(note_id.as_str(), uuid_str);
    }

    #[test]
    fn test_invalid_uuid_string() {
        let result = TestTaskId::from_string("invalid-uuid");
        assert!(result.is_err());
    }

    #[test]
    fn test_type_safety() {
        let note_id = TestTaskId::new();
        let link_id = TestLinkId::new();

        // These are different types, even though they both wrap UUIDs
        assert_eq!(note_id.type_name(), "TestNote");
        assert_eq!(link_id.type_name(), "TestLink");
    }

    #[test]
    fn test_equality() {
        let uuid = Uuid::new_v4();
        let note_id1 = TestTaskId::from_uuid(uuid);
        let note_id2 = TestTaskId::from_uuid(uuid);

        assert_eq!(note_id1, note_id2);
    }

    #[test]
    fn test_clone_and_copy() {
        let note_id = TestTaskId::new();
        let cloned = note_id.clone();
        let copied = note_id;

        assert_eq!(note_id, cloned);
        assert_eq!(note_id, copied);
    }

    #[test]
    fn test_debug_format() {
        let note_id = TestTaskId::new();
        let debug_str = format!("{:?}", note_id);
        assert!(debug_str.contains("EntityId<TestNote>"));
    }

    #[test]
    fn test_display_format() {
        let note_id = TestTaskId::new();
        let display_str = format!("{}", note_id);
        // Should display the UUID string
        assert_eq!(display_str, note_id.as_str());
    }

    // Serialization tests removed - these are infrastructure concerns

    #[test]
    fn test_try_from_string() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";

        let note_id1: TestTaskId = uuid_str.try_into().unwrap();
        let note_id2: TestTaskId = uuid_str.to_string().try_into().unwrap();

        assert_eq!(note_id1, note_id2);
        assert_eq!(note_id1.as_str(), uuid_str);
    }

    #[test]
    fn test_hash_consistency() {
        use std::collections::HashMap;

        let note_id = TestTaskId::new();
        let mut map = HashMap::new();
        map.insert(note_id, "test_value");

        // Should be able to retrieve using the same key
        assert_eq!(map.get(&note_id), Some(&"test_value"));
    }
}