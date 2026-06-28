# Why this project exists

**I wanted a Pomodoro app with this feature set — and the options I tried each
forced a compromise: Electron heft, closed source, or freemium limits. So I built
my own, and it's free (MIT). See the [✨ What it does](../README.md#-what-it-does)
section for the full list.**

The codebase is also a sandbox for three things I wanted to explore along the way.
They shape the structure, not the existence — so everything below is about *how*
the code is written, not *why* the app exists.

## 🤖 Agentic AI development

Much of this code was written *with* AI agents. The real work wasn't generation —
it was exploration: choosing trade-offs, reviewing output, debugging when agents
went wrong, and learning enough Rust/Tauri/DDD to judge the results. The
[commit history](../commits) is the paper trail — terse messages, fix-then-iterate
cycles, the rhythm of engaged authorship.

## 🏗️ Clean Architecture & DDD

Deliberately "over-engineered" for a Pomodoro app — that's the point, and it's
what lets the codebase double as a reference for these patterns. Framework-agnostic
domain core, clear use-case boundaries, swappable infrastructure adapters, and the
dependency rule (UI → Infrastructure → Use Cases → Domain).

The real goal here is **learning by doing**: these patterns stick best when you
apply them to solve real problems in a real app, not by studying toy examples. This
*is* that real app for me — a vehicle to internalize Clean Architecture and DDD,
with every decision weighed against production constraints, not demo convenience.

It's not a finished showcase — it's a work in progress. The patterns here are a
moving target, refined as the app grows and as the trade-offs become clearer in
practice.

## 🦀 Rust + Tauri

A vehicle for idiomatic Rust (traits, async, error handling, Diesel) and Tauri's
native desktop model from a single Rust + web codebase — without Electron's weight.

## For contributors

If the structure looks heavier than the feature set warrants — now you know.
Contributions that follow these patterns are welcome; this is a place to explore
trade-offs, not minimize them.

## Reference projects worth studying

Projects I learned from while shaping this codebase:

- [gothinkster/realworld](https://github.com/gothinkster/realworld) — intentionally over-engineered demo app
- [bxcodec/go-clean-arch](https://github.com/bxcodec/go-clean-arch) — Clean Architecture teaching project
- [android10/Android-CleanArchitecture](https://github.com/android10/Android-CleanArchitecture) — Fernando Cejas's reference implementation
