# Why this project exists

**I wanted a Pomodoro app with this feature set — and nothing out there covered it
without being Electron, closed-source, or freemium. So I built it, and it's free
(MIT). See the [✨ What it does](../README.md#-what-it-does) section for the full
list.**

The codebase is also a sandbox for three things I wanted to explore along the way.
They shape the structure, not the existence.

## 🤖 Agentic AI development

Much of this code was written *with* AI agents — but not hands-off. The real work was
exploration: choosing trade-offs, reviewing output, debugging when agents went wrong,
and learning enough Rust/Tauri/DDD to judge the results. The
[commit history](../../commits) is the paper trail — terse messages, fix-then-iterate
cycles, the rhythm of someone actually engaged, not blindly accepting output.

## 🏗️ Clean Architecture & DDD

Deliberately "over-engineered" for a Pomodoro app — that's the point. Framework-
agnostic domain core, clear use-case boundaries, swappable infrastructure adapters,
and the dependency rule (UI → Infrastructure → Use Cases → Domain).

## 🦀 Rust + Tauri

A vehicle for idiomatic Rust (traits, async, error handling, Diesel) and Tauri's
native desktop model from a single Rust + web codebase — without Electron's weight.

## For contributors

If the structure looks heavier than the feature set warrants — now you know.
Contributions that follow these patterns are welcome; this is a place to explore
trade-offs, not minimize them.

## Reference projects worth studying

- [gothinkster/realworld](https://github.com/gothinkster/realworld) — intentionally over-engineered demo app
- [bxcodec/go-clean-arch](https://github.com/bxcodec/go-clean-arch) — Clean Architecture teaching project
- [android10/Android-CleanArchitecture](https://github.com/android10/Android-CleanArchitecture) — Fernando Cejas's reference implementation
