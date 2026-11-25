# Rust build cache

**Danger level:** Low · **Rule:** none (the module's code, `modules/stale-projects`) · **Since:** 0.1.0

**Example:** `target/debug/incremental` of a Rust project

## What it is

Files Cargo keeps only to build Rust projects faster, often gigabytes, even in a project you work on every day. Made by
Cargo and the Rust compiler on every build, and by cargo llvm-cov.

## Where it lives

Inside a Rust project's `target`: `target/*/incremental`, `target/*/*/incremental` (a target triple's) and `target/llvm-cov-target`.

## What happens if you remove it

Your code and your built programs stay; the next build takes longer once. They come back with the next build.

## How the app decides

The module `stale-projects`. Unlike the other three, a cache is offered by its own age, whatever the project's: when nothing in it changed for 7 days (a constant, not a setting), so a project built every day keeps its cache. A cache whose change time cannot be told, or that could not be read in full, is left out, and so is a restored scan. When the whole `target` of a stale project is offered, the caches inside it are not offered twice. Nothing is run and links are not followed. The folders go to the Trash.
