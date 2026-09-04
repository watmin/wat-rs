# BRIEF — STONE: the registry can be enumerated

You are a **rider**, not the orchestrator. **Ending your turn ENDS you** — nothing will wake you.
Run every command in the FOREGROUND and block on it. You may not spawn sub-agents.

Anchor: **`/home/john/work/holon/wat-rs`**. `pwd` first. Any path containing `.claude/worktrees/`
is harness state — never operate on it. Do not commit, push, stash, or revert. Do not run the full
floor; the orchestrator runs it centrally.

Read `DESIGN-STONE-the-registry-can-be-enumerated.md` (sibling) first — especially its two ⛔
sections, which say what this stone must NOT be used for.

## The work in one paragraph

The registry can be asked about a name but not about a set. Add `(:wat::intrinsic::rows)` — a
zero-arg verb returning `(:wat::core::Vector :- [:wat::intrinsic::Row])`, one Row per registered
entry — so a wat program can run a census. The shape is not a design problem: `metadata-of`
already builds a per-row map and `(:wat::intrinsic::examples)` already returns a Vector of wat-side
records. You are composing two shipped shapes, not inventing one.

## Rooms, in order — the template, then the four sites

**Read the template first, end to end. `:wat::intrinsic::examples` is this stone already built
once**, and every site you touch has a sibling line in it:

1. **`wat/doctest.wat:13`** — `(:wat::core::defrecord :wat::intrinsic::Example [...])`. The record
   shape to mirror.
2. **`src/check.rs:17089`** — its checker scheme, `() -> (Vector :- [:wat::intrinsic::Example])`.
3. **`src/load/stdlib.rs:295`** — the load-order note; the record must load before the verb.
4. **`src/intrinsic/reflect.rs:67`** — the `#[wat_intrinsic]` and the fn that walks
   `crate::intrinsic::registry().all_entries()` and builds the Vector.

Then write the four siblings for `Row`/`rows`. Put the `Row` record wherever the load order and
the existing conventions say it belongs — `wat/runtime-meta.wat` already declares `Kind`,
`Purity`, `Determinism`, `Totality`, `ExpandTime`, `Category`, which the Row's fields reference,
so check whether that is the right home before defaulting to `doctest.wat`.

## The Row's fields

The DESIGN names them and states why each exclusion is excluded. Follow it exactly; the exclusions
are the load-bearing half. `arity` uses `-1` for `Variadic`, which is `metadata-of`'s existing
convention — do not invent a second one. `syntax` is `""` when absent, not an Option, matching
`IntrinsicEntry.syntax`'s own `&'static str` shape.

## Acceptance — three censuses, run from wat

Write a probe under `wat-scripts/scratch-pad/` (durable, loader-gated — the convention) that runs
these and prints the numbers:

```
1  total rows                                    expect 552
2  rows by kind                                  SpecialForm + Intrinsic must sum to 552
3  rows with an empty :syntax                    (any number; this is the census that had no
                                                  instrument before)
4  rows by :totality                             ⬅ THE ONE THAT MATTERS: the count of
                                                  `Totality::Partial` is what `wat/runtime-meta.wat:241`
                                                  calls "THE WORK LIST" for the totality endgame
```

**Cross-check rows 1 and 2 against the authority that already exists**, and report both numbers:

```
cargo nextest run --release -E 'test(probe_can_doc_types_reconstruct_the_checker_scheme)' --no-capture
```

That Rust test prints `total registry rows`, `Kind::SpecialForm, no scheme`, and
`Kind::Intrinsic, no scheme`. Your wat census and that test read the SAME `all_entries()`, so they
must agree on the total. ⚠ They measure different things about schemes — do not expect the
no-scheme splits to match your kind split; only the total is a shared claim.

## STOP triggers — each rejects; none permits a smaller delivery

- **STOP-1** — if your wat census's total disagrees with the Rust test's `total registry rows`,
  STOP and report both. One of the two instruments is wrong and shipping either would put a false
  number into the campaign's record.
- **STOP-2** — do not use this verb to derive, generate, or replace any of the four absence
  ledgers (`REGISTRY_MEMBERSHIP_GAP_A`/`GAP_B`, `FROZEN_CHECKER_DEBT_LEDGER`,
  `FROZEN_TYPES_UNCHECKED`). They are frozen deliberately; a ledger that computes both sides
  always agrees with itself and proves nothing. Do not touch those arrays at all.
- **STOP-3** — do not include `doc`, `prose`, `ret` description, `source`, or `examples` in the
  Row. 552 rows of prose in one value is the reason the exclusions exist.
- **STOP-4** — if the Row cannot carry one of the DESIGN's named fields (a type does not cross the
  boundary, an enum has no wat-side declaration), STOP and report which and why. Do not silently
  drop a field or substitute a String for a typed enum.

## Verification

```
cargo nextest run --release -E 'binary_id(wat)'
cargo nextest run --release -E 'binary_id(wat::reflection)'
cargo nextest run --release -E 'test(every_wat_scripts_file_loads)'
cargo clippy --release --all-targets -- -D warnings
```

Note: your probe is a `.wat` under `wat-scripts/`, so the loader gate parses and type-checks it —
that gate passing is part of the deliverable, not incidental.

## What to report

The four census numbers from your wat probe; the Rust test's `total registry rows` beside your
own; the Row record's final field list; the Summary line per scoped run; and anything that
surprised you — particularly any field that did not cross the wat boundary cleanly.
