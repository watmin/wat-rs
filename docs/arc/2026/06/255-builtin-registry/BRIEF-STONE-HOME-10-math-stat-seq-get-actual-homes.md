# STONE HOME-10 — math, stat, seq get ACTUAL homes (finishing what HOME-9 left)

DRAWN 2026-08-27 against `29f350365`.
**PRIOR ART:** `git log -1 fb0cdb192` (HOME-8 — pure re-registration, nothing renamed; the shape
this stone copies) and `git log -1 29f350365` (HOME-9, which this stone completes).

## ⛔ WHY THIS STONE EXISTS — HOME-9 RENAMED BUT DID NOT HOME

HOME-9 killed `:wat::std::` and the verbs now answer under `:wat::math::` / `:wat::stat::` /
`:wat::seq::`. But **`src/intrinsic/{math,stat,seq}` do not exist.** All twelve are still dispatch
arms in `runtime.rs`.

**That is the orchestrator's defect, not the rider's.** HOME-9's brief said "PHASE 1 register
`:wat::{math,stat,seq}::*`" while every one of its seven acceptance rows measured **naming** — the
new spelling runs, the old is refused, `:wat::std::` is gone. **Not one row asked whether a home
existed.** The rider satisfied every row as written. An acceptance row that does not measure the
deliverable is how a stone ships half-done and green — so this brief carries **row 0** below, and
every future home brief should.

## The move — pure re-registration, nothing renamed

```
:wat::math::{ln exp sqrt sin cos pi}     ->  src/intrinsic/math.rs    (7 verbs)
:wat::stat::{mean variance stddev}       ->  src/intrinsic/stat.rs    (3 verbs)
:wat::seq::{zip window remove-at}        ->  src/intrinsic/seq.rs     (3 verbs)
```

Names are final. The corpus is already migrated. The retirement rows already fire. **No codemod, no
RetirementEntry rows, no dual-spelling window, no `.wat` file to touch.** Writing one is **STOP-4**.

## ⛔ THE ONE CONTRACT DECISION — THREE FILES, NOT THREE DIRECTORIES

HOME-8 split holon into `src/holon/` (algebra) + `src/intrinsic/holon/` (interface) because it had
**1,169 lines of VSA algebra touching no `env`/`sym`**. **That does not apply here.** Measured:

```
eval_math_unary     arity-check -> unwrap -> f(x) -> rewrap, where f is `f64::sqrt` FROM RUST STD
eval_stat_mean 44   variance 51   stddev 36     "real arithmetic" = `let mut sum = 0.0; sum += x`
eval_seq_zip   39   window   36   remove-at 38  thin over `require_seqable_vec`
```

Compare the existing shim-only homes: `uuid` 22 body-lines/verb, `bytes` 31, `char` 56. **These
twelve sit squarely in that band.** There is nothing to put in a `src/math/` except the shim itself.

⚠ **The orchestrator initially claimed math/stat were the two-layer case and was wrong** — it
mistook "has a `for` loop" for "has an implementation". The two-layer doctrine triggers on real
algebra worth naming, not on line count. Do not build `src/math/`, `src/stat/`, or `src/seq/`.
**STOP-1.**

Copy `src/intrinsic/rational.rs` (214 lines / 7 verbs) or `src/intrinsic/hashset.rs` (147/4) for shape.

## Rooms — verified against `29f350365`

```
src/runtime.rs:6393-6398     the six math arms          (:6398 `pi` -> eval_math_pi, arity 0)
src/runtime.rs:6399-6401     the three stat arms
src/runtime.rs:6122-6130     the three seq arms
src/runtime.rs               eval_math_unary / eval_math_pi / eval_stat_{mean,variance,stddev}
                             ⚠ eval_math_unary's docstring enumerates FOUR callers; there are FIVE
                               now that `log` is deleted. Fix it while you are in there.
src/collection/transform.rs  eval_seq_{zip,window,remove_at} + require_seqable_vec
src/intrinsic/rational.rs    the shim shape to copy
src/intrinsic/mod.rs         `mod math; mod stat; mod seq;`
src/macros/eval.rs           11 is_pure_total entries — measure whether they survive registration
src/rete/purity.rs           15 ledger entries — the KNOWN_UNREVIEWED gate is exact BOTH ways
src/check.rs                 6 signature rows
```

## Provenance

Stone G (`38f51c9fc`) made `NativeHandler` return `TrackedValue`. **None of these twelve is a
producer** — they compute scalars and collections from their arguments, they do not mint tracked
values. Returning a bare `Value` (wrapped `Provenance::Unknown` by the shim) is correct and is the
behaviour-preserving choice. If you find one that DOES stamp provenance, that is a finding — report
it. **Do not upgrade any of them speculatively.**

## STOP triggers — each REJECTS

1. **STOP-1 — you would create `src/math/`, `src/stat/`, or `src/seq/`.** Shim-only; three files.
2. **STOP-2 — you would change a verb's behaviour.** Registration only. If a handler must change to
   fit `#[wat_intrinsic]`, report the obstruction rather than adapting the semantics.
3. **STOP-3 — a registry consistency test fires that you cannot satisfy honestly** (`@example`
   runnability, `@ret` vs the checker's TypeScheme, the purity census). HOME-8 made three of these
   fire for real; expect them, and satisfy them with truth, not with a weakened assertion.
4. **STOP-4 — you would write a codemod, a RetirementEntry row, or touch a `.wat` corpus file.**
5. **STOP-5 — a room's line number does not hold.**

## Acceptance

```bash
# 0. ★ THE ROW HOME-9 OMITTED — the home must EXIST. This is the deliverable.
ls src/intrinsic/math.rs src/intrinsic/stat.rs src/intrinsic/seq.rs
grep -c '#\[wat_intrinsic(' src/intrinsic/math.rs src/intrinsic/stat.rs src/intrinsic/seq.rs   # 7 3 3
grep -cE '":wat::(math|stat|seq)::[^"]*"\s*=>' src/runtime.rs                                   # 0

# 1. every verb still RUNS, same answers as before — a scratch-pad probe asserting each.
#    (:wat::math::sqrt 16.0) -> 4.0 ; (:wat::seq::zip (List 1 2 3) (List 4 5 6)) -> [[1 4] [2 5] [3 6]]

# 2. ★ SEQABLE SURVIVES THE CARVE — zip/window/remove-at still accept a LIST, not just a Vector.
#    HOME-9 won this; prove the registration did not quietly lose it. Paste List AND Vector per verb.

# 3. metadata-of answers for one verb from each new home.

# 4. cargo build --release --all-targets
```

## Report back with

Row 0's three outputs verbatim. Row 2's six outcomes (List + Vector × three verbs). Every registry
consistency test that fired and how you satisfied it. What `is_pure_total` and the purity ledger
needed. `runtime.rs`'s line count before and after. Anything this brief got wrong; what you did NOT
do, and why.
