Domain-Centric Organization

priority - i.e `/timer - 2`
- low - 0
- medium - 1
- high - 2

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
src-tauri/
    /services
        notification.rs
    /timer
    /reports
    /settings
        settings.rs
    main.rs
    lib.rs

```