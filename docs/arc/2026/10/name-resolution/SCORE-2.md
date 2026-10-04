# SCORE-2 — the lookup's discarded clone, and the parent walk measured

Executor's clone: `/var/tmp/wat-rs-names`, branch `the-little-wat`, continuing from part 1's
`41352b548`. Brief: `BRIEF-2.md` at `75cf1aac3`. Baseline floor: part 1's
`/var/tmp/wat-rs-names-logs/floor-after-r1.log` (5,405 of 5,405, confirmed by the orchestrator's
own re-run).

Write this file AS YOU GO.

## P1 — no discarded clone

Status: DONE.

### The edit

`src/value/environment.rs`'s `lookup` (around line 210): `entry.value.provenance().clone()`
(an upfront clone of the WHOLE stored provenance, unconditionally, then thrown away on every
arm except `RuntimeBuilt`) replaced with matching `entry.value.provenance()` BY REFERENCE. The
`RuntimeBuilt` arm now clones only what it keeps (`call_span.clone()`; `producer: &'static str`
is `Copy`, dereferenced with `*producer`, no clone needed). Every other arm (`Unknown` /
`Literal` / `SymbolBound`) builds `Provenance::SymbolBound` straight from `entry.binding_span`
and `head_span` — no intermediate clone of the old provenance at all.

STOP-2 checked and does not apply: `Provenance` and `Span` both use plain `#[derive(Clone,
Debug, ...)]` (confirmed by reading `src/value/observe.rs:22` and
`crates/wat-reader/src/span.rs:51,69`) — no custom `Drop` impl, no side-effecting `Clone` impl
(no counter, no hook) on either type anywhere in the tree (grepped `impl Drop for Provenance`,
`impl Clone for Provenance`, `impl Drop for Span`, `impl Clone for Span` — zero hits beyond the
derives). Nothing outside `lookup` could have been reading a side effect of the clone this row
removes, because the clone never had one.

`cargo build --release`: clean on the first attempt.

### Re-measurement (same method as R0/R1)

Checksums, every run (functional + 3×perf-record + 3×perf-stat per workload): conj-build
`499999500000`/`499999500000`; call-heavy `500013696418` — unchanged. **Gate 3 satisfied.**

**Instructions:u / cycles:u, 3 runs each, taskset -c 2, P1 vs R1 (part 1's post-R1 numbers):**

| workload | metric | R1 mean | P1 mean | change | R1 spread | P1 spread |
|---|---|---|---|---|---|---|
| conj-build (N=1e6) | instructions:u | 28,380,056,129 | 27,801,363,098 | **-2.04%** | 0.06% | 0.02% |
| conj-build (N=1e6) | cycles:u | 13,976,872,988 | 13,873,579,043 | -0.74% | 1.38% | 3.85% |
| call-heavy | instructions:u | 54,113,393,785 | 52,931,203,432 | **-2.18%** | 0.67% | 0.90% |
| call-heavy | cycles:u | 23,244,187,186 | 23,246,153,196 | +0.01% (noise) | 0.46% | 6.78% |

Against the orchestrator's prediction ("P1 takes 2–5% of instructions off call-heavy and less
off conj-build"): call-heavy landed at the BOTTOM of the predicted range (2.18%, just inside
2–5%); conj-build (2.04%) again came in essentially equal to call-heavy, not clearly "less" —
same pattern as R1's prediction miss in the same direction. Cycles:u showed no reliable signal
either way — conj-build's small drop (0.74%) and call-heavy's flat reading (+0.01%) are both
inside or close to their own noise floors (3.85%/6.78% spread on only 3 runs); this row's win
is real in instructions, not demonstrated in cycles.

**What happened to `Environment::lookup` and `drop_glue::<Provenance>`, by name, as the brief
asked:**

| workload | symbol | R1 self-time | P1 self-time | direction |
|---|---|---|---|---|
| conj-build | `Environment::lookup` | 5.20% | **4.49%** | down |
| conj-build | `drop_glue::<Provenance>` | 2.66% | 2.71% | flat (noise) |
| call-heavy | `Environment::lookup` | 7.90% | **6.72%** | down |
| call-heavy | `drop_glue::<Provenance>` | 4.33% | 4.64% | flat/up (noise) |

`Environment::lookup` itself fell on both workloads — the self-time SHARE dropped on a total
pie that ALSO shrank, so this is a real, double-confirmed win (approximate absolute-instruction
read, multiplying each row's own share by its own 3-run instructions:u mean: conj-build's
`lookup` cost ≈1.476B→≈1.248B instructions, **-15.4%**; call-heavy's ≈4.275B→≈3.557B,
**-16.8%** — both figures are cross-run estimates, not a single precise measurement, since the
% share comes from one `perf record` run and the total comes from a separate 3-run
`perf stat` mean; read as "clearly down," not to two significant figures).

`drop_glue::<Provenance>` did NOT fall — it is flat within noise on conj-build and ticked up
on call-heavy. This is the honest, slightly counter-to-hypothesis result: removing the
clone-then-discard did not show up as a smaller `drop_glue::<Provenance>` bill. The likely
reason, read from the code rather than assumed: the ELIMINATED drop (of the now-never-created
discarded match scrutinee) was almost certainly getting INLINED directly into
`Environment::lookup`'s own self-time before P1 (same inlining pattern R2 named for FxHash vs.
SipHash) — so its removal shows up as PART OF `lookup`'s drop, not as a separate line that
shrinks. What `drop_glue::<Provenance>` still measures post-P1 is the cost of dropping
provenance values that are NOT the eliminated one: every `BoundEntry`/`TrackedValue` that is
legitimately still alive and eventually torn down (environment teardown, end-of-call cleanup)
carries a `Provenance` that must be dropped regardless of this row's edit. P1 did not touch
those, so their accounting is unaffected — and a workload with proportionally MORE such
necessary drops (call-heavy, which builds many short-lived single-binding environments per
tail-loop iteration) shows a slightly higher `drop_glue::<Provenance>` share after the pie
shrinks elsewhere, which is exactly what was measured.

### Gate after P1

`NEXTEST_TEST_THREADS=4 cargo nextest run --release` → **5405 tests run: 5405 passed (7 slow),
22 skipped (1624.678s). ALL GREEN** — same result as the baseline
(`/var/tmp/wat-rs-names-logs/floor-after-r1.log`). **Gate 2 (after P1) satisfied.**

## P2 — the parent walk, measured

Status: DONE (measured AND rewritten — the walk cleared the brief's ≥5% trigger).

### The new workload

`wat-scripts/bench/deep-scope.wat`: 64 SEPARATE, NESTED `(:wat::core::let [sN …] …)` forms
(`s0` outermost … `s63` innermost), each binding exactly ONE new name — so there are 64
distinct `Environment` objects chained parent-to-child, not one flat 64-binding environment.
A closure `step`, created inside the innermost scope (so its `closed_env` captures the whole
64-deep chain), is applied once per element of `(range 0 1000000)` via `foldl` (a
Rust-implemented builtin iterator — no wat-level recursion needed, so there is no self-reference
question to resolve for the loop driver). Each application reads `s0` (outermost — a 64-hop
parent walk to find it) and `s63` (innermost — a 0-hop hit, the control) and adds their sum to
the accumulator. Checksum: `s0=1`, `s63=64`, so each of 1,000,000 iterations adds 65 →
**`65000000`**, hand-verified against the formula, not just "it ran."

Generated with a small throwaway Python script (not committed — hand-writing 64 correctly
nested, correctly-paren-balanced `let` forms by hand was the wrong tool; the generator's output
is the committed, durable artifact, same spirit as the wat-fix codemod doctrine for `.wat`
corpus edits, though this is new-file generation, not an existing-corpus rewrite). Passes
`every_ungated_wat_checks` (confirmed before profiling).

### Profile (current code = after P1, before any loop rewrite)

```
perf record -g -o perf/deep-scope-p2.data -- taskset -c 2 target/release/wat wat-scripts/bench/deep-scope.wat
perf report -i perf/deep-scope-p2.data --stdio --no-children -g none
```

Checksum: `65000000` (hand-computed and confirmed, every run).

**Top 25 self-time symbols (cpu_core event, 20K samples):**
```
16.69%  <wat::value::environment::Environment>::lookup
 4.44%  wat::numeric::arith::eval_i64_arith::<eval_i64_add::{closure#0}>
 4.35%  libc cfree
 4.35%  wat::runtime::apply_function
 4.31%  libc malloc
 3.17%  <String as Clone>::clone
 3.13%  <WatAST as Clone>::clone
 2.48%  <WatAST as Clone>::clone (2nd call site)
 2.47%  core::ptr::drop_glue::<WatAST>
 2.13%  wat::runtime::eval_inner
 2.07%  core::ptr::drop_glue::<WatAST> (2nd call site)
 2.00%  core::ptr::drop_glue::<Provenance>
 1.99%  libc.so.6 0xa633d (unresolved)
 1.93%  <sip::Hasher<Sip13Rounds> as Hasher>::write
 1.91%  wat::runtime::eval_tail
 1.89%  libc.so.6 0xa6324 (unresolved)
 1.59%  <RandomState as BuildHasher>::hash_one::<&str>
 1.48%  __rustc::__rdl_alloc
 1.47%  <EnvBuilder>::bind_unknown_span::<String>
 1.45%  <HashMap<String, BoundEntry, FxBuildHasher>>::insert
 1.21%  libc.so.6 0x17640d (unresolved)
 1.05%  wat::collection::transform::__wat_intrinsic_shim_eval_vec_foldl
 0.99%  wat::runtime::eval_list
 0.91%  libc.so.6 0xa79d7 (unresolved)
 0.91%  <Arc<String>>::drop_slow
```

**The parent walk's share: `Environment::lookup` is 16.69% self-time** — roughly 2.5–3.7×
conj-build's 4.49% and call-heavy's 6.72% (both measured in P1, same code). Since `lookup`'s
recursive self-call reuses the SAME function symbol at every level, perf's self-time accounting
cannot separate "per-level hash-probe-and-maybe-build work" from "the recursive hop itself"
within one flat symbol — but the comparison across workloads isolates it anyway: deep-scope's
distinctive extra cost, relative to conj-build/call-heavy, is ENTIRELY attributable to each
logical lookup of `s0` costing 64 `Environment::lookup` invocations instead of ~1–2, since
every other per-call cost (the FxHash probe, the value clone, the P1-slimmed Provenance
construction) is identical machinery paying the identical per-hit price the other two workloads
already pay. **16.69% ≥ the brief's 5% trigger — rewrite the walk as a loop.**

**Noise floor (3 runs, taskset -c 2, pre-rewrite):** instructions:u mean 15,511,080,837 (spread
2.15%); cycles:u mean 5,512,044,585 (spread 3.62%).

### The loop rewrite

`src/value/environment.rs`'s `lookup`: the tail call `self.inner.parent.as_ref().and_then(|p|
p.lookup(name, head_span))` became a `let mut current = self; loop { … match current.inner
.parent.as_ref() { Some(parent) => current = parent, None => return None } }`. SAME result and
SAME order: check `self`'s own bindings first, then each ancestor in turn (outermost last),
returning on the first hit or `None` at the chain's root — only the "move to the next level"
mechanism changed, from a recursive call to reassigning a local reference. P1's by-reference
`match` and Provenance construction are otherwise untouched, inside the loop body verbatim.
`cargo build --release`: clean on the first attempt (no lifetime issue — a `&Environment`
local can be reassigned to a `&Environment` borrowed through its own `parent: Option<Environment>`
field across loop iterations without the borrow checker objecting, since each step only needs
the PREVIOUS reference's validity to produce the NEXT one, never both at once).

### Re-measurement — all three workloads, exactly as R0/P1 (Gate 3)

Checksums, every run: conj-build `499999500000`/`499999500000`; call-heavy `500013696418`;
deep-scope `65000000` — **all unchanged. Gate 3 satisfied.**

| workload | metric | pre-loop mean | post-loop mean | change |
|---|---|---|---|---|
| conj-build | instructions:u | 27,801,363,098 | 27,720,297,439 | +0.29% (noise) |
| conj-build | cycles:u | 13,873,579,043 | 13,676,821,926 | +1.42% (likely noise; spread 3.85%→1.51%) |
| call-heavy | instructions:u | 52,931,203,432 | 53,122,712,179 | -0.36% (noise) |
| call-heavy | cycles:u | 23,246,153,196 | 23,186,124,379 | +0.26% (noise) |
| deep-scope | instructions:u | 15,511,080,837 | 15,506,936,494 | **+0.03% — no signal** |
| deep-scope | cycles:u | 5,512,044,585 | 5,672,617,705 | -2.91% (within its own ~3.6%/3.8% spread) |

**`Environment::lookup`'s self-time on deep-scope, pre- vs. post-loop: 16.69% → 16.71%** —
flat, inside noise. The brief's own prediction ("the loop rewrite, if it happens, is worth
little on the other two") held for conj-build and call-heavy (both flat) — and, measured
honestly, ALSO held for `deep-scope` ITSELF, which the orchestrator's prediction did not call
out as a possible outcome. Read against the code: the original recursive form
(`self.inner.parent.as_ref().and_then(|p| p.lookup(…))`) is a self-tail-call with no work
after the recursive call returns — exactly the shape LLVM's optimizer turns into a loop at the
machine-code level in a release build, with or without the Rust source saying `loop` explicitly.
The explicit Rust-level rewrite most likely produced near-identical generated code to what the
compiler had already synthesized from the recursive form — so there was no hidden "function
call overhead" for the rewrite to remove. The walk's real cost (16.69%) is the PER-LEVEL hash
probe repeated 64×, not the mechanism connecting one level to the next; a loop and a compiler-
optimized recursive tail call pay that same per-level cost either way.

**Kept, not reverted:** the loop form is clearer to read as an iteration (matches the brief's
explicit instruction to rewrite once the ≥5% trigger fired) and removes any DEPENDENCY on the
optimizer recognizing the tail call, which is a `-C opt-level`/LLVM-version-contingent guarantee,
not a language one — a debug build or a future compiler change could regress the recursive
form's stack safety on a pathologically deep chain (the-little-wat's own eval-stack-safety arc,
#261, names recursive-eval-without-TCO as a live, tracked risk elsewhere in this codebase) in a
way the explicit loop never can. The measured verdict is "no instruction-count win," not "no
reason to keep it."

### Gate after P2

`NEXTEST_TEST_THREADS=4 cargo nextest run --release` → **5405 tests run: 5405 passed (4 slow),
22 skipped (1510.776s). ALL GREEN** — same result as the baseline. **Gate 2 (after P2)
satisfied.**

## P3 — the slot question, sized, not done

Status: DONE (a map, not a strike — no code changed for this row, no floor/gate run needed).

### What "resolving a local to a slot once" would take off each workload

Estimate method: sum the SAME name-resolution symbol bucket used throughout this arc (the
hashing symbols — `sip::Hasher::write`, `RandomState`/`FxBuildHasher::hash_one` — plus
`Environment::lookup`, `EnvBuilder::bind_unknown_span`, the `BoundEntry` map's `insert`/
`reserve_rehash`, `bind_let_binding`), taken as the UPPER BOUND of what a slot scheme removes:
a resolved-once local becomes a direct index into a flat per-frame `Vec`/array — no hash
computation, no bucket probe, no parent-chain walk, and binding becomes a `Vec::push`/indexed
write instead of a hashed insert. **Not** included in the bucket, and NOT expected to shrink
much: the separately-counted `drop_glue::<Provenance>` (2.00–4.64% across the three) and the
bare `Value`/`String` clones — a slot still has to clone the stored value and still has to
build a `Provenance` for diagnostics on each reference (P1 already made that construction as
cheap as this representation allows); indexing changes how the storage is FOUND, not what is
done once it is found.

| workload | current name-resolution bucket (post-P1/P2) | read as |
|---|---|---|
| conj-build (N=1e6) | **~11.76%** of instructions | upper-bound estimate of what a slot scheme removes |
| call-heavy | **~16.3%** | upper-bound estimate |
| deep-scope (64-deep, adversarial) | **~22.7%** | upper-bound estimate, but see caveat below |

Caveat on `deep-scope`: it is a DELIBERATELY adversarial micro-benchmark built for this arc to
force a visible parent-walk signal (P2) — no ordinary wat program nests 64 single-binding
lexical scopes with no closer binding shadowing the outer name. Its 22.7% is the CEILING a
slot scheme could claim on a pathological case, not a representative number; conj-build and
call-heavy (ordinary benchmarks, pre-dating this arc) are the more honest guide to a typical
program's gain. Read together: a slot scheme's benefit scales with how deep/frequent a
program's lexical nesting is — small on shallow, ordinary code (conj-build, call-heavy:
roughly a eighth to a sixth of total instructions), large on code that nests deeply (whatever
fraction of real wat programs that turns out to be — unmeasured here; the-little-wat's own
self-hosting compiler would be the natural next corpus to check, OUT OF THIS ARC'S TERRITORY).

### What files it would touch, by grep

A slot scheme needs, at minimum: (1) a resolution PASS that assigns each local reference a
`(depth, slot)` pair once (naturally hooking into the checker's EXISTING per-scope walk,
`infer_let` at `src/check.rs:8375`, which already tracks `let`-binding lexical scope during
type inference — REUSE is a candidate, not a given; the project's own standing caution is that
"a reused walk flips safety" (feedback memory), so this is named as a candidate site for the
builder's decision, not verified safe here); (2) a slot-indexed storage shape for
`Environment`/`EnvBuilder` (`src/value/environment.rs`) alongside or instead of the current
`BindingMap`; (3) every call site that resolves a name TODAY, updated to use a slot once one
exists.

Grepped call-site counts (`grep -n` on `src/`, this clone, post-P2):

| call | total sites | files touched | heaviest file |
|---|---|---|---|
| `.lookup(` (`Environment::lookup`) | 35 | 6 (`closure_extract.rs`, `runtime.rs`, `reflect/match.rs`, `intrinsic/mod.rs`, `rete/purity.rs`, `value/environment.rs`) | `runtime.rs` — 29 |
| `.bind(` / `.bind_unknown_span(` (`EnvBuilder`) | 29 | 10 (`lib.rs`, `freeze.rs`, `runtime.rs`, `declare/register.rs`, `kernel/spawn.rs`, `macros/expand.rs`, `reflect/match.rs`, `rete/eval_test.rs`, `value/mod.rs`, `value/environment.rs`) | `runtime.rs` — 20 |
| `.child()` (`Environment::child`) | 23 | 8 (`freeze.rs`, `declare/register.rs`, `function/eval.rs`, `runtime.rs`, `kernel/spawn.rs`, `macros/expand.rs`, `rete/eval_test.rs`, `reflect/match.rs`) | `runtime.rs` — 15 |

`runtime.rs` (22,028 lines) and `check.rs` (24,506 lines, 405 `fn`s) are the two files any slot
scheme cannot avoid: `runtime.rs` is where almost every `.lookup`/`.bind`/`.child` call
actually lives (the eval dispatch), and `check.rs` is where slot numbers would most naturally
get ASSIGNED (it already walks lexical scope once per `let`/`fn`/`defn` to type-check; the
assignment pass would ride that same walk or a sibling one). `src/value/environment.rs`
(this arc's own file, 275 lines before this arc, now larger) is the representation itself and
would need the new slot-indexed storage shape. The AST (`crates/wat-reader/src/ast.rs`)
represents every name reference as a bare `Keyword(String, Span)` (grepped — no existing
"resolved identifier" variant) — a slot scheme needs SOMEWHERE to carry the resolved
`(depth, slot)` per reference: either a new `WatAST` variant (touches the reader crate, a
dependency boundary this arc never crossed) or a side-table keyed by node identity/span
(cheaper to land, more at risk of staleness if the AST is later mutated post-resolution).

**This is the map, not the strike.** The builder's decision is whether ~12–16% off two
ordinary workloads (and more on pathologically deep ones) is worth a change that reaches into
the checker's scope-walk, the AST's name-reference representation, and ~90 call sites across
~15 files in the interpreter's two largest modules — versus a narrower, lower-risk cut (e.g.
caching a resolution per `WatAST` node after its first lookup, which would help repeated calls
of the SAME function without touching the checker or the AST shape at all, at a smaller but
nonzero fraction of this ceiling).

## STOP triggers fired

None of STOP-1/STOP-2 fired. STOP-2 was checked directly (not assumed) at P1 — see that row —
and found not to apply: `Provenance`/`Span` carry no custom `Drop`/`Clone`, so nothing outside
`lookup` could have observed a side effect of the clone P1 removed.

## Summary of rows

| row | status | code changed | floor gate |
|---|---|---|---|
| P1 — no discarded clone | DONE | yes (`environment.rs`) | green, same result |
| P2 — parent walk measured + looped | DONE | yes (`environment.rs`, new `deep-scope.wat`) | green, same result |
| P3 — slot question sized | DONE | no | not applicable (no code change) |
