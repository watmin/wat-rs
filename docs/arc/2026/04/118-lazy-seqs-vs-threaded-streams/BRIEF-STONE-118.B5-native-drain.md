# BRIEF — STONE 118.B5 · the drain gets a native kernel; the wat walk becomes its oracle

`into`'s two Stream arms are the last interpreted materialization in the collection surface. Every
other arm is one native call. Measured: **529ms vs 12ms at n=200,000, ~44×.** Read
`DESIGN-STONE-118.B5-the-drain-is-the-last-interpreted-arm.md` first — it carries the decomposition
and the two traps.

## ⛔ YOU DO NOT RUN THE FLOOR

**You MAY run**, in the FOREGROUND: `cargo build --release`, `./target/release/wat --check <file>`,
a `.wat` probe or bench, and a SCOPED `cargo nextest run --release -E 'test(<pattern>)'`.
**You may NOT run** `scripts/floor.sh` or an unscoped `cargo nextest`. The orchestrator measures
centrally, once, after your tree is quiescent. Ask in your report if you want the full picture.

## Read in order

1. **`wat/seq.wat:121–158`** — `stream->vec` and `stream->pvec` as they stand. `stream->pvec` is the
   real walk (`next` + `PersistentVector/conj`, one element at a time); `stream->vec` wraps it.
   These become the **oracles**.
2. **`wat/seq.wat:160–180`** — `into`'s five clause arms. **Do not touch them.** Two already name
   `stream->vec`/`stream->pvec`; those names simply stop being interpreted.
3. **`src/collection/transform.rs:514–530`** — B6's `eval_vec_foldl`. Its guard reads
   `container.mappable() || matches!(container, StreamContainer::Stream)`, with a comment explaining
   why it does NOT widen `mappable()` — the same reasoning applies here. This is the shape to copy.
4. **`wat/rete.wat:1508`** — the oracle exemplar: `insert-all-spec` (wat) / `insert-all'` (native) /
   `insert-all` (public). *"the native kernel is the fast impl, the spec keeps it honest."*

## The strike path

**1 — rename the walks to oracles.** `stream->pvec` → `stream->pvec-spec`, `stream->vec` →
`stream->vec-spec`. Bodies unchanged, still wat. Their doc headers gain the oracle note.

**2 — the natives take the public names.** New Rust intrinsics `:wat::core::stream->pvec` and
`:wat::core::stream->vec`: realize the Stream one cell at a time and collect into the receiver's
container kind. `into`'s arms already call these names, so nothing at any call site moves.

**3 — the differential.** `native ≡ spec` on both receivers, at empty / one / many. Include a
**non-vacuity control** — perturb one side, watch it go RED, revert byte-identical, and say so.

**4 — the purity ledger.** ★ **Adding a `dispatch_keyword_head` arm makes a verb subject to
`rete::purity::completeness_gate::every_dispatched_verb_is_classified_or_disposed`**, which audits
that every dispatched verb carries a ruling in `intrinsic_meta`. This is a SEPARATE gate from
`is_pure_total` in `macros/eval.rs` and there is no link between them. The B4-0 rider discovered it
by going red; you get it for free. Both verbs are pure ∧ deterministic; neither is total (an empty
Stream is fine, but the walk can raise from the producer). Rule them beside `first`/`second`/`third`.

## The two traps, from the stone

**This drain SHOULD collect, unlike `nth`.** B4-0 forbade a draining `nth` because positional lookup
on a lazy seq *is* a walk. `into` is a **terminal** — it forces everything by contract — so one
native pass is honest here. Do not carry B4-0's rule across; the contracts differ.

**Retention is a scorecard row, not an afterthought.** B3 measured lazy pipelines flat at
**0.38 B/elem**. A native collector that holds the realized chain alive while building puts that
straight back. `wat-scripts/scratch-pad/probe-118B-dorun-retention-slope.wat` exists; drive it at
100k/200k/400k/800k and report the per-element figure.

## Blast radius

`wat/seq.wat` (the two renames + headers), the Rust file you add the natives to, `src/check.rs`
(TypeSchemes), `src/rete/purity.rs` (the rulings), and new tests. **`into`'s arms unchanged. No
other `wat/` file.**

## STOP triggers — ship nothing further, report the gap, stop

- **STOP-1** — the differential disagrees anywhere. Report input, native output, oracle output.
- **STOP-2** — retention is not flat. Report the four points. A native drain that regresses B3's
  measurement is the wrong native, not a tuning problem.
- **STOP-3** — you cannot add the natives without touching `into`'s clause arms. Report what forced it.
- **STOP-4** — a scoped `nextest` fails outside the tests you added or renamed.

## Your report

1. The differential result and its non-vacuity control (what you perturbed, the RED, the byte-identical revert).
2. The retention slope: four points, per-element bytes.
3. The bench re-run — `bench-118B5-into-stream-vs-native-concat.wat`, all three rows.
4. The purity rulings you added, verbatim.
5. What you ran; state plainly that you did not run the floor.
6. Honest deltas, line counts, wall-clock against a 60–90 minute prediction.
