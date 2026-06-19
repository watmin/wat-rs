# Arc 280 — ambient stdio bounded to `EdnRepresentable` (the type makes non-EDN uncompilable)

> **STATUS: STUB — banked, not started (2026-06-17).**
>
> **⛔ PRIORITY — THE #1 ARC TO OPEN THE MOMENT ARC 278 (RETE) CLOSES.** Builder, 2026-06-19:
> *"280 is our highest priority after we wrap up rete."* Sequence: finish 278 (P4b delta → P4c retract/TM
> → P5 wire+bench = rete closes) → **then open this, first.** Re-confirmed live when the deep-cascade perf
> harness emitted EDN via `println` (∀T) and the builder pushed to make the bound structural: *"how do we
> choke the stdio services to make you pass an edn-representable?… basically all scalars are EDN (strings,
> maps, vecs, ints, floats, etc.)."* — i.e. the bound admits every scalar/collection/record and excludes
> only the non-EDN (`Fn`, opaque handles).
>
> Surfaced originally by the builder mid-arc-277:
> *"get a stub arc wired up that makes the ambient stdio only accept EdnRepresentable inputs and return
> them."* The intent is already DOCUMENTED ("EDN-only") but NOT type-enforced — close that gap by
> putting an `EdnRepresentable` bound on the stdio ops' type params, so a non-representable value is
> **uncompilable**, not a runtime surprise.

## The gap (grounded against HEAD)

The ambient stdio surface — the canonical post-arc-109 stdio (`check/error.rs:505`: "EDN-only — any
value EDN-encodes; no manual string formatting"):

- `:wat::kernel::println` — impl `src/services/verbs.rs:47`; dispatch `src/runtime.rs:4553`.
- `:wat::kernel::eprintln` — impl `src/services/verbs.rs:101`; dispatch `src/runtime.rs:4554`.
- `:wat::kernel::readln -> :T` — impl `src/services/verbs.rs:173`; dispatch `src/runtime.rs:4555`;
  the call-site `-> :T` arrow drives the decode type (`check.rs:4031` readln arm).

**The schemes are UNBOUNDED** (`src/check.rs:16958-16977`):
```rust
for op in [":wat::kernel::println", ":wat::kernel::eprintln"] {
    env.register(op, TypeScheme { type_params: ["T"], params: [t_var()], ret: unit_ty(), … });
}
env.register(":wat::kernel::readln", TypeScheme { type_params: ["T"], params: [], ret: t_var(), … });
```
`T` is a free, **unconstrained** type variable. The type system accepts *any* value for `println`/
`eprintln` and produces *any* type for `readln`. The "EDN-only" promise lives only in a doc comment and
the runtime encoder — exactly the **contract-not-encoding** failure: the honesty is a convention, not a
structural guarantee. A value that cannot round-trip as EDN either fails at runtime or encodes lossily;
the type never says no. This is the [[feedback_no_magic_that_lets_llm_fake_correctness]] line — the
wrong input should be *uncompilable*, not "it happens to work."

## The fix (sketch — to design when opened)

Put an `EdnRepresentable` bound on the stdio type params:
- `println`/`eprintln` : `<T: EdnRepresentable> (T) -> nil`
- `readln` : `<T: EdnRepresentable> () -> T` (the `-> :T` arrow still drives the decode; the bound
  constrains which `:T` is legal).

**Mechanism — already in the substrate (ground it, don't rebuild):**
- The Rust trait `EdnRepresentable` exists: `src/comms/mod.rs:102` (the comms wire bound; `HolonRepresentable`
  is its supertrait at `:134`). It is the exact "round-trips as EDN" contract.
- wat-level protocol bounds shipped: **arc 232** (`defprotocol`) + **arc 267** (parametric protocol
  bounds) + **arc 246** (generic protocol methods). So a bound on a type param is expressible in the
  wat type language today.
- **The open question for the DESIGN:** is there (or should there be) a wat-facing `:wat::*::EdnRepresentable`
  *protocol* that the checker maps to the Rust trait, OR does the check-layer carry the bound directly
  on these built-in schemes (a `TypeScheme` bound field)? Ground which path the arc-267 bound machinery
  takes for a BUILT-IN op (vs a user `defprotocol`) before pinning the contract. Probe-first: a
  `println` of a deliberately non-EDN value must go RED at CHECK time (not runtime).

## Out of scope / boundaries (to affirm when opened)

- **NOT** the Rust-side ban on native `println!`/`eprintln!` in `src/` — that is task #201 / the arc-109
  "kill-std" enforcement (a different axis: stop Rust code bypassing the EDN stdio). This arc is the
  **wat-facing type bound**. Note the sibling so the two aren't conflated.
- **NOT** a new encoder — the runtime already EDN-encodes; this arc only makes the *type* demand it.
- Holon values: `HolonRepresentable: EdnRepresentable`, so holons satisfy the bound for free — confirm,
  don't special-case.

## Four questions (sketch, to weigh when it opens)

- **Obvious?** YES — "stdio speaks EDN" is the post-109 doctrine; the bound just says in types what the
  docs already say in prose.
- **Simple?** Likely YES — a bound on three existing schemes; no new runtime. Confirm the bound-on-a-
  built-in path is as clean as a user `defprotocol` bound (the open question above).
- **Honest?** This is the WHOLE POINT — turns a doc-comment promise + runtime encode into a structural
  guarantee; the non-EDN value becomes uncompilable.
- **Good UX?** YES — the error fires at check time, naming the un-representable type, instead of a
  runtime encode surprise; every ordinary value (scalars, records, holons, collections of them) already
  satisfies it.

## Blast radius (estimate)

- `src/check.rs:16958-16977` — add the `EdnRepresentable` bound to the three schemes.
- Possibly a wat-facing `EdnRepresentable` protocol declaration (home TBD — `wat/kernel/` or a comms-
  adjacent wat file) if the bound must reference a wat protocol rather than a check-internal marker.
- A RED probe (`println`/`readln` of a non-EDN type → check error) + deftests.
- Migration: audit existing `println`/`readln` call sites — all current values are EDN-representable, so
  the bound should be additive (zero call-site churn expected; confirm with the full test suite).
