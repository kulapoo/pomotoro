Domain-Centric Organization

priority - i.e `/timer - 2`
- low - 0
- medium - 1
- high - 2


```

└── pomotoro-domain / # DDD Domain Layer
    │   ├── Cargo.toml                   # itala-domain
    │   └── src/
    │       ├── lib.rs
    │       ├── shared_kernel/           # Shared concepts across domains
    │       │   ├── mod.rs
    │       │   ├── events/              # Domain events
    │       │   │   ├── mod.rs
    │       │   │   └── domain_event.rs
    │       │   ├── traits/              # Core domain traits
    │       │   │   ├── mod.rs
    │       │   │   ├── readable.rs
    │       │   │   ├── searchable.rs
    │       │   │   └── writable.rs
    │       │   └── value_objects/       # Shared value objects
    │       │       ├── mod.rs
    │       │       ├── identifier.rs
    │       │       ├── location.rs
    │       │       ├── tag.rs
    │       │       └── timestamp.rs
    │       │
    │       ├── task /                    # task Domain (Bounded Context)
    │       │   ├── task_cycling_srv.rs
    │       │   ├── mod.rs
    │       │   ├── task_repo.rs
    │       │   ├── task_session_srv.rs
    │       │   ├── task_config.rs
    │       │   ├── task_id.rs
    │       │   ├── task.rs
    │       │   └── task_status.rs
    │       ├── timer /                    # timer Domain (Bounded Context)
    │       │   ├── mod.rs
    │       │   ├── phase.rs
    │       │   ├── phase_transition_srv.rs
    │       │   ├── timer_state.rs
    │       │   ├── timer_state_with_task.rs
    │       │   └── timer_status.rs
    │       │
    │       │
    │       │
    │       ├── Config/                 # Config Domain (Bounded Context)
```

```
src/
    /components
        circular_progress.rs
    /services
        notification.rs
        store.rs
    /timer - 2
        timer_controls.rs
        timer_display.rs
        timer_state.rs
    /reports
    /tasks
    /settings
        settings.rs

    app.rs
    main.rs
```

```
src-tauri/ # Application layer and Infra
    /services
        notification.rs
    /timer
    /reports
    /settings
        settings.rs
    main.rs
    lib.rs

```