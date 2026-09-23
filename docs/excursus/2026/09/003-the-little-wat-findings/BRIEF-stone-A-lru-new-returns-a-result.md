# BRIEF — STONE A: `Lru::new` returns a Result

**Against `reason/little-wat-findings`.** Read `DESIGN-stone-A-lru-new-returns-a-result.md` beside
this first — it holds the mandate, the measured blast radius, and the one contract decision.

## The work in one paragraph

Turn `:rust::cache::Lru::new`'s `panic!` into `Result<Self, RawFault>`, propagate the `Result`
through `:wat::cache::Lru/new` and `:wat::cache::HolographicLru/new`, and move every caller —
including `lru-svc`'s durable-rebuild `:init`. A wat program that passes a non-positive capacity
must get a wat error naming `:wat::cache::Lru/new` and carrying the user's span; it must never
see a Rust panic or a `RUST_BACKTRACE` note.

## Read in order, and why

1. **`src/rust_deps/sqlite.rs`, § "Errors-as-values — the exact mechanism"** (module doc, ~lines
   11-35) — **this is the pattern; copy it.** `Sqlite::open`/`open_readonly` already return
   `Result<Self, RawFault>` through `#[wat_dispatch]` with zero macro changes. `RawFault =
   (i64, String, String) = (code, diagnostic, message)`.
2. **`src/rust_deps/cache.rs:44-73`** — the module's own "Failure surface" note. It states the
   merits, names the arc-109 NOTE as the tracking home, and says the deferral reason expired.
   ⛔ **The doc comment at `:100` is now FALSE** (*"the dispatch macro cannot yet marshal a
   method-internal error"*) — rewrite it, do not leave it.
3. **`src/rust_deps/cache.rs:90-109`** — the panic itself.
4. **`wat/cache.wat:85-88`** — the wat surface; **`:211`** the durable-rebuild `:init`;
   **`:289-295`** `HolographicLru/new`; **`:401`** the second factory.
5. **`docs/arc/2026/04/109-kill-std/NOTE-the-cache-lru-panics-…md`** — the standing ruling. Its
   header reads "awaiting MANDATE"; **the mandate is in the DESIGN and you must update that
   header** so the note stops asking for a decision that has been made.

## Implementation sketch

```rust
// src/rust_deps/cache.rs
pub fn new(capacity: i64) -> Result<Self, RawFault> {
    if capacity <= 0 {
        return Err(( /* code */, ":wat::cache::Lru/new".into(),
                     format!("capacity must be positive; got {capacity}") ));
    }
    …
}
```
⭐ The `diagnostic` field is where the **user-facing** name goes — `:wat::cache::Lru/new`, never
`:rust::cache::Lru/new`. Naming the internal shim is half of what F-084 reports.

## Blast radius

`src/rust_deps/cache.rs` · `wat/cache.wat` · the `.wat` call sites listed in the DESIGN's table ·
the arc-109 NOTE header. **No other `src/` module. `put`/`get` are NOT converted.**

## STOP triggers

1. **`put`/`get` start looking like they should be converted too.** They are affirmatively out of
   scope and the standing note warns against converting all three for symmetry. Report the
   argument; convert nothing.
2. **The `Result` cannot propagate cleanly through `HolographicLru/new`** — e.g. a service
   `:init` cannot express a failing rebuild. STOP and report what the surface actually allows.
   Do NOT swallow the error to make it compile; that re-creates the defect one level up.
3. **A caller you did not expect appears.** The DESIGN's table is measured, not exhaustive by
   construction — derive it yourself with a grep over `Lru/new` and report any row it lacks.
4. **Any gate in `tests/lint/` reddens that you did not add.** Capture the whole untruncated
   block, name the test and the arm, report. ⛔ Do not re-run first.

## ⚠ The cross-effect you must handle, not discover

`tests/lint/little_wat_findings_board__f083_holographic_lru_reput.wat` calls
`HolographicLru/new`. Its board row pins the-little-wat **F-083** as `(check 0, run 0, stdout
"0")` — a program that runs clean and prints a WRONG answer. Your change alters how that fixture
must be written. **Update the fixture and re-measure its row.** ⛔ **F-083 is a different defect
and this stone does not cure it** — after your change the row must still pin a re-put emptying
the cache. If it no longer can, STOP and report; do not delete the row.

## Prove it

- A wat program calling `(:wat::cache::Lru/new 0)` produces a **wat error**, naming
  `:wat::cache::Lru/new`, with **no `RUST_BACKTRACE` note anywhere on stderr**.
- The same for `HolographicLru/new`.
- ⭐ **Mutation:** restore the panic, watch your new test go RED, restore the cure. A gate that has
  never failed is not a gate.
