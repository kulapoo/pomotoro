# Leptos Developer Guidelines

## Table of Contents
1. [Introduction](#introduction)
2. [Core Concepts](#core-concepts)
3. [Project Setup](#project-setup)
4. [Reactive System](#reactive-system)
5. [Components & Props](#components--props)
6. [Server Functions](#server-functions)
7. [Async & Resources](#async--resources)
8. [Error Handling](#error-handling)
9. [Styling](#styling)
10. [Testing](#testing)
11. [Performance Optimization](#performance-optimization)
12. [Deployment](#deployment)
13. [Best Practices](#best-practices)

## Introduction

Leptos is a full-stack Rust web framework focused on fine-grained reactivity, isomorphic rendering, and type safety. It enables building web applications entirely in Rust with excellent performance and developer experience.

### Key Features
- **Fine-grained reactivity**: Only updates what changes in the DOM
- **Full-stack capabilities**: Write frontend and backend in Rust
- **Server functions**: Seamless client-server communication
- **Multiple rendering modes**: CSR, SSR, SSG, and hydration
- **Type safety**: Leverage Rust's type system throughout

## Core Concepts

### Architecture Philosophy
- **Components run once**: Setup functions, not render functions
- **No virtual DOM**: Direct DOM manipulation with reactive signals
- **Signals-based reactivity**: Automatic dependency tracking
- **Zero-cost abstractions**: Compile-time optimizations

### Rendering Modes

#### Client-Side Rendering (CSR)
```rust
fn main() {
    leptos::mount::mount_to_body(App)
}
```

#### Server-Side Rendering (SSR)
```rust
#[cfg(feature = "ssr")]
async fn main() {
    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;

    HttpServer::new(move || {
        let leptos_options = leptos_options.clone();
        App::new()
            .leptos_routes(leptos_options, routes, App)
    })
    .bind(&addr)?
    .run()
    .await
}
```

## Project Setup

### Installation

#### Prerequisites
```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
cargo install cargo-leptos
```

#### Create New Project

##### CSR with Trunk
```toml
[dependencies]
leptos = { version = "0.6", features = ["csr", "nightly"] }
console_error_panic_hook = "0.1"
```

##### Full-Stack with cargo-leptos
```toml
[dependencies]
leptos = { version = "0.6", features = ["nightly"] }
leptos_axum = { version = "0.6", optional = true }
leptos_router = { version = "0.6" }
leptos_meta = { version = "0.6" }

[features]
hydrate = ["leptos/hydrate"]
ssr = ["leptos/ssr", "dep:leptos_axum"]
```

### Project Structure
```
project/
├── src/
│   ├── main.rs          # Entry point
│   ├── app.rs           # Root component
│   ├── components/      # Reusable components
│   ├── pages/           # Route components
│   └── server/          # Server functions
├── style/
│   └── main.css
├── Cargo.toml
├── index.html           # CSR template
└── Leptos.toml          # cargo-leptos config
```

## Reactive System

### Signals

#### Creating Signals
```rust
let (read_signal, write_signal) = signal(0);
let rw_signal = RwSignal::new(0);
```

#### Reading Signals
```rust
let value = signal.get();
signal.with(|value| {

});
let guard = signal.read();
```

#### Writing Signals
```rust
set_signal.set(5);
set_signal.update(|n| *n += 1);
signal.write().push("item");
```

### Derived Signals
```rust
let double = move || count.get() * 2;
let formatted = move || format!("Count: {}", count.get());
```

### Memos
```rust
let expensive_computation = Memo::new(move |_| {
    count.get() * complex_calculation()
});
```

### Effects
```rust
Effect::new(move |_| {
    logging::log!("Count changed to: {}", count.get());
});

Effect::watch(
    move || source.get(),
    move |new_value, prev_value, _| {
        handle_change(new_value, prev_value);
    },
    false,
);
```

### Reactive Best Practices

#### Signal Composition Patterns
```rust
let (first_name, set_first_name) = signal("John");
let (last_name, set_last_name) = signal("Doe");
let full_name = move || format!("{} {}", first_name.get(), last_name.get());

let combined = move || {
    batch(|| {
        set_first_name.set("Jane");
        set_last_name.set("Smith");
    })
};
```

#### Avoiding Anti-patterns
```rust
let (a, set_a) = signal(0);
let (b, set_b) = signal(0);

Effect::new(move |_| {
    set_b.set(a.get() * 2);
});

let b = move || a.get() * 2;
```

## Components & Props

### Component Definition
```rust
#[component]
fn Button(
    #[prop(optional)] class: Option<&'static str>,
    #[prop(default = "Click me")] text: &'static str,
    #[prop(into)] count: Signal<i32>,
    children: Children,
) -> impl IntoView {
    view! {
        <button class=class>
            {text}: {count}
            {children()}
        </button>
    }
}
```

### Component Patterns

#### Generic Components
```rust
#[component]
fn List<T, F>(items: Vec<T>, render_item: F) -> impl IntoView
where
    T: Clone + 'static,
    F: Fn(&T) -> View + 'static,
{
    view! {
        <ul>
            <For
                each=move || items.clone()
                key=|item| item.clone()
                let:item
            >
                {render_item(&item)}
            </For>
        </ul>
    }
}
```

#### Context API
```rust
#[derive(Clone)]
struct AppState {
    user: RwSignal<Option<User>>,
    theme: RwSignal<Theme>,
}

#[component]
fn App() -> impl IntoView {
    provide_context(AppState::new());

    view! {
        <Router>
            <Routes>
            </Routes>
        </Router>
    }
}

#[component]
fn Child() -> impl IntoView {
    let state = use_context::<AppState>()
        .expect("AppState not provided");

    view! {
        <div>{move || state.user.get()}</div>
    }
}
```

## Server Functions

### Basic Server Function
```rust
#[server(SaveUser, "/api")]
pub async fn save_user(
    name: String,
    email: String,
) -> Result<User, ServerFnError> {
    let pool = extract::<State<PgPool>>().await?;

    let user = sqlx::query_as!(
        User,
        "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING *",
        name,
        email
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    Ok(user)
}
```

### Using Server Functions
```rust
#[component]
fn UserForm() -> impl IntoView {
    let save_user_action = Action::<SaveUser, _>::server();

    view! {
        <ActionForm action=save_user_action>
            <input type="text" name="name" />
            <input type="email" name="email" />
            <button type="submit">"Save"</button>
        </ActionForm>

        <Show
            when=move || save_user_action.pending().get()
            fallback=|| ()
        >
            <p>"Saving..."</p>
        </Show>
    }
}
```

### Custom Error Types
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppError {
    NotFound,
    Unauthorized,
    Database(String),
}

impl FromServerFnError for AppError {
    type Encoding = JsonEncoding;

    fn from_server_fn_error(err: ServerFnError) -> Self {
        AppError::Database(err.to_string())
    }
}

#[server]
pub async fn protected_action() -> Result<String, AppError> {
    let user = get_current_user().await
        .ok_or(AppError::Unauthorized)?;

    Ok(format!("Welcome, {}", user.name))
}
```

## Async & Resources

### Resources for Data Loading
```rust
#[component]
fn UserProfile(id: Signal<u32>) -> impl IntoView {
    let user = Resource::new(
        move || id.get(),
        |id| async move {
            fetch_user(id).await
        }
    );

    view! {
        <Suspense fallback=|| view! { <p>"Loading..."</p> }>
            {move || user.get().map(|u| match u {
                Ok(user) => view! {
                    <div>
                        <h1>{&user.name}</h1>
                        <p>{&user.email}</p>
                    </div>
                },
                Err(e) => view! {
                    <p>"Error: "{e.to_string()}</p>
                }
            })}
        </Suspense>
    }
}
```

### Actions for Mutations
```rust
#[component]
fn TodoList() -> impl IntoView {
    let add_todo = Action::new(|input: &String| {
        let input = input.clone();
        async move {
            add_todo_to_db(&input).await
        }
    });

    let (input, set_input) = signal(String::new());

    view! {
        <input
            type="text"
            on:input=move |ev| set_input.set(event_target_value(&ev))
            prop:value=input
        />
        <button
            on:click=move |_| add_todo.dispatch(input.get())
            disabled=move || add_todo.pending().get()
        >
            "Add Todo"
        </button>
    }
}
```

### SSR Modes

#### Async Mode
```rust
#[component(async)]
fn Page() -> impl IntoView {
    let data = fetch_critical_data().await?;

    view! {
        <div>{data}</div>
    }
}
```

#### Streaming Mode
```rust
#[component]
fn StreamingPage() -> impl IntoView {
    view! {
        <div>"Shell loads immediately"</div>
        <Suspense fallback=|| view! { <p>"Loading..."</p> }>
            <AsyncContent />
        </Suspense>
    }
}
```

## Error Handling

### Component Error Boundaries
```rust
#[component]
fn SafeComponent() -> impl IntoView {
    let (value, set_value) = signal(Ok(0));

    view! {
        <ErrorBoundary
            fallback=|errors| view! {
                <div class="error">
                    <h3>"Errors:"</h3>
                    <ul>
                        {move || errors.get()
                            .into_iter()
                            .map(|(_, e)| view! {
                                <li>{e.to_string()}</li>
                            })
                            .collect_view()
                        }
                    </ul>
                </div>
            }
        >
            <RiskyComponent value />
        </ErrorBoundary>
    }
}
```

### Custom Error Types with thiserror
```rust
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum ValidationError {
    #[error("Value must be between {min} and {max}")]
    OutOfRange { min: i32, max: i32 },

    #[error("Invalid format: {0}")]
    InvalidFormat(String),
}

#[component]
fn ValidatedInput() -> impl IntoView {
    let (value, set_value) = signal(Ok(0));

    let validate = move |input: String| {
        match input.parse::<i32>() {
            Ok(n) if n < 0 || n > 100 => {
                Err(ValidationError::OutOfRange { min: 0, max: 100 })
            }
            Ok(n) => Ok(n),
            Err(_) => Err(ValidationError::InvalidFormat(input)),
        }
    };

    view! {
        <input
            type="text"
            on:input=move |ev| {
                set_value.set(validate(event_target_value(&ev)))
            }
        />
    }
}
```

## Styling

### CSS Integration

#### Trunk Assets
```html
<!-- index.html -->
<link data-trunk rel="css" href="style/main.css"/>
```

#### Tailwind CSS
```toml
# Cargo.toml
[package.metadata.leptos]
tailwind-input-file = "style/tailwind.css"
tailwind-config-file = "tailwind.config.js"
```

#### Component Styling
```rust
#[component]
fn StyledButton() -> impl IntoView {
    view! {
        <button class="px-4 py-2 bg-blue-500 hover:bg-blue-600 text-white rounded">
            "Click me"
        </button>
    }
}
```

### Scoped CSS Libraries

#### Stylers (Compile-time)
```rust
use stylers::style;

#[component]
pub fn Card() -> impl IntoView {
    let class = style! {
        .card {
            padding: 1rem;
            border-radius: 0.5rem;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);

            &:hover {
                box-shadow: 0 4px 8px rgba(0,0,0,0.15);
            }
        }
    };

    view! {
        <div class=class>
            <slot/>
        </div>
    }
}
```

## Testing

### Component Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use leptos::*;

    #[test]
    fn test_counter_component() {
        create_runtime();

        let (count, set_count) = create_signal(0);

        let node = view! {
            <Counter count set_count />
        };

        set_count.set(5);
        assert_eq!(count.get(), 5);

        dispose_runtime();
    }
}
```

### Server Function Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_save_user() {
        let pool = setup_test_db().await;
        provide_context(pool);

        let result = save_user("Test".to_string(), "test@example.com".to_string()).await;

        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.name, "Test");
    }
}
```

## Performance Optimization

### Signal Optimization
```rust
let (names, set_names) = signal(Vec::new());

if names.read().is_empty() {
    set_names.write().push("Alice".to_string());
}

let expensive = Memo::new(move |_| {
    compute_expensive(data.get())
});

let derived = move || simple_transform(data.get());
```

### Component Optimization
```rust
#[component]
fn OptimizedList() -> impl IntoView {
    let items = RwSignal::new(vec![1, 2, 3]);

    view! {
        <For
            each=move || items.get()
            key=|item| *item
            children=move |item| view! {
                <ListItem value=item />
            }
        />
    }
}

#[component]
fn ListItem(value: i32) -> impl IntoView {
    view! {
        <li>{value}</li>
    }
}
```

### Bundle Size Optimization
```toml
[profile.release]
opt-level = 'z'
lto = true
codegen-units = 1
strip = true

[profile.wasm-release]
inherits = "release"
opt-level = 'z'
```

## Deployment

### Build Configuration

#### Release Mode
```bash
cargo leptos build --release

cargo build --release --target wasm32-unknown-unknown
trunk build --release
```

#### Environment Variables
```toml
# Leptos.toml
[env]
LEPTOS_OUTPUT_NAME = "app"
LEPTOS_SITE_ROOT = "site"
LEPTOS_SITE_PKG_DIR = "pkg"
LEPTOS_ASSETS_DIR = "assets"
```

### Deployment Platforms

#### Static Hosting (CSR)
```nginx
server {
    listen 80;
    root /var/www/html;

    location / {
        try_files $uri $uri/ /index.html;
    }
}
```

#### Server Deployment (SSR)
```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/app /usr/local/bin/app
EXPOSE 3000
CMD ["app"]
```

#### Edge Deployment
```toml
# Deploy to Cloudflare Workers, Deno Deploy, etc.
[dependencies]
leptos = { version = "0.6", features = ["ssr", "nonce"] }
leptos_axum = { version = "0.6", features = ["wasm"] }
```

## Best Practices

### Code Organization
- **Single Responsibility**: Keep components focused
- **Reusability**: Extract common logic into hooks/functions
- **Type Safety**: Leverage Rust's type system
- **Error Handling**: Use Result types consistently

### Performance Guidelines
- **Minimize Signals**: Combine related state
- **Use Memos Judiciously**: Only for expensive computations
- **Lazy Load**: Split code with dynamic imports
- **Optimize Images**: Use appropriate formats and lazy loading

### Security Considerations
- **Validate Server Functions**: Never trust client input
- **Sanitize HTML**: Use text nodes for user content
- **CORS Configuration**: Set appropriate headers
- **Environment Variables**: Never expose secrets to client

### Developer Experience
```toml
# .vscode/settings.json
{
    "rust-analyzer.procMacro.ignored": {
        "leptos_macro": ["server"]
    },
    "rust-analyzer.cargo.features": ["ssr"]
}
```

### Common Patterns

#### Loading States
```rust
#[component]
fn DataView() -> impl IntoView {
    let data = create_resource(|| (), |_| fetch_data());

    view! {
        <Transition fallback=|| view! { <Skeleton /> }>
            {move || data.get().map(|result| match result {
                Ok(data) => view! { <DataDisplay data /> },
                Err(e) => view! { <ErrorDisplay error=e /> },
            })}
        </Transition>
    }
}
```

#### Form Handling
```rust
#[component]
fn ContactForm() -> impl IntoView {
    let submit_action = ServerAction::<SubmitContact>::new();

    view! {
        <ActionForm action=submit_action>
            <input type="text" name="name" required />
            <input type="email" name="email" required />
            <textarea name="message" required />

            <button type="submit" disabled=move || submit_action.pending()>
                {move || if submit_action.pending() {
                    "Sending..."
                } else {
                    "Send Message"
                }}
            </button>
        </ActionForm>

        <Show when=move || submit_action.value().get().is_some()>
            <p>"Message sent successfully!"</p>
        </Show>
    }
}
```

#### Authentication Pattern
```rust
#[derive(Clone)]
struct AuthContext {
    user: RwSignal<Option<User>>,
    login: Action<LoginCredentials, Result<User, AuthError>>,
    logout: Action<(), ()>,
}

#[component]
fn ProtectedRoute() -> impl IntoView {
    let auth = use_context::<AuthContext>()
        .expect("AuthContext not provided");

    view! {
        <Show
            when=move || auth.user.get().is_some()
            fallback=|| view! { <Navigate to="/login" /> }
        >
            <Outlet />
        </Show>
    }
}
```

## Conclusion

Leptos provides a powerful, type-safe foundation for building modern web applications in Rust. By following these guidelines and best practices, you can build performant, maintainable applications that leverage the full power of Rust's ecosystem while providing excellent user experiences.