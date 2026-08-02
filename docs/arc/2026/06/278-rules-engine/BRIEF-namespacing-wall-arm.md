# BRIEF — arm the namespacing wall (`Unnamespaced` beside `Reserved`)

**Anchor:** all work in `/home/watmin/work/holon/wat-rs/`. Use `git -C /home/watmin/work/holon/wat-rs`
for any git read. Any path containing `.claude/worktrees/` is harness state and must not be operated on.

**You are a rider, not the orchestrator. Ending your turn ENDS you** — it does not suspend you, and
nothing will wake you. There is no notification coming. Run every command in the FOREGROUND and block
on it: your turn ends when the numbers are in your hands, not when a command is launched.

## The work, in one paragraph

A top-level name in wat must be namespaced. Only fn arguments and `let` bindings may be bare, and those
are lexical — they never reach a registration gate. So *everything* that reaches the shared gate must be
namespaced, with **no exceptions to carve**. Today nothing enforces this. Add an `Unnamespaced` verdict
to `resolve::registration`, make `gate()` return it, and give every door that consumes a `Registration`
a located error for it. `src/` only.

**Adding the variant is the worklist.** `Registration` is matched exhaustively; the moment you add a
variant, rustc names every consumer that must decide. Do not grep for callers — let the compiler
enumerate them.

## Rooms — read in order, and why

1. `src/resolve/registration.rs:49-59` — the `Registration` enum. The variant goes here.
2. `src/resolve/registration.rs:76-89` — `gate()`. The new arm sits inside the `Existing::Absent`
   branch, beside the `is_reserved_prefix` check.
3. `src/resolve/registration.rs` (same file) — `is_reserved_prefix`. Put the new predicate beside it,
   and re-export it from `src/resolve/mod.rs` exactly the way `is_reserved_prefix` is.
4. `src/types.rs:552-561` — the TYPE door. **This is the pattern to copy**: it already maps
   `Reserved → Err(TypeError::new(span, TypeErrorKind::ReservedPrefix { name }))`. A span is in scope.
5. `src/runtime.rs:916-923` — the user-`def` door. The second pattern:
   `Reserved => return Err(RuntimeError::new(form_span, RuntimeErrorKind::ReservedPrefix(path)))`.
6. `src/check/env.rs:311-330` (`register_overlay`) and its caller at `:150-165`. `register_overlay`
   returns `Err(verdict)`; the caller `eprintln!`s it. **Leave that shape as it is** — it replays an
   already-frozen table at `Privilege::Stdlib` and is not a user-facing door. Just keep the match total.
7. `src/macros/registry.rs:63` — the macro door.
8. `src/runtime.rs` `:949` `:2496` `:2682` `:2743` `:2856` `:3383` `:3457` `:6522` — the variadic-def,
   alias, struct-constructor, accessor and defclause doors. **`:2743` and `:2856` are where STOP-1
   lives** (they register *generated* names).

## Implementation sketch — fill it, do not invent a different shape

```rust
// src/resolve/registration.rs
pub enum Registration { Insert, NoOp, Duplicate, Reserved, Unnamespaced }

/// A top-level name must carry a namespace. Only fn args and let-bindings may be bare,
/// and those are lexical — they never reach this gate.
///
/// NOT "starts with ':' and contains '::'": parametric heads drop the leading colon
/// (`wat::kernel::Peer`), recorded in arc 170's 24t seam. The test is containment.
pub fn is_namespaced(name: &str) -> bool { name.contains("::") }

pub fn gate(name: &str, privilege: Privilege, existing: Existing) -> Registration {
    match existing {
        Existing::Equivalent => Registration::NoOp,
        Existing::Divergent  => Registration::Duplicate,
        Existing::Absent => {
            if !is_namespaced(name) {
                Registration::Unnamespaced
            } else if privilege == Privilege::User && is_reserved_prefix(name) {
                Registration::Reserved
            } else {
                Registration::Insert
            }
        }
    }
}
```

**Ordering is deliberate and must not change:** `Equivalent → NoOp` stays first, so an idempotent
re-declaration of a name that already exists is still a no-op and the freeze/replay path cannot break.
The wall therefore fires only on a *first* registration. `Unnamespaced` is tested before `Reserved`
because a bare name cannot be reserved (every reserved prefix contains `::`), and "not namespaced" is
the more specific truth about it.

**Two new error kinds**, beside the existing `ReservedPrefix` in each taxonomy:
`TypeErrorKind::UnnamespacedName { name }` and `RuntimeErrorKind::UnnamespacedName(String)`.

**The message must teach the fix** (a diagnostic that names no remedy teaches nothing):

```
top-level name ':no-ns' is not namespaced — only fn arguments and let-bindings may be bare;
give it a namespace, e.g. ':my::no-ns'
```

`Privilege::Stdlib` is held to the rule too — do not add a privilege escape.

## Blast radius

`src/` **only**. Do **NOT** edit any `.wat` file. There are 24 wat files holding 57 bare names; fixing
them is a separate pass the orchestrator owns. Your build must go green with the variant added and every
door deciding — the wat corpus is not your problem and you must not touch it.

## STOP triggers — these are rejection criteria. Ship nothing and report.

- **STOP-1 — a GENERATED name reaches the gate bare.** Struct/record accessors (`Type/method`), enum
  variants, macro-minted companions, the surface-minted `<Surface>::<op>/Request|Response` aliases.
  If the build or a run shows the substrate rejecting *its own emission*: **HALT.** Report the exact
  name, the registering `file:line`, and how it was constructed. Do **not** exempt generated names, do
  **not** special-case a prefix, do **not** route them around the gate.
- **STOP-2 — the change cannot go green from `src/` alone.** If making the build pass requires editing
  a `.wat` file, a fixture, or a test's expected output: **HALT** and report what demanded it.
- **STOP-3 — a door has no span to build a located error from.** Report which door. Do **not** fall
  back to an `eprintln!` at a user-facing door, and do not silently drop the verdict.

## Gate — build-only, and one unit test

1. `cargo build --release --all-targets` → **exit 0, zero warnings.** This is the load-bearing gate.
2. Add unit tests in `src/resolve/registration.rs`'s existing `mod tests`:
   - `gate(":no-ns", User, Absent) == Unnamespaced`
   - `gate(":my::ok", User, Absent) == Insert`
   - `gate("wat::kernel::Peer", Stdlib, Absent) == Insert` (the no-leading-colon parametric head)
   - `gate(":no-ns", Stdlib, Absent) == Unnamespaced` (stdlib is held to it)
   - `gate(":no-ns", User, Equivalent) == NoOp` (idempotent replay still short-circuits)
   Run them: `cargo test --release --lib resolve::registration -- --nocapture`.
3. Report the located error text produced for a bare `defn`. Write the probe to a **temp path outside
   the repo** (e.g. `/tmp/ns_probe.wat`) — a `.wat` file under `wat-scripts/` is loaded and
   type-checked by a corpus gate, so a deliberately-bad one there would go permanently RED:
   ```
   printf '(:wat::core::defn :no-ns [] -> :wat::core::i64 0)\n' > /tmp/ns_probe.wat
   ./target/release/wat --check /tmp/ns_probe.wat
   ```
   Paste the exact output. It should be the new located error, not `UnresolvedReference` or a panic.

**Do NOT run `cargo nextest`.** The orchestrator weighs the full floor centrally, once, on a quiescent
tree. Do not commit, push, stash, or revert anything.

## Report

Return: the diff summary per file; the `--all-targets` result; the unit-test result; the verbatim
`--check` output from step 3; the **complete list of doors rustc named** when you added the variant and
what each now does with it; and any STOP you hit, with its evidence.
