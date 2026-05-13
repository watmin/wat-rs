# Arc 196 — Runtime env-var interactions

**Status:** STUB. Captured 2026-05-15 from user direction. Not yet designed.

## Goal

Wat programs need to read and (probably) write OS environment variables. Today there is no substrate primitive for this — wat-cli inherits env from the OS shell but nothing inside wat can introspect or modify it.

User direction 2026-05-15:
> *"we'll need to introduce env var interactions — I'm thinking `:wat::runtime::ENV/get`, `:wat::runtime::ENV/set`"*

## Proposed API (sketch — final form earned through four-questions)

```scheme
;; Read a var by name; returns Option<String>
(:wat::runtime::Env/get "PATH")
;; → :wat::core::Some<:wat::core::String>("/usr/local/bin:...")

;; Write a var; returns nil (or Result if we add failure modes)
(:wat::runtime::Env/set "MY_VAR" "value")
;; → :wat::core::nil
```

### Naming note — PascalCase, not ALL_CAPS

User's initial sketch used `ENV` (ALL_CAPS). The substrate's existing convention (post arc 109 § D' / arc 167 / arc 168) is PascalCase for Type names with `/` verbs: `Option/expect`, `Result/ok`, `Process/stdin`, `Vector/concat`. ALL_CAPS reads as a constant/sentinel; PascalCase `Env` matches sibling types. The full path becomes `:wat::runtime::Env/get` / `:wat::runtime::Env/set`. Verify via `/gaze` when arc gets DESIGNed.

(The `:wat::runtime::*` namespace already has lowercase ambient values per slice 1e — `:wat::runtime::argv`, `:wat::runtime::current-thread`. `Env` is a Type with verbs, not an ambient value, so PascalCase is consistent with Type-attached method shape.)

## Open questions

1. **Set semantics — runtime mutation vs startup-only.** POSIX `setenv` is NOT thread-safe; readers may observe partial writes if `getenv` runs concurrently in another thread. Three options:
   - **(a) Runtime-set allowed, with a discipline that callers serialize via channel** — flexible but easy to misuse
   - **(b) Startup-only set, runtime read-only** — safe but limits use cases; users wanting env mutation must opt into a startup-phase
   - **(c) Set via a substrate-controlled service** — `EnvService` like the stdio trio; serializes mutations via send/recv
   
   Per failure-engineering: option (c) is most aligned with the ZERO-MUTEX discipline (don't construct the racy situation). Option (b) is the simplest safe answer. Settle via four-questions when arc gets DESIGNed.

2. **Inheritance semantics across spawn-process.** OS-level: forked children inherit parent's environ snapshot. Wat-level: should `(:wat::runtime::Env/set ...)` in the parent affect the child's view BEFORE fork? AFTER fork? Today's POSIX-fork semantics: env propagates at fork; post-fork mutations are isolated per-process. Document this explicitly.

3. **Read-of-modified semantics.** If a wat program does `Env/set "FOO" "bar"` then `Env/get "FOO"`, the read MUST observe the write. POSIX libc honors this for single-threaded callers. Multi-threaded: see Q1.

4. **Should `Env/get` return `Option<String>` or `String`?** Missing var is common; Option-ful return is honest. Pattern: same as `HashMap/get`.

5. **Set with non-String values?** OS env is bytes; ASCII / UTF-8 lazy convention. Substrate either accepts only `:wat::core::String` (mirror libc) or accepts `:wat::core::Bytes` (more honest). Start with String; surface in DESIGN if Bytes turns out needed.

6. **`Env/del` / `Env/keys` / `Env/has?`?** Companions. POSIX has `unsetenv` (del). `Env/keys` enumerates. `Env/has?` is `(option::is-some? (Env/get k))` — derivable; not needed as a primitive.

7. **Interaction with arc 191 `exec-program` (hot-reload).** New universe inherits env; `Env/set` calls in old universe should propagate (they're process-global). Document explicitly.

## Out of scope (until DESIGN)

- A wat-level type for "the env" as a value (e.g., `Env::snapshot()` returning a HashMap). Can be built on top of `Env/keys` + `Env/get` if needed; substrate primitive stays minimal.
- Default values / fallback. User-side wat handles via `(option::expect (Env/get "VAR") "VAR must be set")`.
- Sandboxing / filtering for spawn-process children (env scrubbing). Future arc if needed.

## Cross-references

- Arc 109 § D' — Type/verb shape precedent (`Option/expect`, `Result/ok`)
- Arc 170 slice 1e — `:wat::runtime::argv` + `:wat::runtime::current-thread` ambient runtime; precedent for the `:wat::runtime::*` namespace
- Arc 170 RUNTIME-BOOTSTRAP-BACKLOG — `bootstrap_wat_vm_process` is where env propagation hooks (if Stone A's BootstrapArgs ever carries env override)
- Arc 109 § I — kernel/string family rename queue (parallel discipline: PascalCase Type/verb shape)
- Arc 191 stub (hot-reload `exec-program`) — env survives universe-swap; document
- POSIX `getenv(3)` / `setenv(3)` / `unsetenv(3)` — substrate-implementation reference
- Rust `std::env::{var, set_var, vars, remove_var}` — Rust-side reference for the substrate impl
