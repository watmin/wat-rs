# BRIEF — a natively-fired Session carries no alpha-memory

## The work

The native rete kernel spends **31.3% of every fire** serializing an alpha-memory that **nothing ever
reads** — both engines discard the incoming alpha and rebuild it from `facts`. Beta already gets dropped
before freeze; alpha never did. Drop it the same way, at the same two places, and re-point the one probe
that inspected it so it inspects the wat oracle instead (where alpha population is a real property).

Two one-line clears, one probe re-point, two stale comments trued, one new RED gate.

## Read in order (the rooms, and why you are being sent to each)

1. **`docs/arc/2026/06/278-rules-engine/DESIGN-STONE-alpha-is-fire-scoped.md`** — the measurement, the
   contract decision, the affirmative scope cuts. Read it first; it answers "why is this safe."

2. **`src/rete/kernel.rs:1001-1019` — `fire_once_session`.** THE EXEMPLAR. Note the shape at `:1015-1018`:
   a comment naming *why* the memory is dropped, then `wm.beta.clear();`, then `to_persistent(wm)`.
   Your alpha clear goes **beside** that beta clear, in the same style, with its own reason.

3. **`src/rete/kernel.rs:2455-2473` — the tail of `fire_fixpoint_delta`.** The second site, same shape
   (`wm.beta.clear();` at `:2462`, `to_persistent(wm)` inside the `OUT: to_persistent` phase marks).

4. **`src/rete/kernel.rs:343-360` — `to_persistent`.** Read it so you can see that it is a **pure
   converter** and confirm for yourself that the clear does *not* belong here (see STOP-1).

5. **`src/rete/kernel.rs:3148-3165` — `round_trip_fired_session`.** The test that makes (4) load-bearing:
   it asserts `to_persistent(to_transient(fired)) == fired`. Its doc comment claims the session has
   "populated alpha/beta/production memories" — beta is already false, alpha becomes false. True it.

6. **`tests/rete/probe_arc278_2b_insert_alpha.wat`** — three entries that fire via
   `(:wat::rete::fire-rules sess2)` then read `(:wat::rete::Session/alpha-memory fired)`. These are the
   assertions you re-point.

7. **`wat/rete.wat:170-188` — the `Session` record + its field-by-field doc comment.** And, 20 lines
   below it, the `Support` record: `;; EPHEMERAL — carried only in Explained; never serialized`. That is
   the established vocabulary for marking a field's lifetime in this file — mirror it.

## ★ THE ONE CONTRACT DECISION

**The clear goes at the two FIRE SITES, never inside `to_persistent`.**

`to_persistent` is a pure converter and `round_trip_fired_session` asserts its identity. A clear inside it
makes the converter lossy and that identity false — breaking a conversion test for a reason that has
nothing to do with conversion. One line at each fire path's own freeze boundary, mirroring beta.

## Implementation sketch

```rust
// in fire_once_session (kernel.rs ~:1015) and at the tail of fire_fixpoint_delta (~:2462),
// beside the existing beta clear — same shape, its own reason:

    // Drop alpha elements before freeze — alpha is fire-scoped scratch, not session state.
    // Both engines rebuild it from `facts` every fire (native: the clear at the top of this
    // function; the wat oracle: `fire-once` seeds its alpha fold empty, rete.wat:1409-1411),
    // so a frozen alpha is written for nobody and costs ~31% of fire to serialize.
    wm.alpha.clear();
    wm.beta.clear();
```

The 2b probe: change the three `(:wat::rete::fire-rules sess2)` to `(:wat::rete::fire-rules-spec sess2)`
and update the file header to say the assertions now inspect the **oracle's** alpha (native no longer
carries it; native's alpha correctness rides on the count differentials).

## The new RED gate — `tests/rete/probe_arc278_alpha_is_fire_scoped.{wat,rs}`

Copy the shape of `tests/rete/probe_arc278_native_insert_differential.{wat,rs}` (a `.wat` exposing
zero-arg `:user::` entries returning `i64`; the `.rs` calling each and asserting). Any workload that
populates alpha works — the 2b fixture's `:user::Temp` + `(> ?t 20)` shape is the smallest one.

Four assertions, and **all four are required**:

| # | entry | expected | why it is in the gate |
|---|---|---|---|
| 1 | `native-alpha-key-count` (via `fire-rules`) | `== 0` | the clear happened |
| 2 | `oracle-alpha-key-count` (via `fire-rules-spec`) | `> 0` | **the workload really populates alpha — without this, #1 passes vacuously over a no-match workload**; also proves the oracle is unmoved |
| 3 | `native-derived-count` | `== oracle-derived-count` | the RESULT is untouched |
| 4 | both derived counts | `> 0` | the differential itself is not vacuous |

Put the "what would turn this red" reasoning in the `.rs` header, as
`probe_arc278_native_insert_differential.rs:18` does.

## Blast radius

`src/rete/kernel.rs` (2 clears + 1 doc comment), `wat/rete.wat` (the `Session` doc comment ONLY — no
logic), `tests/rete/probe_arc278_2b_insert_alpha.{wat,rs}`, and the two new gate files. No corpus
migration, no codemod, no new types.

## STOP triggers (each is a rejection: ship nothing for it, report the gap)

1. **STOP-1** — if you find yourself putting the clear inside `to_persistent`, STOP. Report it. The
   contract decision above is the whole reason this brief exists.

2. **STOP-2** — `kernel.rs:3206` deliberately runs the four passes inline *instead of* calling
   `fire_once_session`, because that fn clears beta before freeze. It is intentional. If you are
   tempted to refactor it to share code, STOP and report.

3. **STOP-3** — if any existing rete test goes red other than the three `probe_arc278_2b_insert_alpha`
   entries you are re-pointing, STOP and report the test name and its assertion. A count differential
   going red means the RESULT moved, which this stone must not do.

4. **STOP-4** — if the oracle (`fire-rules-spec`) turns out NOT to populate alpha in its output (i.e.
   gate assertion #2 comes back 0), STOP and report. The whole stone rests on the oracle being the
   place alpha remains observable.

## Definition of done

- `cargo nextest run --release -E 'test(/alpha_is_fire_scoped/)'` — the new gate passes all four.
- `cargo nextest run --release -E 'binary_id(wat::rete)'` — all pass.
- `cargo nextest run --release` — the whole floor, 4208/4208.
- `cargo clippy --all-targets --release` — silent.
- Report `git diff --stat`.

Leave the tree dirty and uncommitted; the orchestrator weighs by its own re-run and commits.

## A prior result to copy for shape

`d9eadfe3` (native `insert`) is the pattern end to end: a measured claim, a differential that could go
red, a narrow diff, weighed centrally. This one is smaller — two clears and a gate.
