# Technical Debt & Improvements

## Architecture Concerns

### ⚠️ Watch Items

- **Large command handler list (43 commands)** - consider grouping
  - Current: All commands in single `invoke_handler!` macro
  - Suggestion: Group by domain (timer, task, config, audio)
  - Impact: Better maintainability and clearer API surface

- **Event error handling prints to stderr** - could use structured logging
  - Current: `eprintln!("Failed to handle WorkSessionCompleted event: {}", e);`
  - Suggestion: Implement structured logging with proper error context
  - Impact: Better debugging and monitoring capabilities


## Potential Evolution Paths

### Near-term (MVP 2.0 alignment):

- TODO: Add task cycling automation
- TODO: Implement screen blocking service
- TODO: Enhance audio system with background soundscapes

## Long-term (Beyond MVP 2.0):

- TODO: Replace InMemoryTaskRepository with SqliteTaskRepository
- TODO: Add event sourcing for complete state reconstruction
- TODO: Implement distributed event bus for multi-device sync
