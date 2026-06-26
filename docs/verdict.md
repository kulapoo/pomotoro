# Why this project exists

Pomotoro is a personal learning project. It is not built to be the simplest possible
Pomodoro app — it is intentionally a sandbox where I can put three things into
practice at once:

## 🤖 Agentic AI development

The primary goal is to experiment with building software *with* AI coding agents —
not just autocomplete, but agents that plan, edit, test, and refactor across a real
codebase. I want to understand:

- Where agentic AI accelerates real work and where it stalls
- Prompt patterns and workflows that hold up under a layered architecture
- How to keep human judgment in the loop on design decisions that matter
- What a sustainable AI-assisted development workflow actually looks like day to day

This README and the surrounding docs are part of that experiment — they are written
and maintained alongside the code with the same tools.

## 🏗️ Clean Architecture & DDD

The codebase follows Clean Architecture and Domain-Driven Design. For a Pomodoro app
this is deliberately "over-engineered" — and that is the point. I want hands-on
reps with:

- A framework-agnostic domain core (zero Tauri dependencies)
- Clear use-case boundaries that orchestrate domain logic
- Infrastructure as swappable adapters (repos, event bus, audio, timer tick)
- The dependency rule: UI → Infrastructure → Use Cases → Domain

If a simpler architecture would do, that is not a bug here — it means the patterns
are being practiced on purpose.

## 🦀 Rust

Everything above is implemented in Rust, which is the third pillar. This project is a
vehicle for honing idiomatic Rust: trait design, async with Tokio, error handling,
persistence with Diesel, and structuring a larger workspace without it collapsing
into a ball of `Arc<Mutex<...>>`.

## For contributors

If you land here and wonder why the structure looks heavier than the feature set
warrants — now you know. Contributions that follow these patterns are welcome,
questions about the architecture are encouraged, and learning is prioritized over
simplicity. This is a place to explore trade-offs, not minimize them.

## Reference projects worth studying

- [gothinkster/realworld](https://github.com/gothinkster/realworld) — intentionally over-engineered demo app
- [bxcodec/go-clean-arch](https://github.com/bxcodec/go-clean-arch) — Clean Architecture teaching project
- [android10/Android-CleanArchitecture](https://github.com/android10/Android-CleanArchitecture) — Fernando Cejas's reference implementation
