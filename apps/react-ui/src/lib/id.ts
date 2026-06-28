/**
 * Short, git-SHA-style prefix of an entity id (first 7 hex chars).
 *
 * A UUID's first hyphen sits at index 8, so the leading 7 characters are
 * always pure hex — no separator stripping needed. Mirrors the Rust
 * `EntityId::short()` in core/domain/src/shared_kernel/value_objects/identifier.rs.
 */
export function shortId(id: string): string {
  return id.slice(0, 7)
}
