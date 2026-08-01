# BRIEF v2 — `fire-rules` must not return an alpha-memory the oracle does not

> **v1 was wrong and the STOP caught it.** v1 told you to re-point the 2b probe at `fire-rules-spec`,
> claiming the oracle populates alpha there. It does not — `fire-stratified` (`rete.wat:1817-1820`)
> returns an empty alpha map. Your STOP-4 was correct and the revert was correct. v2 changes the target
> and **narrows the cut to one site.** Everything else you built was on the right track.

## The work

Native `fire-rules` returns a populated `alpha-memory`; the wat oracle `fire-rules-spec` returns an empty
one. They **disagree today**. Serializing that alpha costs **31.3% of every fire**, and nothing reads it —
both engines discard the incoming alpha and rebuild from `facts`. Drop it at the one fire site that has it,
which simultaneously closes the divergence and removes the cost.

One clear, one probe re-point, two stale comments, one new five-assertion gate.

## Read in order

1. **`docs/arc/2026/06/278-rules-engine/DESIGN-STONE-alpha-is-fire-scoped.md`** (v2) — read the table at
   the top; it is the whole justification, and it is different from v1.

2. **`src/rete/kernel.rs:2455-2473` — the tail of `fire_fixpoint_delta`.** THE ONE SITE. `wm.beta.clear()`
   at `:2462` is the exemplar: a comment naming *why*, then the clear, then `to_persistent(wm)` inside the
   `OUT: to_persistent` phase marks. Your alpha clear goes beside it, same style, its own reason.

3. **`src/rete/kernel.rs:1001-1019` — `fire_once_session`.** Read it so you can see the shape you are
   copying — and note that you are **deliberately NOT changing it** (see STOP-1).

4. **`src/rete/kernel.rs:3148-3165` — `round_trip_fired_session`.** Its doc comment claims the session has
   "populated alpha/beta/production memories". Beta is already false; alpha becomes false. True it.

5. **`tests/rete/probe_arc278_2b_insert_alpha.wat`** — three entries firing via
   `(:wat::rete::fire-rules sess2)`. Re-point to **`(:wat::rete::fire-once' sess2)`**.

6. **`wat/rete.wat:170-188` — the `Session` record's field-by-field doc comment**, and ~20 lines below it
   the `Support` record's `;; EPHEMERAL — carried only in Explained; never serialized`. That is this
   file's established vocabulary for a field's lifetime — mirror it for alpha/beta.

## ★ THE ONE CONTRACT DECISION

**Clear alpha in `fire_fixpoint_delta` ONLY** — not in `fire_once_session`, never inside `to_persistent`.

Leaving `fire_once_session` populated is deliberate: native `fire-once'` then still matches the oracle's
`fire-once`, which genuinely fills alpha (`rete.wat:1462`). That alignment is what gives the 2b probe a
truthful home. Narrowing the cut is the point, not an omission.

## Implementation sketch

```rust
// fire_fixpoint_delta, kernel.rs ~:2462, beside the existing beta clear:

    // Drop alpha elements before freeze — alpha is fire-scoped scratch, not session state.
    // The wat oracle's fire-rules-spec returns an EMPTY alpha (fire-stratified, rete.wat:1817),
    // so carrying one here is a divergence as well as a cost: both engines rebuild alpha from
    // `facts` every fire and never read a frozen one. It was ~31% of fire to serialize.
    // (fire_once_session deliberately keeps its alpha — it mirrors the oracle's fire-once,
    //  which does populate it.)
    wm.alpha.clear();
    wm.beta.clear();
```

The 2b probe: three `(:wat::rete::fire-rules sess2)` → `(:wat::rete::fire-once' sess2)`; update the `.wat`
and `.rs` headers to say these assert the **single-pass** alpha activation, which is what stone 2b is
about, and that the fixpoint verb no longer carries alpha.

## The new RED gate — `tests/rete/probe_arc278_alpha_is_fire_scoped.{wat,rs}`

Copy the shape of `tests/rete/probe_arc278_native_insert_differential.{wat,rs}`. The 2b fixture's
`:user::Temp` + `(> ?t 20)` shape is the smallest workload that works.

| # | entry | expected | why it is in the gate |
|---|---|---|---|
| 1 | `native-alpha-key-count` (via `fire-rules`) | `== 0` | the clear happened |
| 2 | `oracle-alpha-key-count` (via `fire-rules-spec`) | `== 0` | the oracle's state, asserted not assumed |
| 3 | 1 `==` 2 | equal | the divergence is closed |
| 4 | `single-pass-alpha-key-count` (via `fire-once'`) | `> 0` | **the anchor — without it, 1–3 pass vacuously over a workload that matches nothing** |
| 5 | `native-derived-count` `==` `oracle-derived-count`, both `> 0` | equal, non-zero | the RESULT is untouched |

Put the "what would turn this red" reasoning in the `.rs` header, as
`probe_arc278_native_insert_differential.rs:18` does.

## Blast radius

`src/rete/kernel.rs` (one clear + one doc comment), `wat/rete.wat` (the `Session` doc comment ONLY — no
logic), `tests/rete/probe_arc278_2b_insert_alpha.{wat,rs}`, the two new gate files.

## STOP triggers (each is a rejection: ship nothing for it, report the gap)

1. **STOP-1** — do not clear alpha in `fire_once_session`, and do not put any clear inside
   `to_persistent`. If either looks necessary, STOP and report.
2. **STOP-2** — `kernel.rs:3206` avoids `fire_once_session` deliberately (it clears beta before freeze).
   Do not refactor it.
3. **STOP-3** — if any rete test goes red other than the three `probe_arc278_2b_insert_alpha` entries you
   re-point, STOP and report the test name and its assertion. A count differential going red means the
   RESULT moved, which this stone must not do.
4. **STOP-4** — if gate assertion #4 (`fire-once'` alpha `> 0`) comes back 0, STOP and report: the gate
   would be vacuous and the workload is wrong.

## Definition of done

- `cargo nextest run --release -E 'test(/alpha_is_fire_scoped/)'` — all five assertions pass.
- `cargo nextest run --release -E 'test(/2b_insert_alpha/)'` — green via `fire-once'`.
- `cargo nextest run --release -E 'binary_id(wat::rete)'` — all pass.
- `cargo nextest run --release` — the whole floor.
- `cargo clippy --all-targets --release` — silent.
- Report `git diff --stat`.

Leave the tree dirty and uncommitted. The orchestrator weighs by its own re-run and commits.
