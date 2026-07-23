# BRIEF — Strike B: `struct-new` respects Nature (the wall) + reclaim the Failure heretics

> **Tier:** sonnet shadowdancer. **Arc:** 278 item-c tail. **HEAD:** `5ec387a2` (Strikes M + A committed).
> This closes the Failure-canonicalization: A gave us the one constructor + fixed the client path; B walls
> the class so `struct-new :<record>` is uncompilable, and reclaims the remaining heretics the wall lights.

## Why (one paragraph)

`:wat::core::struct-new` builds a `Nature::Struct` aggregate. It is currently **unchecked** on its
type-name argument — `src/check.rs:5843` falls into the `_ => {}` silent-pass ("struct-new — intentional
runtime-only dispatch, no scheme"). So `struct-new :wat::kernel::Failure` compiles even though `Failure` is
`Nature::Record` (a crash cause crosses the wire — pure EDN; arc 293.W.2b), producing a value the Record
accessor `Failure/message` can't read. Add one check-time arm: **`struct-new` on a record-natured (or enum)
type is a located compile error.** This imposes "all Failures must be a record" transitively (the only
wat-source way to mint a non-record Failure was `struct-new :Failure`), walls **every** record type at
once, and — landing green after the reclamation — proves the canonicalization complete.

## Phase 1 — the wall (`src/check.rs`, the `struct-new` seam ~:5843)

Add a check arm for head `k == ":wat::core::struct-new"` (before/replacing the `_ => {}` fall-through at
:5843). Resolve the **type-name argument** (`items[1]` — the keyword right after the head; ground the exact
indexing from how the runtime reads it: `eval_struct_new`, `runtime.rs:4285`). Look it up:
`env.types().get(<type-fqdn>)`. Then:

- resolved to a **record-natured** TypeDef (`nature == Nature::Record` or `HolonRecord`) **or an Enum** →
  emit a located `TypeError` (reuse the existing error kinds; a `TypeMismatch`/`MalformedForm`-style located
  error). Message names the type and the remedy, e.g. *"struct-new on record-natured `:wat::kernel::Failure`
  — construct it as a record (its kwargs ctor, or `:wat::kernel::message-only-failure`)."*
- resolved to a **struct-natured** TypeDef (`nature == Nature::Struct`) → OK, fall through unchanged.
- **not resolved** (unknown type) → leave to the existing handling (do NOT turn unknown-type into this
  error; that's a separate concern).

Ground the TypeDef shape at `src/types.rs:371` (`enum TypeDef`) + `:224-230` (the struct/record def carries
`nature: Nature` as its categorical field; `Nature` at `:133`). Do NOT change `eval_struct_new` (runtime) —
the **checker is the gate**; once it rejects, no such form reaches runtime.

## Phase 2 — reclaim what the wall lights (atomic with Phase 1)

The wall lights record-natured `struct-new` sites RED at load. **Expected RED set (grounded): the 6
`struct-new :wat::kernel::Failure` sites**, all message-only, all → `(:wat::kernel::message-only-failure <msg>)`:

- `wat/test.wat:713`, `:816`, `:874`
- `wat/kernel/hermetic.wat:64`, `:97`
- `wat/kernel/sandbox.wat:54` (message is `(:wat::core::string::concat "startup: " (StartupError/message err))` —
  wrap that whole expr as the helper's arg)

**Verify `wat/test.wat:1045` `struct-new :wat::test::RunResultIO`** — confirm its Nature. If **Struct**, the
wall allows it (leave it). If **Record**, the wall lights it too → convert to its record kwargs ctor
`(:wat::test::RunResultIO :field val …)` (ground its fields); if it has no clean record ctor, **STOP-2**.

Everything else stays: `struct-new :wat::kernel::RunResult` (×12, `Nature::Struct` — correct),
`struct-new :my::Point`/`:myapp::Point` (on `defstruct` Points — Struct, correct). The wall must leave these
green.

## Phase 3 — the RED gate (prove the wall catches the class)

Add a probe proving a fresh `struct-new :Failure` is now a **compile error**. A `.wat.bad` fixture with
`(:wat::core::struct-new :wat::kernel::Failure "x" :wat::core::None (:wat::core::Vector :wat::kernel::Frame) :wat::core::None :wat::core::None)`
+ a `.rs` test that `startup_from_file(...).expect_err(...)` and asserts the nature-violation error is
present — **using the `assert_check_error_present!` macro Strike M added** (membership, not `errs[0]`).
Place it beside the other arc-278 service/check probes. This is the disconfirming proof: before the wall
this form compiled + built a wrong-nature value; after, it's a located check error.

## STOP triggers

- **STOP-0:** the wall lights MORE than {the 6 Failure sites, + RunResultIO}. That means a broader
  record-natured `struct-new` usage exists we didn't scope — STOP and report the full RED set; do NOT
  mass-convert blindly.
- **STOP-1:** the general wall breaks a **struct-natured** site (RunResult / a defstruct Point goes RED).
  That's a bug in the wall (it must ALLOW structs) — fix the wall's nature check; do NOT narrow the rule to
  "only Failure" (it must stay general: struct-new respects Nature).
- **STOP-2:** RunResultIO is a Record with no clean record constructor — STOP, report (needs a ctor decision).
- Do NOT edit `eval_struct_new` / the runtime. Do NOT touch Strike A/M's committed work.

## Verify (weigh by your own re-run)

1. `./target/release/wat --check` clean on every edited `.wat` (post-conversion; the wall now rejects a
   fresh `struct-new :Failure`).
2. The RED probe: its `.rs` test passes (i.e. the bad form IS rejected with the nature error).
3. **Whole release floor:** `cargo nextest run --release` — READ THE SUMMARY yourself; **4206/0** (the M-fixed
   deterministic floor), self_scheduling ×2 still `#[ignore]`'d. The struct-new-RunResult/Point tests stay
   green (proving the wall allows structs). Run twice; both `0 failed`.

## Deliverable

The `struct-new` Nature wall in `src/check.rs` + the 6 (or 7 w/ RunResultIO) reclaimed sites + the RED probe.
Report: (1) the wall arm's final form + the exact error it emits; (2) the actual RED set the wall lit
(confirm it matched the expected scope); (3) RunResultIO's nature + disposition; (4) two floor Summaries
(both 0-failed); (5) `git diff --stat`. Do NOT commit — leave it for the orchestrator to weigh.

## Blast radius

`src/check.rs` (one arm at the struct-new seam) + `wat/test.wat` + `wat/kernel/hermetic.wat` +
`wat/kernel/sandbox.wat` (the 6 Failure conversions) + one new `.rs`/`.wat.bad` RED probe pair
(+ `wat/test.wat:1045` only if RunResultIO is a Record). NO runtime changes. NO Strike A/M files.
Scratch logs → `/tmp/claude-scout/`.
