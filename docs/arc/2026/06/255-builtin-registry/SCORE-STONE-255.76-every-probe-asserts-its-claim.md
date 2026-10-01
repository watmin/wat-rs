# SCORE — STONE 255.76: every probe asserts its claim — the thirty-nine that only print

**Executor: a Sonnet subagent.** Branch `main`, drawn at `561b8d46b` (the draw commit; code tree
identical to `519035e69`/`82af2f167`). Not pushed.

## 1. The ruling (E3)

"A probe asserts its own claim in wat; the floor runs every probe and requires exit 0. Exit 0
must mean 'the claim held.'" 39 probes named by the orchestrator's census printed only, never
asserting. Each gets exactly one disposition: **assert** (the claim is a computed value — pin it
with `:wat::test::assert-eq`/`assert-true` or a `match` whose wrong arms call
`assertion-failed!`), **check-claim** (the claim is that the program type-checks/starts up — exit
0 already IS the claim; header gets a `;; CLAIM (exit 0):` line, no ceremonial assertion), or
**retire** (`git rm` — the claim is superseded).

## 2. Per-probe disposition

| # | probe | claim (one line) | disposition | assertion / reason |
|---|---|---|---|---|
| 1 | `arc-054/probe-054-fn-idempotency.wat` | re-declaring a record standalone AND inside a defsurface's `:messages` is a no-op (byte-equivalent, arc-054) at BOTH the type and the constructor-fn registration | **check-claim** | no computed value past "did loading raise" — a load-time property, same class as a `--check` claim; exit 0 is the proof |
| 2 | `arc-170/probe-fnforms-keyword.wat` | `fn-forms` on a runtime-computed keyword (not a direct fn value) resolves to the registered fn and reifies | **assert** | `(length forms) == 2` |
| 3 | `arc-170/probe-reason-downcast.wat` | an open-surface value down-narrows to its concrete record via a defclause dispatch | **assert** | `code == 2067` (mutation-proved, see §4) |
| 4 | `arc-170/probe-c2-narrow-2param-plainrecord.wat` | a plain record satisfies a 2-param surface and dispatches | **assert** | `ok == 42` |
| 5 | `arc-170/probe-fnforms-shape.wat` | fn-forms shape reconnaissance: arg/return-type keyword placement | **assert** | `(length forms) == 2`; `return-type-of == "wat::core::i64"` |
| 6 | `arc-170/probe-recordtype-fixed.wat` | a user defrecord ships to a process-bracket child WITH its fields | **assert** | `result == [2 4 6]` |
| 7 | `arc-170/probe-c2-narrow-multisurface.wat` | a record satisfying BOTH a mono and a parametric surface still dispatches the parametric one | **assert** | `ok == 42` |
| 8 | `arc-170/probe-freeze-narrow.wat` | a plain top-level defrecord (no defservice) doesn't break the process bracket | **assert** | `pr == [2 4 6]` |
| 9 | `arc-170/probe-s2-runner-count.wat` | ThreadOpts/ProcessOpts carry a `runner-count` field (default cpu-count), read directly AND via the tier-blind Locus reader | **assert** | `n8==8`, `ndef==cpu-count`, `n4==4`, `lproc==8`, `lthr==cpu-count` (machine-dependent values asserted against the SAME builtin they default from, never a hardcoded digit) |
| 10 | `arc-170/probe-c2-nonparam-baseline.wat` | baseline: non-parametric surface + extend-type + method-call mechanics work | **assert** | `ok == 42` |
| 11 | `arc-170/probe-kwargs-struct.wat` | the kwargs-struct flip lets an impure fn field (`& [...]`) through | **assert** | `result == 42` |
| 12 | `arc-170/probe-s3-bracket-loci.wat` | the SAME work farmed over a thread pool and a process pool, loci-agnostic | **assert** | `tr == pr == [2 4 6 8 10]` |
| 13 | `arc-170/probe-cap2-isolate.wat` | a process bracket pool-map runs correctly alongside a live defservice on the same locus | **assert** | `pr == [2 4 6 8 10]` |
| 14 | `arc-170/probe-locus1-generic-surface-method.wat` | a generic method member on an aggregate defsurface, satisfied via extend-type, type-checks AND dispatches | **assert** | `result == 5` (mutation-proved, see §4) |
| 15 | `arc-170/probe-s3a-select-peer.wat` | `select'` accepts the abstract `Peer'` element type (type-checker gap, S3a) | **check-claim** | `:probe::sel` is never called by `main` (only bound/discarded internally) — the claim is entirely at freeze time; exit 0 is the proof |
| 16 | `arc-170/probe-cap2-process-grantpath.wat` | the process grant path (peer-pid Some → foldl over grantables) runs end-to-end with an empty grantable vector | **assert** | `pr == tr == [2 4 6 8 10]` |
| 17 | `arc-170/probe-locus2-abstract-surface-dispatch.wat` | an aggregate surface's method, called through an ABSTRACT surface-typed param, dispatches to the concrete extend-type impl | **assert** | `result == 42` (mutation-proved, see §4) |
| 18 | `arc-170/probe-s3b-extract.wat` | extraction of the concrete arg/return type keywords off fn-forms output | **assert** | `arg-kind=="keyword"`, `arg-name==":wat::core::i64"`, `ret-kind=="keyword"`, `ret-name==":wat::core::i64"` (data via `ast-kind`/`ast-name` accessors, never the rendered println text) |
| 19 | `arc-170/probe-cap2-spawnrunner-pid.wat` | `peer-pid` on the bracket's actual process-spawned worker peer vs. a thread-spawned one | **assert** | shape via `match`: process → `Option.Some` of a positive i64 (`assert-true`); thread → `Option.None` (a pid can't be pinned by digits) |
| 20 | `arc-170/probe-m1-apply-generic.wat` | a body can APPLY a value whose type is a generic param `W` | **assert** | `d == 10` (mutation-proved, see §4) |
| 21 | `arc-170/probe-strikeB-fields.wat` | struct-field reflection: `field-names-of`/`field-types-of` | **assert** | `names == [:kv :n]` exactly (keywords are Equatable); `(length types) == 2` — the field TYPES are structured type descriptors, same shape-not-digits exception the brief grants a pid/address, rather than reconstructing a byte-identical Type literal |
| 22 | `arc-170/probe-compound-upcast.wat` | Tuple/Map constructors up-cast components against a known expected type at construction | **check-claim** | `pr`/`mp` never inspected past being built; `tests/process/probe_arc255_74_compound_upcast_runs.rs` already drives `--check` + run and asserts exit 0 — exit 0 here is the proof |
| 23 | `arc-170/probe-m1-arity.wat` | a generic-`W` param accepts a 2-arg fn | **assert** | `g == 7` (the fn's own constant; proves the param ACCEPTS the arg, never calls it) (mutation-proved, see §4) |
| 24 | `arc-170/probe-thread-only.wat` | bracket::map over a thread pool of 2 | **assert** | `tr == [2 4 6 8 10]` |
| 25 | `arc-170/probe-defclause-open-arg.wat` | a defclause with ONLY concrete clauses, passed an open-surface-typed value, narrows and dispatches | **assert** | `d == "sqlite 2067"` |
| 26 | `arc-170/probe-m1-erase-only.wat` | a concrete `Address'` erased to bare `Address'` via ann-form, stored, sent as a bare-D `PoolMsg::Setup` | **assert** | shape via `match`: `msg` must be `PoolMsg.Setup`, not `.Work` (an Address' can't be pinned by digits) |
| 27 | `arc-170/probe-trivial.wat` | smoke baseline: minimal defn + println runs | **check-claim** | no computed value past the literal "ok" sentinel; exit 0 is the proof |
| 28 | `arc-170/probe-defclause-real-shape.wat` | the REAL contract shape (agnostic enum field typed `Reason`) still dispatches through the concrete-clause defclause | **assert** | `d == "sqlite 2067"` |
| 29 | `arc-170/probe-m1-service-pid.wat` | a started service's pid is readable via `peer-pid` on `Handle/handle` | **assert** | shape via `match`: `Option.Some` of a positive i64 (`assert-true`) |
| 30 | `arc-170/probe-type-splice.wat` | does a generic fn substitute its type param into a `forms` quote at a concrete call | **assert** | measured NO: `arg-ty`/`ret-ty` ast-name both `":I"` (un-substituted) — asserted structurally via `ast-kind`/`ast-name`, never the edn/write string |
| 31 | `arc-170/probe-deporder.wat` | `:wat::deporder::verify-stdlib` finds zero order violations | **assert** | `(length violations) == 0` |
| 32 | `arc-170/probe-mod-in-macro.wat` | `i64::mod` on the macro pure-total allow-list lets a macro branch parity at expand time | **assert** | `(list-parity 1 2 3 4) == "even"`; `(list-parity 1 2 3) == "odd"` — the macro's own literal return values, not a stand-in |
| 33 | `arc-170/root-gapA.wat` | fn-forms directly on the kwargs `$impl` vs. a hand-written same-shape fn — do they diverge | **assert** | measured NO divergence: `(length hf) == (length kf) == 5`; last-form `ast-name` is `":test::hand"`/`":test::work"` respectively |
| 34 | `arc-170/probe-edn.wat` | `edn/write` renders a `Vector[i64]` as `"[2 4 6]"` | **assert** | `rendered == "[2 4 6]"` — the function under test IS the string renderer, so this is the data under test, not a stand-in |
| 35 | `arc-170/probe-nested-vector-of-tuples.wat` | a Handle inside a Tuple inside a Vector literal recursively up-casts | **check-claim** | `hs` never inspected past being built; exit 0 is the proof |
| 36 | `arc-293/s3-probe-struct-satisfies-nature-struct.wat` | after the 4th Nature variant (Peer), an ordinary struct still satisfies a `:nature :Struct` surface | **assert** | `r == 42` |
| 37 | `arc-170/probe-edn2.wat` | byte-identical duplicate of `probe-edn.wat` (same literal program, no distinguishing claim or doc) | **retire** (`git rm`) | superseded by `probe-edn.wat`, which carries the only doc reference between the two (`BRIEF-STONE-1c-0a-five-call-sites-name-nothing.md`, `SCORE-0b-batch-1.md`) |
| 38 | `arc-170/probe-process-only.wat` | bracket::map over a process pool of 2 | **assert** | `pr == [2 4 6 8 10]` |
| 39 | `arc-293/s4c-carrier-probe.wat` | a user defsurface's generated `surface-forms` accessor resolves (AMEND-255.4's "reaches beyond the builtin families" finding) | **assert** | `(length forms) == 1` |

The five with no doc (`probe-fnforms-keyword`, `probe-trivial`, `probe-type-splice`,
`probe-deporder`, `probe-edn2`) got their claim from their own header/body text, per the brief:
#2, #27, #30, #31 kept (assert/check-claim); #37 (`probe-edn2`) retired as a byte-identical
duplicate carrying no claim of its own.

**No STOP fired.** Every recovered claim held under its own assertion; no probe's measured value
disagreed with its header's claim (STOP-1 never matched), no claim's value varied between runs
(STOP-2 never matched — the only non-pinnable values are pids/addresses, asserted by shape per
the brief's own carve-out), and every claim was recoverable from the header, the code, or the
doc census (STOP-3 never matched).

## 3. Two negative/decisive findings surfaced while recovering claims

- **`probe-type-splice.wat` (#30):** the header asks "does a generic fn substitute its type
  param into a `forms` quote when called concretely?" Measured: **no** — `(probe::mk 5)`'s
  emitted form still carries the literal, un-substituted symbol `:I` at both the arg-type and
  return-type positions, never the caller's concrete `i64`. `forms` is a reflective AST quote,
  not a monomorphizing template. Asserted structurally (`ast-kind`/`ast-name` on the actual type
  keyword nodes), never against the rendered `edn/write` string.
- **`root-gapA.wat` (#33):** the header asks whether fn-forms on a hand-written fn diverges from
  fn-forms on a kwargs `$impl`-generated fn of the same shape. Measured: **no divergence** —
  both ship exactly 5 forms (fn-forms ships the full currently-registered macro/struct set, not
  a precise per-fn dependency closure, so the two share the same supporting-form count), and each
  one's LAST form correctly names its own target (`:test::hand` / `:test::work` via `ast-name`).
  "ROOT Gap A" resolves negative: no gap found between the two shippers.

## 4. Mutation proof — five `assert` probes, each gate test driven red by name, then restored

For each, the asserted expected value was changed to an impossible one, the SPECIFIC generated
test was run by exact name (`cargo nextest run --release -E 'test(/every_probe_runs::<name>/)'`),
confirmed FAILED, then the file was restored and re-run to confirm PASS. `git diff --stat` empty
after all five restorations (verified, §6).

1. **`probe-m1-apply-generic.wat`** — `d 10` → `d 999`:
```
FAIL [   0.378s] (1/1) wat::process every_probe_runs::probe_wat_scripts_probes_arc_170_probe_m1_apply_generic
thread '...' panicked at .../every_probe_runs.rs:41:5:
assertion `left == right` failed: wat-scripts/probes/arc-170/probe-m1-apply-generic.wat must run clean (exit 0) — ruling E3 ...
stderr:
#wat.kernel/AssertionFailure {:thread "main" :message "assert-eq failed" ... :actual "10" :expected "999" ...}
```
Restored; re-run: PASS.

2. **`probe-m1-arity.wat`** — `g 7` → `g 999`:
```
FAIL [   0.383s] (1/1) wat::process every_probe_runs::probe_wat_scripts_probes_arc_170_probe_m1_arity
#wat.kernel/AssertionFailure {:thread "main" :message "assert-eq failed" ... :actual "7" :expected "999" ...}
```
Restored; re-run: PASS.

3. **`probe-locus1-generic-surface-method.wat`** — `result 5` → `result 999`:
```
FAIL [   0.385s] (1/1) wat::process every_probe_runs::probe_wat_scripts_probes_arc_170_probe_locus1_generic_surface_method
#wat.kernel/AssertionFailure {:thread "main" :message "assert-eq failed" ... :actual "5" :expected "999" ...}
```
Restored; re-run: PASS.

4. **`probe-locus2-abstract-surface-dispatch.wat`** — `result 42` → `result 999`:
```
FAIL [   0.387s] (1/1) wat::process every_probe_runs::probe_wat_scripts_probes_arc_170_probe_locus2_abstract_surface_dispatch
#wat.kernel/AssertionFailure {:thread "main" :message "assert-eq failed" ... :actual "42" :expected "999" ...}
```
Restored; re-run: PASS.

5. **`probe-reason-downcast.wat`** — `code 2067` → `code 9999`:
```
FAIL [   0.385s] (1/1) wat::process every_probe_runs::probe_wat_scripts_probes_arc_170_probe_reason_downcast
#wat.kernel/AssertionFailure {:thread "main" :message "assert-eq failed" ... :actual "2067" :expected "9999" ...}
```
Restored; re-run: PASS.

## 5. Gates

| what | how | result |
|---|---|---|
| every probe | `cargo nextest run --release -E 'test(/every_probe_runs/)'` | `Summary [ 4.307s] 69 tests run: 69 passed, 6291 skipped` (70 − 1 retired `probe-edn2.wat`) |
| no print-only probe left | re-ran the orchestrator's census (`assert-eq`/`assertion-failed!`/`:wat::test::assert` grep over `git ls-files 'wat-scripts/probes/**/*.wat'`) | 5 hits remain, exactly the 5 `check-claim` dispositions (#1, #15, #22, #27, #35), each carrying its own `;; CLAIM (exit 0):` line |
| release floor | `scripts/floor.sh`, one run, foreground, nothing else running | `.floor/2026-10-01T07-56-44Z/clean.log`: `Summary [ 380.735s] 6336 tests run: 6336 passed (26 slow), 24 skipped`; exit=0. 6336 = 6337 (`82af2f167` baseline) − 1 (`probe-edn2.wat` retired, its generated `every_probe_runs` test gone with it). 24 skipped — unchanged from baseline. One run; green. |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | `Finished \`release\` profile [optimized] target(s) in 12.08s`, rc 0 |

## 6. Verification after mutation proofs

```
$ git diff --stat
 .../probes/arc-054/probe-054-fn-idempotency.wat    |  5 ++++
 ... (38 files changed, 290 insertions(+), 81 deletions(-))
```
No stray mutation leftovers — matches the 38 edited files (39 named probes minus 1 `git rm`).

## Files touched

- 38 probes edited in place (hand-edits, one named file each — 33 `assert`, 5 `check-claim`),
  each ending with a `;; CLAIM:` or `;; CLAIM (exit 0):` line; stale `EXPECT` prose corrected
  where present (e.g. `probe-s2-runner-count.wat`'s header already matched measured behavior,
  left as-is; `probe-kwargs-struct.wat`'s RED/GREEN framing kept as historical record, result
  now pinned by assertion rather than the bare "42" comment).
- `wat-scripts/probes/arc-170/probe-edn2.wat` — `git rm` (#37, retired — byte-identical
  duplicate of `probe-edn.wat`).
- `docs/arc/2026/06/255-builtin-registry/SCORE-STONE-255.76-every-probe-asserts-its-claim.md`
  — this file.

## STOPs triggered

None. See §2.
