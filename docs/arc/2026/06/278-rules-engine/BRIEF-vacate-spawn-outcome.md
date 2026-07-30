# BRIEF — vacate `SpawnOutcome`: purgare the arc-060 join-result chain

> **The work in one paragraph.** A locus is `:user::main` in its own context — it communicates by
> channel and has **no meaningful return value** (builder-ruled 2026-07-30; arc 170's IPC triangle:
> `:user::main -> nil`, complex values go to stdout). The arc-060 `SpawnOutcome {Ok/RuntimeErr/Panic}`
> is the pre-170 shape — *spawn a **fn**, get a `Value` back* — and every one of its producers and
> consumers was retired by the IPC de-prime (24r–24t). It now has **zero constructors at three
> levels**, its death-reason job is carried structurally by `recv'` → `Lost[LociDiedError]`, and its
> **name is wanted** by the ratified creation wall (`:wat::kernel::SpawnOutcome<I,O>`, the twin of
> `ConnectOutcome`). Delete the chain. That vacates the name and drives the clippy floor to zero.

## Why this is a deletion and not the rename the prior brief drew

`BRIEF-spawn-outcome-wall.md` Phase 1 renamed this value to `Demise` — the death bookend of
`spawn`/`demise`. **That premise is disproven and the brief is SUPERSEDED** (see its banner). Its
Phase 1 rested on 24m's claim that *"the INTERNAL one-shot `SpawnOutcome` channel SURVIVES → Demise
is a rename-on-remainder."* It does not survive: those cited lines now hold `died_error_payload_message`,
and nothing constructs the chain. 24m conflated *the death path survives* (**true** — verified at
`kernel/spawn.rs:717/730` → `runtime.rs:22716` `loci_died_error_from_reason`) with *the SpawnOutcome
channel survives* (**false**). Deaths ride their own structured EDN channel and always did.

With a locus that only ever sends: `Returned[v]` has no subject, and `Errored`/`Panicked` duplicate
what `Lost[LociDiedError]` already delivers. **Demise has no job.** The name-vacating that Phase 1
existed to accomplish is achieved by deletion instead.

## Read in order (the rooms — every site, grounded 2026-07-30)

**1. `src/value/value.rs` — the definitions and the `Value` arms.** This is the whole substance;
everything else is a re-export or a match arm the compiler will name.

| line | what | action |
|---|---|---|
| `:169` | `wat__kernel__ProgramHandle(Arc<ProgramHandleInner>)` — the `Value` variant | **delete the variant** |
| `:582` | doc comment listing the opaque variants | scrub the mention |
| `:680` | `PartialEq` arm | delete |
| `:906-907` | `is_atomizable` `unreachable!` arm | delete |
| `:1092-1121` | `SpawnOutcome` + its doc | **delete the enum** (the clippy site) |
| `:1100` | the `rune:solvere(historical-shape)` marker naming both types | delete — its subject is gone |
| `:1123-1150` | `ProgramHandleInner` + its doc | **delete the enum** |
| `:1184-1186` | key-eligibility preamble naming `ProgramHandle`/`Forked` | scrub the mention; **keep** the `ChildHandle` reasoning |
| `:1444-1451` | the key-eligibility table's `ProgramHandle` row + comment | delete the row |
| `:1723` | `type_name` arm | delete |

**2. The two re-export lines** — `src/value/mod.rs:45` and `src/runtime.rs:715`. Drop `SpawnOutcome`
and `ProgramHandleInner` from each list; leave every other name alone.

**3. The remaining `Value` arms** — the compiler names each the instant the variant goes:
`src/value/observe.rs:417` · `src/closure_extract.rs:1994` · `src/edn_shim.rs:3713` ·
`src/runtime.rs:7229` · `src/runtime.rs:9909`.

**4. Prose** — `src/value/mod.rs:10`, `src/runtime.rs:713`, `src/runtime.rs:21953`,
`src/runtime.rs:21990`, `wat/kernel/channel.wat:30`.

## The comment rule (do not flatten this — it is the FM-14 classification)

- A comment that describes the type as **live or transitional** is a lie once the type is gone →
  **rewrite or delete**. (`value.rs:1100`'s "kept transitionally, they relocate at the migration
  stone"; `mod.rs:10`; `runtime.rs:713`.)
- A comment that **records a retirement** is history → **keep**, and extend it to record this one.
  `runtime.rs:21953` already reads *":wat::kernel::spawn retired in arc 114. The arc-060
  SpawnOutcome…"* — that is a tombstone; update it to say the arc-060 value itself was purged here
  and why (a locus has no return value), rather than deleting the lineage.
- `runtime.rs:21990`'s *"Arc 105c: `SpawnOutcome::Panic` widened to…"* documents a function that
  still exists; re-word it so it stops naming a deleted type.

## Blast radius — bounded

`src/value/value.rs`, `src/value/mod.rs`, `src/value/observe.rs`, `src/runtime.rs`,
`src/edn_shim.rs`, `src/closure_extract.rs`, and the one comment in `wat/kernel/channel.wat`.

**No new types. No renames. No `.wat` code changes** (the single `.wat` hit is prose).
**Do not touch** `ChildHandle` (`src/process/handle.rs`) or `Value::wat__kernel__ChildHandle` — it is
an independent type with its own `Drop` and its own key-eligibility row, and it carries the
pdeathsig/lifeline custody. **Do not touch** any outcome wall (`RecvOutcome`/`SendOutcome`/
`ConnectOutcome`/`AcceptOutcome`/`CloseOutcome`) or the `LociDiedError` death path.

## STOP triggers — halt and surface; ship nothing past one of these

- **STOP-1 — a real constructor.** If anything anywhere *builds* a `SpawnOutcome::*`,
  `ProgramHandleInner::*`, or `Value::wat__kernel__ProgramHandle(...)` — as opposed to matching or
  naming it in a type position — **STOP**. The premise of this stone is that the chain has no
  producer; a producer disproves it. Report the site.
- **STOP-2 — a wat-facing surface.** If `:wat::kernel::ProgramHandle` turns out to be registered as a
  wat type (in `src/types.rs` or `src/check.rs`) or referenced by executable `.wat` (not a comment),
  **STOP**. The scout found neither; either finding changes the shape of the work.
- **STOP-3 — the cascade leaves the named blast radius.** If deleting the variant forces a change in
  a file not listed above, **STOP** and report which and why. The expectation is that the compiler's
  list is exactly the rooms above.
- **STOP-4 — a test's subject.** If a test *fails to compile* because its subject is one of these
  types, **STOP** and name it rather than deleting or rewriting the test. Disposition is the
  orchestrator's call (subject-is-dead → annihilate; vehicle → re-point).

## Gate

`cargo build --release --all-targets` → exit 0, **zero warnings**. Run it in the foreground and read
its output directly. Then report:

- the exact set of sites the compiler named, and whether it matched the room map above;
- the final state of each rewritten comment;
- `grep -rn "SpawnOutcome\|ProgramHandleInner\|wat__kernel__ProgramHandle" --include="*.rs" src/`
  output (expected: empty, or only the retirement tombstone prose).

The orchestrator runs the floor and clippy; that is not part of this strike.

## Copy for shape

`1c098243` (`git show 1c098243`) — the non-prime IPC **type** annihilation from 24t: the same act
(delete the registration/definition, let the compiler enumerate the arms, sweep, keep the floor's
`passed` count byte-identical). `5362a8fd` — the 8-verb annihilation, same method.
