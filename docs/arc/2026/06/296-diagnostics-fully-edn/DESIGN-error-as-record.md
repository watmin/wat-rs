# Arc 296 (part 2) — Errors are records satisfying `:wat::core::Error`

> **Status: DESIGN SETTLED (2026-06-30, co-designed + four-questions-ratified with the builder). Building.**
> Part 1 (`DESIGN.md`, `7f17054a`) made error *content* structured EDN via the `ToEdn` trait. This part makes errors
> **records satisfying one minimal surface**, and constrains the whole propagation path to it.

## Thesis
Every error is a wat **record satisfying one minimal surface, `:wat::core::Error`.** The entire propagation path —
`raise!`, `Failure`, recovery — accepts and carries **only** an `:wat::core::Error`. Structured EDN, always. **Never a
`HolonAST`, never a hologram, never a bare `Value`, never a stringified render.** The HolonAST abuse in `raise!` dies
because its gate becomes `:wat::core::Error` — not because it loosens to `Value` (that was a rejected framing).

## Why it opened
A sonnet fumbled on incoherent/stringly errors; the builder wants coherent errors so that stops. The error layer is the
one place wat stopped being wat: `raise!` (runtime.rs:11871) **demands `:wat::holon::HolonAST`** and **stringifies it**
into `Failure.message` — a crutch from before `EdnRepresentable` existed (`comms/mod.rs:102`; `Value` impls it at `:794`;
`HolonRepresentable: EdnRepresentable` is now a *specialization*). This arc makes errors coherent records.

## OUT OF SCOPE (affirmative cuts)
- **HolonAST's full removal** (1172 refs) — that is **arc 294** (the next arc). We only re-gate `raise!`; we touch none
  of the other HolonAST sites.
- **The holon layer / holograms** — untouched. Every surface still auto-derives its `$core-record`/`$holon-record` pair;
  a user still *opts in* to uplift an error to a hologram. **We reach for a hologram nowhere in error propagation.**
- **The `Display` human-render path** — stays (the human face; EDN is the wire/data face).

## The surface
```clojure
(:wat::core::defsurface :wat::core::Error
  :holder :wat::core::Record
  :features [message  <- :wat::core::String                       ; freeform instructions (producer)
             location <- :wat::kernel::Location                   ; the blame coordinate (defaults to `here`)
             causes   <- :wat::core::Vector<wat::core::Error>])    ; the causal tree (recursive; `[]` = leaf)
```
- **`message`** — freeform text; producer-supplied.
- **`location`** — *where the error is about* (decision **P**). Defaults to `(:wat::kernel::here)` (the caller/raise
  site); a producer **cites** a different coordinate (a parser's token in the input) as an override. **Mandatory — never
  Option, never a sentinel.** This is what annihilates `Span::unknown()`: a locationless error becomes a bug to fix.
- **`causes`** — the causal tree, recursive; producer-supplied; `[]` = a root/leaf.
- **`frames`** is **NOT** on the surface — it is the caller backtrace `raise!` stamps. **`Failure` = `:wat::core::Error` + `frames`.**

## The decisions behind it (four-questions ratified this session)
- **`location` = P (problem coordinate), not R (raise-site):** P scored YES/YES/YES/YES, R scored NO/YES/NO/NO; the
  long-term-stability bias disqualified R (the parse/check/type/config families must point into *input*, not the call site).
- **`causes` on the floor, `frames` off it:** grounded on *provenance* — the producer knows `causes`; only the runtime
  knows `frames` (via `snapshot_call_stack`, the Ruby-`caller` mechanism in `src/value/frame.rs`). `location` = `head(frames)` today.
- **`location` DEFAULTS to `here`, is not always-written:** the always-explicit `(here)` mechanic FAILED Honest + Good UX
  (a footgun that reproduces R's dishonesty as an opt-in, plus boilerplate). The default must be automatic; citing is the override.
- **`raise!` constrained to `:wat::core::Error`:** constraint engineering — a non-error has **no form** (`(raise! 42)` is a compile error).

## The primitives this arc adds
- **`(:wat::kernel::here)`** — returns the caller-top `Location` (wat exposure of `snapshot_call_stack().first()`). *Name: intueri.*
- **A wat-constructible `:wat::kernel::Location`** — so a producer can build/cite one (today it is Rust-populated only).
- **`deferror`** — sugar: declares an `:wat::core::Error`-satisfying record with the floor (`message` +
  `location`-defaulting-to-`here` + `causes`-defaulting-to-`[]`) so the user writes only their domain fields. *Name: intueri.*
- **`#[derive(WatErrorRecord)]`** — single-source: generates the record registration + EDN + tag from the Rust error
  definition, for the ~80 substrate errors (crowned N5). Per-phase tag namespaces (`#wat.check/…`, `#wat.runtime/…`; crowned N3).
- **`raise!` re-gate:** `(:wat::core::Error) -> :T`; the HolonAST gate is removed.
- **`Failure` converges** to `:wat::core::Error` (+ `frames`). An `assertion-failed!` result is an Error too (with its own `actual`/`expected` fields).

## Dependencies
- **`is_pure_type(Surface)` fix** — a `Record`-holdered surface is pure (the stale `Surface => false` stub at
  `check.rs:13718` predates surfaces having holders). **Unblocks `causes <- Vector<:wat::core::Error>`** (proven RED:
  `ImpureFieldInPureAggregate`). Holon-free; the `$holon-record` in that probe was the *wanted* surface-pair, not a bug.
- **9a (kwargs default + field-defaults)** — for the ergonomic forms (`:message …`, `location` omitted → default). QUEUED
  in the 293/294 close sequence. The keystone can prove the *shape* with positional ctors before 9a lands.

## The test surface = the user forms (these ARE the tests to prove)
Declare (`deferror`, domain fields only) · raise common (location auto = here) · raise cited (parser overrides location)
· wrap (causes tree) · the wall (`(raise! 42)` rejects) · handle (uniform `Error/message`·`Error/location`·`Error/causes`
+ `match` on the tagged record) · substrate errors read identically · `edn::read (edn::write e)` round-trips. See the
session's pitched forms; each becomes a co-located `.wat` fixture (`feedback_test_wat_is_colocated_fixture`).

## Decomposition (build in order; RED-probe each; FULL gate `cargo nextest run --release` green after each)
- **S1 — `is_pure_type`: a holder-pure surface is a pure field type.** Unblocks the recursive `causes` field. RED probe
  already proven (`ImpureFieldInPureAggregate` on a surface-typed recursive field). Tiny, isolated Rust fix. **← FIRST.**
- **S2 — the primitives:** `(:wat::kernel::here)` + a wat-constructible `:wat::kernel::Location`.
- **S3 — THE KEYSTONE:** the `:wat::core::Error` surface + `raise!` re-gated to it. RED probe: a wat error record is
  `raise!`d, caught, and read structurally (`message`/`location`/`causes`); a non-error rejects; it round-trips. Proves the core forms.
- **S4 — `deferror` sugar.**
- **S5 — `Failure` converges** to `:wat::core::Error` (+ `frames`); `assertion-failed!` yields an Error.
- **S6 — `#[derive(WatErrorRecord)]` + per-phase namespaces + retrofit the ~80 substrate errors.** The tail.
