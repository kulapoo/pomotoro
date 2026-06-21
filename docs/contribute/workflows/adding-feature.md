# 🚀 Adding a New Feature

A step-by-step guide to implementing new features following Clean Architecture.

## Feature Implementation Flow

```mermaid
graph TD
    REQ[Requirements] --> DESIGN[Design Domain Model]
    DESIGN --> TESTS[Write Domain Tests]
    TESTS --> DOMAIN[Implement Domain Logic]
    DOMAIN --> UC_TEST[Write Use Case Tests]
    UC_TEST --> UC[Implement Use Cases]
    UC --> INFRA_TEST[Write Infrastructure Tests]
    INFRA_TEST --> INFRA[Implement Infrastructure]
    INFRA --> UI_TEST[Write UI Tests]
    UI_TEST --> UI[Implement UI]
    UI --> E2E[End-to-End Testing]
    E2E --> DOC[Documentation]
    
    style DOMAIN fill:#f9f,stroke:#333,stroke-width:4px
    style UC fill:#bbf,stroke:#333,stroke-width:2px
```

## Example: Adding a "Notes" Feature

Let's walk through adding a notes feature to tasks.

### Step 1: Design Domain Model

#### Define Requirements
- Users can add notes to tasks
- Notes have content and timestamp
- Notes can be edited or deleted
- Notes are displayed chronologically

#### Design Entities
```rust
// domain/src/task/note.rs
use crate::shared_kernel::value_objects::{Timestamp, NoteId};

#[derive(Clone, Debug, PartialEq)]
pub struct Note {
    id: NoteId,
    content: String,
    created_at: Timestamp,
    updated_at: Option<Timestamp>,
}

impl Note {
    pub fn new(content: String) -> Result<Self, DomainError> {
        if content.is_empty() {
            return Err(DomainError::EmptyNote);
        }
        
        Ok(Self {
            id: NoteId::new(),
            content,
            created_at: Timestamp::now(),
            updated_at: None,
        })
    }
    
    pub fn update_content(&mut self, content: String) -> Result<(), DomainError> {
        if content.is_empty() {
            return Err(DomainError::EmptyNote);
        }
        
        self.content = content;
        self.updated_at = Some(Timestamp::now());
        Ok(())
    }
}
```

#### Update Task Entity
```rust
// domain/src/task/task.rs
pub struct Task {
    // ... existing fields
    notes: Vec<Note>,
}

impl Task {
    pub fn add_note(&mut self, content: String) -> Result<NoteAdded, DomainError> {
        let note = Note::new(content)?;
        let note_id = note.id().clone();
        
        self.notes.push(note);
        
        Ok(NoteAdded {
            task_id: self.id.clone(),
            note_id,
            added_at: Timestamp::now(),
        })
    }
    
    pub fn update_note(&mut self, note_id: NoteId, content: String) -> Result<(), DomainError> {
        let note = self.notes.iter_mut()
            .find(|n| n.id() == &note_id)
            .ok_or(DomainError::NoteNotFound)?;
        
        note.update_content(content)?;
        Ok(())
    }
    
    pub fn remove_note(&mut self, note_id: NoteId) -> Result<(), DomainError> {
        let index = self.notes.iter()
            .position(|n| n.id() == &note_id)
            .ok_or(DomainError::NoteNotFound)?;
        
        self.notes.remove(index);
        Ok(())
    }
}
```

### Step 2: Write Domain Tests

```rust
// domain/src/task/tests/note_tests.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_can_add_note() {
        let mut task = Task::new("Test Task".to_string());
        
        let result = task.add_note("First note".to_string());
        
        assert!(result.is_ok());
        assert_eq!(task.notes().len(), 1);
        assert_eq!(task.notes()[0].content(), "First note");
    }
    
    #[test]
    fn empty_note_returns_error() {
        let mut task = Task::new("Test Task".to_string());
        
        let result = task.add_note("".to_string());
        
        assert!(matches!(result, Err(DomainError::EmptyNote)));
    }
    
    #[test]
    fn note_can_be_updated() {
        let mut task = Task::new("Test Task".to_string());
        let event = task.add_note("Original".to_string()).unwrap();
        
        let result = task.update_note(event.note_id, "Updated".to_string());
        
        assert!(result.is_ok());
        assert_eq!(task.notes()[0].content(), "Updated");
        assert!(task.notes()[0].updated_at().is_some());
    }
}
```

### Step 3: Create Use Cases

#### Add Note Use Case
```rust
// usecases/src/task/add_note.rs
pub struct AddNoteToTask {
    task_repository: Arc<dyn TaskRepository>,
    event_bus: Arc<dyn EventBus>,
}

impl AddNoteToTask {
    pub async fn execute(
        &self,
        task_id: TaskId,
        content: String,
    ) -> Result<NoteDto, UseCaseError> {
        // Load task
        let mut task = self.task_repository
            .find(task_id.clone())
            .await?
            .ok_or(UseCaseError::TaskNotFound)?;
        
        // Add note (domain logic)
        let event = task.add_note(content)?;
        
        // Save task
        self.task_repository.save(&task).await?;
        
        // Publish event
        self.event_bus.publish(event.clone()).await;
        
        // Return DTO
        let note = task.notes()
            .iter()
            .find(|n| n.id() == &event.note_id)
            .unwrap();
        
        Ok(NoteDto::from(note))
    }
}
```

#### DTOs
```rust
// usecases/src/task/dto/note_dto.rs
#[derive(Serialize, Deserialize)]
pub struct NoteDto {
    pub id: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: Option<String>,
}

impl From<&Note> for NoteDto {
    fn from(note: &Note) -> Self {
        Self {
            id: note.id().to_string(),
            content: note.content().to_string(),
            created_at: note.created_at().to_rfc3339(),
            updated_at: note.updated_at().map(|t| t.to_rfc3339()),
        }
    }
}
```

### Step 4: Infrastructure Implementation

#### Update Repository
```rust
// infra/src/adapters/task/file_repo.rs
impl TaskRepository for FileTaskRepository {
    async fn save(&self, task: &Task) -> Result<()> {
        let path = self.task_path(task.id());
        
        // Include notes in serialization
        let dto = TaskWithNotesDto::from(task);
        let json = serde_json::to_string_pretty(&dto)?;
        
        tokio::fs::write(path, json).await?;
        Ok(())
    }
}
```

#### Add Tauri Commands
```rust
// infra/src/commands/task_cmd.rs
#[tauri::command]
pub async fn add_note_to_task(
    state: State<'_, AppState>,
    task_id: String,
    content: String,
) -> Result<NoteDto, String> {
    let task_id = TaskId::from_str(&task_id)
        .map_err(|e| e.to_string())?;
    
    state.add_note_use_case()
        .execute(task_id, content)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_task_note(
    state: State<'_, AppState>,
    task_id: String,
    note_id: String,
    content: String,
) -> Result<(), String> {
    // Implementation
}

#[tauri::command]
pub async fn delete_task_note(
    state: State<'_, AppState>,
    task_id: String,
    note_id: String,
) -> Result<(), String> {
    // Implementation
}
```

### Step 5: UI Implementation

The React UI lives in `apps/react-ui/src/` and is organized **feature-sliced**:
each feature owns its `types.ts`, `model/` (Zustand store), `components/`, and
`pages/`. Cross-feature imports go through `@/lib/` (typed Tauri bridge, logger,
duration helpers) — never feature-to-feature deep imports except for `types.ts`.

#### 1. Register the command + event names

Add the new command/event to the single source of truth in
`apps/react-ui/src/lib/tauri.ts`:

```ts
export const commands = {
  // ...
  getTaskNotes: 'get_task_notes',
  addNoteToTask: 'add_note_to_task',
} as const

interface CommandMap {
  // ...
  get_task_notes: { args: { task_id: string }; ret: NoteDto[] }
  add_note_to_task: { args: { task_id: string; content: string }; ret: NoteDto }
}
```

These strings MUST match the `#[tauri::command]` names and `emit()` event names
in the Rust backend. Drift is a compile error.

#### 2. Create the feature store

```ts
// apps/react-ui/src/features/tasks/model/useNoteStore.ts
import { create } from 'zustand'
import { invokeCmd } from '@/lib/tauri'
import { BackendError } from '@/lib/errors'
import { logger } from '@/lib/logger'
import type { NoteDto } from '@/features/tasks/types'

interface NoteStore {
  notes: NoteDto[]
  error: BackendError | null
  loadNotes: (taskId: string) => Promise<boolean>
  addNote: (taskId: string, content: string) => Promise<boolean>
  clearError: () => void
}

export const useNoteStore = create<NoteStore>((set) => ({
  notes: [],
  error: null,
  loadNotes: async (taskId) => {
    try {
      const notes = await invokeCmd('get_task_notes', { task_id: taskId })
      set({ notes, error: null })
      return true
    } catch (e) {
      logger.error('loadNotes failed', e)
      set({ error: e as BackendError })
      return false
    }
  },
  addNote: async (taskId, content) => {
    try {
      const note = await invokeCmd('add_note_to_task', { task_id: taskId, content })
      set((s) => ({ notes: [...s.notes, note], error: null }))
      return true
    } catch (e) {
      logger.error('addNote failed', e)
      set({ error: e as BackendError })
      return false
    }
  },
  clearError: () => set({ error: null }),
}))
```

Actions return `Promise<boolean>` and store a `BackendError` on failure — they
never throw. The app-wide `<ErrorWatcher>` toasts errors automatically, so
components only toast on **success**.

#### 3. Create the component

```tsx
// apps/react-ui/src/features/tasks/components/NotesList.tsx
import { useEffect, useState } from 'react'
import { toast } from 'sonner'
import { useNoteStore } from '@/features/tasks/model/useNoteStore'
import type { NoteDto } from '@/features/tasks/types'

export function NotesList({ taskId }: { taskId: string }) {
  const notes = useNoteStore((s) => s.notes)
  const loadNotes = useNoteStore((s) => s.loadNotes)
  const addNote = useNoteStore((s) => s.addNote)
  const [draft, setDraft] = useState('')

  useEffect(() => {
    void loadNotes(taskId)
  }, [taskId, loadNotes])

  const handleSubmit = async () => {
    if (!draft.trim()) return
    const ok = await addNote(taskId, draft.trim())
    if (ok) {
      setDraft('')
      toast.success('Note added')
    }
  }

  return (
    <div>
      <h3>Notes</h3>
      <input
        value={draft}
        onChange={(e) => setDraft(e.target.value)}
        onKeyDown={(e) => e.key === 'Enter' && handleSubmit()}
      />
      <ul>
        {notes.map((n) => (
          <li key={n.id}>
            <p>{n.content}</p>
            <span>{n.created_at}</span>
          </li>
        ))}
      </ul>
    </div>
  )
}
```

#### Key conventions

- **Never** call `invoke()` directly — use `invokeCmd()` from `@/lib/tauri`.
- **Never** use bare `console.*` — use `logger` from `@/lib/logger` (forwards to
  the Rust log file).
- **Never** `toast.error()` in a component for store failures — `<ErrorWatcher>`
  handles it. Only `toast.success()` / `toast.info()` for user-facing success.
- Shared UI primitives (Row, Section, Toggle, NumberInput, SelectInput) live in
  `@/components/ui/`.

### Step 6: End-to-End Testing

```rust
// tests/e2e/notes_feature.rs
#[tokio::test]
async fn complete_notes_workflow() {
    let context = TestContext::new().await;
    
    // Create task
    let task_id = context.create_task("Test Task").await;
    
    // Add note
    let note = context.add_note_to_task(
        task_id.clone(),
        "First note"
    ).await;
    
    assert_eq!(note.content, "First note");
    
    // Update note
    context.update_note(
        task_id.clone(),
        note.id.clone(),
        "Updated note"
    ).await;
    
    // Verify update
    let notes = context.get_task_notes(task_id.clone()).await;
    assert_eq!(notes[0].content, "Updated note");
    
    // Delete note
    context.delete_note(task_id.clone(), note.id).await;
    
    // Verify deletion
    let notes = context.get_task_notes(task_id).await;
    assert!(notes.is_empty());
}
```

### Step 7: Documentation

#### Update API Documentation
```markdown
## Task Notes API

### Add Note to Task
POST /api/tasks/{task_id}/notes
```json
{
  "content": "Note content"
}
```

### Update Note
PUT /api/tasks/{task_id}/notes/{note_id}
```json
{
  "content": "Updated content"
}
```

### Delete Note
DELETE /api/tasks/{task_id}/notes/{note_id}
```

## Feature Checklist

### Planning ✅
- [ ] Requirements documented
- [ ] Domain model designed
- [ ] Use cases identified
- [ ] UI mockups created

### Implementation ✅
- [ ] Domain entities created
- [ ] Domain tests written
- [ ] Use cases implemented
- [ ] Use case tests written
- [ ] Repository updated
- [ ] Commands added
- [ ] UI components created
- [ ] UI tests written

### Testing ✅
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] E2E tests pass
- [ ] Manual testing complete

### Documentation ✅
- [ ] API documented
- [ ] User guide updated
- [ ] Code comments added
- [ ] CHANGELOG updated

### Review ✅
- [ ] Code reviewed
- [ ] Tests reviewed
- [ ] Documentation reviewed
- [ ] PR approved

## Common Pitfalls

### 1. Starting with UI
❌ Don't start with UI implementation
✅ Start with domain model

### 2. Skipping Tests
❌ Don't implement without tests
✅ Write tests first (TDD)

### 3. Violating Boundaries
❌ Don't mix layer responsibilities
✅ Keep each layer focused

### 4. Big Bang Implementation
❌ Don't implement everything at once
✅ Incremental implementation with tests

## Tips for Success

1. **Start Small**: Implement minimal viable feature first
2. **Test Early**: Write tests before implementation
3. **Refactor Often**: Clean up as you go
4. **Document Always**: Update docs with code
5. **Get Feedback**: Review early and often

## Next Steps
- See [Testing Workflow](./testing.md)
- Learn [Fixing Bugs](./fixing-bugs.md)
- Review [Code Review Process](./code-review.md)