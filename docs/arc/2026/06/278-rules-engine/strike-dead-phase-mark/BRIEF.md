# BRIEF — make an unemitted phase mark a RED instead of a 0.00 ms row

One cost table reads a census row the engine never emits, gets `0` from an `unwrap_or(0)`, prints it
as a measurement and subtracts it. Restate that table to what is true, and add a lint so the next
dead mark cannot reach a table at all.

## Read in order

1. `src/rete/kernel/fire/delta.rs:270-276` — **the ground truth.** `phase_start()`, two `FxHashSet`
   allocations, then BOTH `phase_end`s. The two marks are coextensive and wrap allocation only.
   Everything else follows from this.
2. `src/rete/kernel/fire/pass/alpha.rs:58` and `src/rete/kernel/fire/pass/production.rs:114` — the
   only two `seen_insert` call sites, inside the `alpha` and `production` phases. This is where the
   insert cost already is.
3. `src/rete/kernel/tests/accum_cost.rs:1528-1636` — the test `accum_seen_fire_context_split`. The
   dead read is `:1603`; the false rows are `insert` and `in-fire insert − S` in the `format!`.
4. `src/rete/kernel/tests/accum_cost.rs:100-118` — `REQUIRED_PHASES`, the existing subset assertion.
   Note `setup:seen:insert` is absent from it: the list was already right.
5. `tests/lint/rete_citation_resolves.rs` — **the shape to copy** for the new lint: it reads source,
   classifies code vs prose, resolves names against a set gathered from the tree, and fails with a
   message naming each unresolved item and the fix.
6. `tests/lint/` — how a lint test is registered and what its harness looks like.

## The two pieces

**Piece 1 — the lint.** A new `tests/lint/<name>.rs` that:
- gathers the engine's emitted names: every `phase_end("...")` and `census_count("...")` /
  `census_count_n("...")` string literal under `src/`, excluding `src/rete/kernel/tests/`;
- gathers the names the cost tests READ: every `of("...")`-style literal and every string literal
  containing a census tree glyph (`├`, `│`, `└`) in `src/rete/kernel/tests/*.rs`;
- fails naming every read-but-never-emitted name with its `file:line`.

At HEAD this must go **RED on exactly one name** — `"  │  setup:seen:insert"` at
`accum_cost.rs:1603`. That RED is the proof the lint works; capture it before fixing piece 2.

**Piece 2 — the table.** Remove the `insert` row and the `in-fire insert − S` row, drop `fire_ins`
and its dead read, and state in the table (and in the test's doc comment at `:1528`) what is true:
`setup:seen` is coextensive with `setup:seen:alloc` and covers allocation only; the insert cost is
inside `alpha` and `production` via `seen_insert`. Keep `in-fire seen − S`.

## Blast radius

`src/rete/kernel/tests/accum_cost.rs` and one new file under `tests/lint/`. **Nothing under
`src/rete/kernel/fire/`** — the engine is correct; do not add a mark.

## STOP triggers

1. **If the lint finds names beyond `setup:seen:insert`**, stop and report the list. The corpus was
   measured at exactly one; more means the sweep here differs from mine and we should reconcile
   before anything is deleted.
2. **If you find yourself adding a `phase_end` to the engine**, stop. The insert work is already
   inside two timed phases and a new mark would double-count it.
3. **If removing `fire_ins` makes another row change value**, stop and report which — nothing else
   should depend on it.
4. **If `setup:seen` and `setup:seen:alloc` turn out NOT to be coextensive** when you read
   `delta.rs:270-276`, stop: the design's central claim is then wrong.

## Mutation proofs — run both, report both

1. **Restore the dead read** (put `setup:seen:insert` back) → the new lint must go RED naming it.
2. **Point an EXISTING mark's name at a typo** (e.g. read `"  ├ setup:see"` somewhere in a cost
   test) → the lint must go RED naming that too. Proves it resolves names generally, not that it
   special-cases one string. Restore after each.

## Report

- The lint's RED at HEAD, verbatim, before the fix.
- The table before and after, verbatim.
- Both mutation results.
- Your scoped nextest Summary lines including `binary_id(wat::lint)`.
- Anywhere this brief was thin, wrong, or pointed you at the wrong line. Be blunt — that section is
  the most useful thing you return.
