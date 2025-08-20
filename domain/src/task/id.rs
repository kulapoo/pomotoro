use crate::shared_kernel::value_objects::identifier::{EntityId, EntityMarker};

/// Marker type for Task entities.
/// 
/// This marker ensures type safety when working with Task identifiers,
/// preventing accidental mixing with other entity types.
pub struct Marker;

impl EntityMarker for Marker {
    const TYPE_NAME: &'static str = "Task";
}

/// Strongly-typed identifier for Task entities.
/// 
/// Uses the generic EntityId system to provide compile-time type safety
/// and prevent mixing Task IDs with other entity types.
pub type Id = EntityId<Marker>;