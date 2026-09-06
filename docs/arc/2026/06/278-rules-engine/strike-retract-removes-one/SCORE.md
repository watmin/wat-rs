# SCORE — `retract` drops one; the redrawn axis is green

`remove-one` replaces `remove-every-equal`. The axis duplicates only F(0). Pre-cure wat `[1 2]` vs Clara `[0 1 2]`. Post-cure both `[0 1 2]`. Floor GREEN. No other value moved.

## Scorecard

| # | result |
|---|---|
| 1 ★ axis GREEN | **HOLD.** Raw `#grid/Verdict` below. `:accuracy :match :oracle-accuracy :match :port-accuracy :match`. Three-way: `clara=3 native=3 oracle=3 ALL THREE MATCH`. |
| 2 ★ before/after | **HOLD.** Pre-cure (redrawn seed, old door): wat/oracle `[1 2]`, Clara `[0 1 2]`. Post-cure: both `[0 1 2]`. |
| 3 ★ nothing else moved | **HOLD.** Floor 5465/5465, same count as strike 1. Port check `want_n` 2→3 is the axis's own bump. TMS fuzzer comment-only. |
| 4 ★ old door GONE | **HOLD.** `grep -rn remove-every-equal` hits only `docs/arc/.../strike-*`. Wrap file now names `remove-one`. |
| 5 ★ floor | **HOLD.** `Summary [ 460.785s] 5465 tests run: 5465 passed (3 slow), 21 skipped`. `.floor/2026-09-06T09-28-20Z/`. Count did not move vs strike 1 (the 5 lint tests are already there). |
| 6 `want_n` | **HOLD.** `CORRECTNESS_SIZES` 2 → 3. Derivation: duplicate only F(0); remove-one leaves one F(0); Out is `[0, items)`. |
| 7 order preserved | **HOLD.** Fold walks items in order; skips the first equal; conjs every other fact. Absent ⇒ unchanged. |
| 8 fuzzer comment | **HOLD.** `final-facts` sentence gone. Diff is the header paragraph only. |
| 9 clippy | **HOLD.** `cargo clippy --all-targets --release -- -D warnings` rc=0. |

★ load-bearing. **Row 1 is the cure's acceptance test; row 3 is what says the cure is precise.**

## Row 1 — quoted `#grid/Verdict`

```
#grid/Verdict {:axis "retract-multiplicity" :size [3] :accuracy :match :oracle-accuracy :match :port-accuracy :match :runs 1 :ratio 11.3151 :min 11.3151 :max 11.3151 :wat-ns 30932 :wat-ns-min 30932 :wat-ns-max 30932 :clara-ns 349998 :winner :us :wat-wall-ms 389 :clara-wall-ms 3062 :wall-ratio 7.8715 :wall-winner :us :fire-share-pct 0.01}
```

`check-grid-three-way.sh retract-multiplicity` (exit 0):

```
retract-multiplicity clara=3    native=3    oracle=3     ALL THREE MATCH
grid-three-way: 1 axis/axes, all AGREED — Clara == oracle == native (4s)
```

## Row 2 — pre-cure, redrawn seed, old door still in

```
#grid/Result … :derived #wat.core/PersistentVector [1 2] … :oracle-derived #wat.core/PersistentVector [1 2]
```

```
[retract-multiplicity] ⛔ oracle != clara  =>  THE SPEC IS WRONG (size [3])
      oracle (2 elems): 1 2
      clara (3 elems): 0 1 2
      only in clara: 0
[retract-multiplicity] ⛔ native != clara  =>  THE FAST PATH IS WRONG (size [3])
```

No `oracle != native` line. Port held. Sole witness is key 0.

## `remove-one`

`FactBagDrop { items, dropped }` — StratifyAcc shape. Keep `f` when `dropped OR f ≠ fact`; otherwise skip once and set `dropped`. First-not-last, order-preserving. `retract` calls it. Docstring now describes the symmetry rather than promising it.

## Landing

Axis seed + cure + `want_n` bump together. No `gen-` twin (static `.clj`). Throwaway `gen-` used only to mint the `#grid/Verdict` line, then deleted. Do not commit unless asked; when committed they are one commit.
