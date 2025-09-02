use crate::shared_kernel::value_objects::identifier::{EntityId, EntityMarker};

/// Marker type for Timer entities.
///
/// This marker ensures type safety when working with Timer identifiers,
/// preventing accidental mixing with other entity types.
pub struct Marker;

impl EntityMarker for Marker {
    const TYPE_NAME: &'static str = "Timer";
}

/// Strongly-typed identifier for Timer entities.
///
/// Uses the generic EntityId system to provide compile-time type safety
/// and prevent mixing Timer IDs with other entity types.
pub type Id = EntityId<Marker>;