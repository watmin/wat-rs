# SCORE — lowered bodies, a shim beside the interpreter

Executor's own clone: `/var/tmp/wat-rs-names`, branch `the-little-wat`, started at `89e3d49cd`
(the brief commit). Machine: `taskset -c 2` pins the same P-core logical CPU the
name-resolution arc used. Write this file AS YOU GO — see Method.

## Naming deviation from the brief, named before made

The brief says the new module is `src/lower/`. **`src/lower.rs` already exists** — a
pre-existing, unrelated module (`WatAST` → `HolonAST` algebra-core lowering, nothing to do
with this arc) that is itself a load-bearing entry on `tests/lint/holon_is_vsa_only.rs`'s
`VSA_HOME_FILES` allowlist (`const VSA_HOME_FILES: &[&str] = &["src/lower.rs", ...]`).
Renaming it out of the way to free the name would mean editing that lint test too — a file
outside this arc's own module, which STOP-3 says to name before making.

Lower-merge-cost fix, named here instead of made silently: the new module is **`src/body_lower/`**
— a sibling name, zero collision, zero edits anywhere outside it. Same shape the brief asked
for otherwise: one new module directory, one `mod` line in `src/lib.rs`
(`pub(crate) mod body_lower;`, next to the pre-existing `pub mod lower;`), one hook in
`apply_function`. No STOP fired — this did not require a change outside the module + the one
mod line + the hook; it just isn't named `lower`.

## L0 — the census

Status: **DONE.**

### What it is

`src/body_lower/mod.rs` — one file, this row's entire footprint besides the mod line and the
hook. Holds:

- `classify()` / `classify_call()` / `classify_if()` / `classify_let()` / `classify_args()` —
  a total, structural walk over `WatAST` answering whether a body consists ONLY of the forms
  L1 will lower (literals incl. bare keyword values; params/`let`-locals referenced by the
  SAME `Identifier` equality `Environment`/`EnvBuilder` already key bindings by — not a second
  notion of "the same name"; `if`; `let` with plain-symbol binders; calls to
  `SymbolTable::functions`-registered user functions; calls to intrinsics whose registry entry
  has a `value_handler` AND `@Purity Pure`). Anything else refuses, carrying the first
  refusing form's head (exact FQDN) or node kind (`Refusal` enum).
- `classify_function()` — seeds the bound set from `func.params` (+ `rest_param` via
  `Identifier::bare`, matching how a rest-param is actually bound) and walks the body once.
- The census itself: `record_application()` (the `apply_function` hook target), keyed by
  `Arc::as_ptr(&func) as usize` exactly as the brief's own L1 cache key will be — classification
  runs ONCE per distinct function pointer, cached inline in the same map entry as the
  application counter. `WAT_LOWER_CENSUS=1` gates everything; unset, the hook costs one
  `OnceLock` read and returns. At process exit (`libc::atexit`, already a crate dependency and
  already used for the same purpose elsewhere — `panic_hook`'s `#[ctor::ctor]` sibling pattern,
  not a new mechanism) it prints, to **stderr** (so a bench's stdout checksum stays
  unpolluted): a per-function table (applications, YES/no, first refusal) and a ranked table of
  refusal reasons weighted by application count.

### The hook

`src/runtime.rs`, inside `apply_function`'s trampoline `loop { ... }`, first line:

```rust
crate::body_lower::record_application(&cur_func, sym);
```

Placed INSIDE the loop (not once before it) so a self-tail loop's hops each count as their own
application — `tail-loop`'s 1,000,000 iterations show up as 1,000,000, not 1.

### Build

`cargo build --release`: clean on the first attempt, no new warnings (clippy::all is
`deny`-level workspace-wide; nothing fired).

### Census results — the three benches

All three ran with `WAT_LOWER_CENSUS=1 taskset -c 2`; checksums below are unchanged from the
name-resolution arc's own baselines (functional correctness unaffected by the census, as
expected — it only reads `Arc<Function>`/`SymbolTable`, never mutates eval).

**conj-build (N=1,000,000):** checksums `499999500000` / `499999500000` (unchanged).
110 distinct functions applied, 4,005,139 total applications, **4,000,009 (99.87%) land on a
function whose body would lower.** The mass is the two fold-closures `build-vec`/`sum-vec`
pass to `foldl` (plain `(fn [acc x] (:wat::i64::+ acc x))`-shaped — only params + a
value-doored, Pure intrinsic) — 3,000,000 + 1,000,000 applications, both YES. The functions
that DEFINE those closures (`:perf::build-vec`, `:perf::build-list`, `:perf::sum-vec`,
`:perf::sum-list`) each refuse once, at `:wat::core::foldl` itself (no value door on `foldl`)
— so the outer driver stays interpreted while the inner per-element work would lower. Ranked
refusals (weighted): `:wat::core::first` (2094/5 fns), `:wat::core::Option/expect` (1912/9),
`:wat::core::conj` (771/30), `:wat::core::match` (238/11) — all bootstrap/kernel-service
traffic, not bench-specific.

**call-heavy:** checksum `500013696418` (unchanged). 99 distinct functions, 3,640,743
applications, **only 9 (0.0002%) land on a would-lower function** — effectively none of the
bench's own three call shapes. **Why, with evidence:** `fib`/`tail-loop`/`closure-loop` each
refuse at their own `if` condition's comparison —
```
1000001  no  :perf::tail-loop    — refuses: :wat::i64::=
1000001  no  :perf::closure-loop — refuses: :wat::i64::=
 635621  no  :perf::fib          — refuses: :wat::i64::<
```
Grepped `src/intrinsic/i64.rs`: `:wat::i64::+`/`:wat::i64::-` carry
`#[wat_intrinsic(".." , value = eval_i64_add_value)]` / `value = eval_i64_sub_value` — a value
door. `:wat::i64::<` (line 412), `:wat::i64::<=` (448), `:wat::i64::=` (557) carry **bare**
`#[wat_intrinsic("..")]` — no `value = ..` clause at all, despite each one's own doc declaring
`@Purity Pure @Determinism Deterministic @Totality Total` in plain prose right above the
attribute. **This is a real, named gap, not a shim defect**: the i64 comparison family was
never migrated to the value-door registry (arc 255's migration evidently covered `+`/`-` but
stopped short of `<`/`<=`/`=`). Per STOP-3, adding `value = ..` handlers to
`src/intrinsic/i64.rs` is a change OUTSIDE `src/body_lower/` + the mod line + the hook — named
here, not made. **Consequence for L1, stated plainly now so it isn't a surprise at that row's
gate:** as specified, L1 will not lower `fib`/`tail-loop`/`closure-loop`'s top-level bodies,
because every one of them branches on a comparison with no value door. `call-heavy`'s
instruction-count delta under L1 is expected to be ~0%, not the 10–30% the brief's prediction
named for "leaf arithmetic and recursion helpers" — the prediction assumed the comparison
value doors already existed; they do not. `:perf::ns-between` (both benches use it, 2–3
applications total) separately refuses at `:wat::time::epoch-nanos` (also no value door).

**deep-scope:** checksum `65000000` (unchanged). 95 distinct functions, 1,005,098
applications, **only 6 land on a would-lower function.** The `step` closure (applied
1,000,000 times — the whole point of this bench) refuses on its FIRST form:
```
1000000  no  :wat::core::Fn @ wat-scripts/bench/deep-scope.wat:90:133
  — refuses: free variable `s0` (closure capture — not this function's own
    param/let-bound local)
```
Exactly the design's own prediction, confirmed mechanically rather than asserted: `step`'s body
references `s0`/`s63`, which are free variables reached through `closed_env` (the 64-deep
parent chain this whole arc exists downstream of), not `step`'s own params or
`let`-locals — outside L1's "read by SLOT in THIS frame" contract by construction. L1 does not
and should not touch this; a slot scheme for a closure's free variables is a different,
larger mechanism than this arc's additive shim.

### Census results — the floor's wat programs

Running the census across the actual `cargo nextest` floor isn't practical in one process:
nextest isolates each test to its own process, so the census's own cross-call accumulation
(the thing that makes the ranked table meaningful) would reset per-test — thousands of
near-empty reports, not one informative one. Instead: the **`kernel`** integration-test binary
(`[[test]] name = "kernel", path = "tests/kernel/mod.rs"`, run via plain `cargo test --release
--test kernel -- --test-threads=4`, which keeps every `#[test]` fn — 567 of them, covering a
large slice of `wat-tests/core/*.wat` and friends — in ONE process) gives a real, large,
representative accumulation: **34,838 distinct functions applied, 4,059,114 total
applications, 492,069 (12.1%) land on a would-lower function.**

Ranked refusal reasons (weighted by application count), top of table:

| applications | functions | reason |
|---|---|---|
| 1,134,800 | 4,588 | `:wat::core::first` |
| 775,471 | 15,591 | `:wat::core::conj` |
| 742,384 | 4,003 | `:wat::core::Option/expect` |
| 274,296 | 1,207 | `:wat::core::=` |
| 239,739 | 18 | `:wat::core::Vector` (Vector node in value position) |
| 169,390 | 925 | `:wat::core::match` |
| 169,092 | 23 | `:wat::core::ast-kind` |
| 34,205 | 589 | `:wat::core::ast-name` |

`match` is confirmed, by real measurement on this repo's own code (not just the brief's
prediction), as one of the top blockers — consistent with "the census decides" for L2. `first`/
`conj`/`Option/expect`/`=` outrank it here; all four are the CORE (polymorphic) spellings, not
a type-specific one like `:wat::i64::=` — read as "most stdlib code calls the generic verb,
which is either a `defclause`-dispatched form (not in `SymbolTable::functions`, not a single
registry entry) or an intrinsic without a value door" rather than "these verbs are impure."
Not re-verified per-verb here (out of L0's scope — L0 answers "does it lower," not "why not,
exhaustively, for every reason"); named as the natural next grep if L2 wants the generic-core
family specifically.

**An unrelated finding, isolated and ruled out, not left as a shadow over this row:** the first
`kernel` run (census on) reported `test result: FAILED. 547 passed; 17 failed`. All 17
failures are `wat_dispatch_*`/`wat_harness_deps` fixtures panicking at `src/freeze.rs:1162:9`
on `#wat.resolve/UnresolvedReferences` for `:rust::test::MathUtils` — a Rust-shim-registration
error with nothing to do with this arc's AST walk. **Controlled, not assumed:** re-ran the
SAME `cargo test --release --test kernel -- --test-threads=4` with `WAT_LOWER_CENSUS` UNSET —
identical 17 failures, same names, same panic site, 193.74s vs. the census run's 172.23s (both
ordinary variance). This isolates the cause to invoking this binary via plain `cargo test`
rather than `cargo nextest run` (nextest is this repo's own floor convention, confirmed green
at `/var/tmp/wat-rs-names-logs/floor-after-p2.log`, 5,405/5,405) — not to `WAT_LOWER_CENSUS` or
`src/body_lower/`. Per "never re-run a red," this was a DIFFERENT configuration run to isolate
causality (a control), not a re-run of the same one to make it go away; the result is reported
as a control, not discarded.

### Floor gate

The brief's Gates section ties the three-way floor run (`WAT_LOWER_CENSUS=0`/default/`check`)
to "after each of L1 and L2," not to L0. L0's own validation, short of that formal gate:
clean `cargo build --release`; all three benches' checksums unchanged with the census ON; the
`kernel` binary's 547 genuine passes (17 failures proven pre-existing/unrelated, above).
`WAT_LOWER_CENSUS` defaults OFF (env var unset in every normal run, including the whole
`nextest` floor), so the floor is structurally unaffected by this row — the hook's disabled
path is one `OnceLock::get_or_init` read per `apply_function` loop iteration, already paid
today by every other such read in this codebase's own conventions.

### STOP triggers

None fired. The naming deviation above was the one decision worth naming before making;
STOP-3 itself did not fire (no edit landed outside `src/body_lower/`, the one `mod` line, or
the `apply_function` hook).

## L1 — the shim, minimal

Status: **PAUSED by the builder, 2026-10-04 — code designed, written, and extensively tested;
TWO real bugs found and fixed; a THIRD class of gap found in the verification tool itself
(`WAT_LOWER=check`'s own comparator) and partly fixed; not yet fully green on all three modes,
so NOT committed (see "Paused" note at the bottom of this row and the file).** Builder is
moving to native work; wat-rs continues on the side later. Everything below is written so the
next session can reconstruct the exact code from this description alone — the working tree
was reverted to this file's own last commit (`bb957790a`, L0 only) per the pause instruction,
rather than leaving a half-green row uncommitted in the clone.

A full, buildable copy of the reverted code (the LAST state reached, i.e. with the keyword-lift
fix and the `values_equal_for_check` fix both applied, immediately before the pause) is saved
OUTSIDE the clone, for reference only, at:
- `/var/tmp/wat-rs-lowered-bodies-l1-paused/body_lower_mod.rs`
  (the full `src/body_lower/mod.rs`)
- `.../lowered-bodies-l1-paused/runtime.rs.diff` (the `apply_function` hook diff against `bb957790a`)
- `.../lowered-bodies-l1-paused/floor-l1-check-v2.log` (the full check-mode floor log this row's
  numbers below are read from)

These are scratch files in this executor's own session directory, NOT part of the repo and NOT
guaranteed to survive — treat the prose below as the durable record; the files are a shortcut
if they're still there.

### The design (as built)

- `src/body_lower/mod.rs` gained, beyond L0's census: a `Prog` tree (`Lit`, `Slot`, `If`, `Let`,
  `Seq` — implicit-do for a multi-form `let` body, `CallUser`, `CallIntrinsic`), a `LowerCx`
  that assigns slot numbers (a `Vec<(Identifier, u16)>` stack, resolved by scanning from the
  end so shadowing picks the most recent binder; `next` only ever grows, so truncating the
  stack when a `let` scope ends never reuses a slot number), `lower`/`lower_call`/`lower_if`/
  `lower_let`/`lower_args`/`lower_body`/`lower_function` (replacing L0's `classify*` family
  entirely — ONE compiler now, shared by the census and the shim), `exec`/`exec_tail` (the
  `eval`/`eval_tail` mirror — `exec_tail`'s `CallUser` arm emits
  `EvalBreak::Signal(EvalSignal::TailCall{..})` for `apply_function`'s own trampoline to catch;
  everything else defers to plain `exec`), `build_frame` (positional args + rest-param into a
  flat `Vec<Value>`, mirroring `apply_function`'s own existing bind loop minus the
  `Environment`/`EnvBuilder` construction), `ShimMode`/`shim_mode()` (`WAT_LOWER=0`/default/
  `"check"`), `ShimAction`/`shim_action()` (the hook's single entry point), and
  `assert_same`/`results_match`/`values_equal_for_check` (check-mode's comparator).
- The shared lowering cache (`LOWER_CACHE`, `lower_cached`) and the census's own per-function
  map (`CensusState`) both gained a **witness** field (`std::sync::Weak<Function>`) — see Bug 1
  below for why.
- `src/runtime.rs`'s `apply_function`: the L0 census hook stayed (first line of the trampoline
  `loop`); a new block right after the (unchanged) arity check and right before the (unchanged)
  `Environment`/`EnvBuilder` construction calls `crate::body_lower::shim_action(&cur_func,
  &cur_args, sym)` — `Interpret` falls through to the existing code unmodified; `Ran(result)`
  reproduces the SAME 4-arm match the existing `eval_tail` handling already does (`Ok` /
  `TailCall` / `TryPropagate` / `OptionPropagate` / `Diagnostic`) and returns/continues without
  ever reaching the `Environment` construction; `Checked(result)` is stashed in a
  `checked_result: Option<Result<Value, EvalBreak>>` local, and right after the (unchanged)
  `eval_tail(body_ast, &call_env, sym)` call — captured into a local instead of matched
  directly — `crate::body_lower::assert_same(&cur_func, checked, &interp_result)` runs before
  the existing `match interp_result { ... }` (also unchanged) takes over. Every edit stayed
  inside `apply_function` itself; nothing outside it, the one `mod body_lower;` line, or this
  function's body was touched.

### Bug 1 — a reused pointer, measured and fixed

`conj-build.wat`'s `sum-vec` builds an accumulator closure, folds it 1,000,000 times, and drops
it when `sum-vec` returns; `build-list`'s own, DIFFERENT accumulator closure, created
immediately after, was handed back the freed allocation by the allocator — same address, a
completely different function (`(acc x) -> acc+x` vs. `(acc i) -> conj acc i`). A cache keyed
by the bare `Arc::as_ptr(func) as usize` (what the brief's own prose names) served `build-list`
calls `sum-vec`'s stale lowered program, raising a real `:wat::i64::+` `TypeMismatch` on a List
value under `WAT_LOWER` (default, on) — a correctness bug, reproduced live on this exact bench,
before any floor run.

Fix: every cache entry (`LOWER_CACHE`'s `LowerCacheEntry`, and census's `CensusSlot`) carries a
`Weak<Function>` alongside its outcome. `Weak::upgrade` is tied to the original allocation's
control block, not the raw address — once the original object's strong count hits zero,
`upgrade` returns `None` permanently for that `Weak`, even after the allocator reuses the
address for an unrelated `Arc<Function>`. A cache hit requires BOTH the address to match (the
`HashMap` key) AND the witness to upgrade to the SAME live object (`Arc::ptr_eq` against the
function just received) — never a coincidence of addresses alone. On a miss (stale or first
sight), the census additionally RETIRES the old entry into a `Vec<CensusEntry>` rather than
discarding it, so the final report still accounts for every application ever recorded.

After this fix: `WAT_LOWER=0` and default both gave `499999500000`/`499999500000` on
conj-build, matching baseline.

### Bug 2 — "names are values," missed and fixed

`eval_inner`'s `WatAST::Keyword` arm (`src/runtime.rs`, around line 1678) runs a cascade before
falling back to a plain keyword literal: `:wat::core::nil` -> `Unit`; `:None`/
`:wat::core::None` -> a retired-spelling error; `:wat::core::Option.None` -> `Option(None)`; a
registered enum unit variant (`sym.unit_variant`) -> that `Enum`; a top-level `def`-bound name
(`sym.def_value`) -> the stored value; and — the one this arc's first pass missed —
**"Arc 009 — names are values. If the keyword is a registered user/stdlib define, lift it to a
callable Function value"** (`sym.get(k)` -> `Value::wat__core__fn`). A bare keyword naming a
registered function, used as a VALUE (not a call head), is NOT a keyword literal — it is a
function value. The first `lower()` treated every bare `WatAST::Keyword` as
`Prog::Lit(Value::wat__core__keyword(...))` unconditionally.

Caught by the full three-way floor gate (not by either bench): 66 failures under `WAT_LOWER`
(default, on), ALL in this one family —
`tests/rete/probe_arc278_7strat_native_differential.wat` passing `:wat::rete::fire-rules$oracle`
(a registered fn) as a bare-keyword ARGUMENT was the first one read in full; also
`wat_names_are_values::*` (a test file LITERALLY named for this feature),
`wat_arc157_def::*` (def-bound values), `enums::unit_variant_evaluates_via_bare_keyword`,
`wat_arc170_closure_extraction::*` (HOF args), `probe_arc237_stone6_is_predicate`,
`probe_def_not_special`. All `NotCallable: got keyword` or equivalent.

Fix: `lower_keyword(k, sym)` replicates the exact cascade (same order), baking the resolved
`Value` into a `Prog::Lit` once at LOWERING time rather than re-deriving it at exec time — sound
because every registry this reads (`unit_variants`, `runtime_def_values`, `functions`) is
populated once at freeze and never mutated during execution. Also added: a bare `WatAST::Symbol`
spelled `"nil"` (the PARSER's own nil-token form, per Stone 242.2 "Doctrine 1" — distinct from
`WatAST::NilLit`) is special-cased to `Value::Unit` BEFORE the general slot-resolution lookup,
matching `eval_inner`'s own order (this one was never observed failing — "nil" can't legally
name a parameter or let-binder — but the ORDER needed to match, not just the outcome).

After this fix: **the full floor, `WAT_LOWER=0`: 5405/5405 green (1549.264s).** **The full
floor, `WAT_LOWER` default (on): 5405/5405 green (1675.171s).** Both logged at
`/var/tmp/wat-rs-names-logs/floor-l1-off.log` / `floor-l1-on-v2.log`. Gate 2 (after L1) is
SATISFIED for these two modes.

### Bug 3 (partial) — `WAT_LOWER=check`'s own comparator, not the shim

Running the fixed binary's floor under `WAT_LOWER=check` surfaced a THIRD class of defect —
in the DIAGNOSTIC TOOL itself, not in `lower`/`exec`. `Value::wat__core__fn`'s own `PartialEq`
is `Arc::ptr_eq` BY DESIGN (`src/value/value.rs`'s own comment: "pointer-equality like fn ...
not implemented — same rationale") — two independently-evaluated closures are deliberately
never equal, even byte-identical ones. That rule is right for the LANGUAGE but wrong for
`assert_same`'s QUESTION ("did both paths compute the same answer"): a tail-called `(fn [b]
...)` literal gets constructed FRESH by the lowered exec AND by the interpreter running
independently on the SAME call, so the two `Arc<Function>`s are never `ptr_eq` even though they
close over the exact same source — `probe_arc278_vsa_where_native_differential` false-positived
on exactly this, with every OTHER field in the dumped values visibly identical.

Fix applied (`values_equal_for_check` + `functions_same_shape`, replacing the bare `==` inside
`results_match`): compare `Value::wat__core__fn` by SHAPE (name + params + rest_param + the
body's own source span — two evaluations of the identical textual `fn` literal always share
that span) when not literally the same allocation, recursing through `Vec`/`Tuple`/`Option`/
`Result`/`Aggregate`/`Enum` (the containers a closure can be smuggled inside that this pass
covered) and deferring to `==` for everything else.

**This fix is INCOMPLETE — confirmed by re-running the full floor under `WAT_LOWER=check`
after applying it:**

```
Summary [1645.613s] 5405 tests run: 5389 passed (3 slow), 12 failed, 4 timed out, 22 skipped
```

Full log: `/var/tmp/wat-rs-names-logs/floor-l1-check-v2.log` (also copied to the scratch
directory above). Gate 2's `WAT_LOWER=check` leg, and Gate 3, are NOT yet satisfied. The 16
failures, by confirmed or likely cause (not fixed — named for the next session):

- **Confirmed: more composite `Value` variants `values_equal_for_check` doesn't cover.**
  `probe_arc278_derived_exists_acc::*` (2) and the `wat::rete`/`wat::kernel` `Stream`/`seqable`
  rows (`deftest_wat_tests_core_core_seqable_seq_of_stream`,
  `parametric_surface_dispatches_stream`, `clause_arm_dispatches_stream`,
  `control_defn_dispatches_stream`, `deftest_wat_tests_core_core_seqable_generic_fn_over_seqable_accepts_all_four`
  — 5 more) read verbatim-identical `lowered=Ok(wat__stream__Stream(Empty))
  interpreted=Ok(wat__stream__Stream(Empty))` in the panic dump for the Stream case, and a
  `wat__core__PersistentMap(Trie(HashTrieMap{...}))` nested inside a `TailCall`'s `Aggregate`
  arg for the `derived_exists_acc` case — both variants this pass's `values_equal_for_check`
  falls through to bare `==` for, and `Value::eq`'s own `_ => false` catch-all (or a
  `Stream`-specific arm not reached by this grep) makes two independently-produced-but-
  equivalent instances compare unequal. Needs: explicit `wat__stream__Stream` and
  `wat__core__PersistentMap`/`wat__core__PersistentVector` arms in `values_equal_for_check`,
  recursing the same way the existing arms do (or, for `Stream` specifically, checking the
  project's own "Streams are Enumerators, not memoized" doctrine before deciding what
  "equal" should even mean for two lazy, stateful stream handles).
- **Confirmed: error LOCATION, not error KIND, differs.** `probe_int_modrem::*` (2) and
  `probe_rational_C3_i64_overflow::*` (2): both sides raise the identical `DivisionByZero` /
  `IntegerOverflow` error KIND, but at slightly different `Span`s (e.g. `col 46..67` vs.
  `col 65..66` for the same division) — the `Diagnostic` arm's `format!("{d1:?}") ==
  format!("{d2:?})"` includes the location, so a span-only difference reads as a
  "disagreement." Likely cause: `Prog::CallIntrinsic`'s `span` is the whole call's
  `list_span`; the AST-door handler the interpreter calls may report a narrower, more specific
  sub-expression span. Needs: either a location-insensitive comparison for the `Diagnostic` arm
  (compare `RuntimeErrorKind` only, via `into_kind()` / matching without the span) or lowering
  more than one span per call so `Prog::CallIntrinsic` can report the SAME location the AST
  door would. One further row, not yet triaged: `probe_runtime_error_produces_structured_edn`
  — likely the same span-sensitivity, not read in detail.
- **Confirmed: `WAT_LOWER=check` is O(n²) on a growing persistent collection, by design, not
  a bug.** `Value::Vec`/`PVec`'s equality is "by sequence" (its own doc comment) — comparing two
  independently-built length-k vectors costs O(k); doing that once per fold step over N
  elements costs O(N²) total. Measured directly on `conj-build.wat` BEFORE the floor run (not
  re-measured after the fixes, since the fixes don't touch this cost): N=1,000 -> ~33ms;
  N=10,000 -> ~2.9s (not linear — confirms the O(n²) read); N=50,000 -> 78s, correct checksums,
  no mismatch. Full N=1,000,000 would be computationally infeasible under `check` (the
  conj-build-with-census run alone, at full N, was killed after 10+ CPU-minutes with no sign of
  finishing). The floor's two `TIMEOUT`s under `check`
  (`deftest_wat_tests_deporder_verify_stdlib_runs`,
  `test_stdlib_load_order::verify_stdlib_has_no_load_order_violations`, plus
  `probe_arc275_verify_stdlib::probe_verify_stdlib_violation_count` /
  `_violations_detail` — 4 total, each at exactly the 30s per-test deadline) are very likely
  this same cost class (stdlib-verification walks are exactly the kind of code that folds over
  a large collection of declarations). Not a shim defect; a property of exhaustive differential
  checking against accumulated collection state. Needs a builder decision: whether
  `WAT_LOWER=check` is meant to be run at REDUCED scale (an env knob capping N on the known
  large-N fixtures) or whether the floor's own per-test deadline should be relaxed specifically
  under `check` mode (mirroring the earlier arc's own `DEFAULT_TIME_LIMIT_MS` stopgap
  precedent), given `check` is explicitly a diagnostic/CI gate, not a production path.

### What's still unmeasured for this row (not reached before the pause)

- The Gates' own instruction/cycle reporting (`perf stat -e instructions:u,cycles:u`, 3 runs,
  `taskset -c 2`, shim on vs. off) was NOT done formally for any of the three benches —
  wall-clock-only numbers exist for conj-build (`WAT_LOWER=0`: vec-ns 5.0s/list-ns 2.25s;
  default: vec-ns 4.36s/list-ns 1.54s — a real, measured improvement, but not the Gate's own
  metric). call-heavy and deep-scope were run functionally (checksums confirmed unchanged in
  all three modes) but not perf-stat'd, since — per L0's own finding — NOTHING in either bench
  lowers (both refuse entirely on the `:wat::i64::<`/`<=`/`=` value-door gap named in L0), so
  their instruction counts are expected to be ~0% different and lower-priority than finishing
  the correctness gates first.
- L2 (coverage from the census) was not started.

### STOP triggers

None fired in this row either. Every fix above (Bugs 1–3) landed inside `src/body_lower/`
alone; the `apply_function` hook's shape (one call to `shim_action`, one call to
`assert_same`) did not change after its first draft.

### Paused 2026-10-04 by the builder

Builder is moving to native work; wat-rs continues on the side later. Per instruction: the
clone's working tree was reverted to this file's own last commit (`bb957790a`, L0 only) —
`src/body_lower/mod.rs` and `src/runtime.rs`'s uncommitted L1 additions were NOT committed,
since `WAT_LOWER=check` was not yet green on the full floor (Gate 2/3 unmet). This file (and the
scratch-directory reference copy named above) is the complete resume record.

**Resume here, in order:**
1. Re-apply the L1 design described above (or copy the scratch-directory reference files back
   in) — `lower`/`exec`/`shim_action`/`assert_same` as designed, Bugs 1 and 2's fixes included
   (both are load-bearing — reverting them re-introduces a real correctness defect, not just a
   style choice).
2. Fix `values_equal_for_check`'s remaining gaps (`wat__stream__Stream`,
   `wat__core__PersistentMap`, `wat__core__PersistentVector`) and decide the `Diagnostic`-arm
   span-sensitivity question, per Bug 3's write-up above.
3. Decide (builder call, named above) how `WAT_LOWER=check` should handle the O(n²)
   large-collection cost on the floor's 4 timing-out tests — a capped-N knob, or a relaxed
   per-test deadline under `check` specifically.
4. Re-run the full three-way floor gate; once all three are green, commit L1 as one row and
   push, THEN do the Gates' own `perf stat` instruction/cycle measurement on the three benches
   (shim on vs. off) that this pause skipped.
5. L2 (coverage from the census) has not been started; L0's own floor-slice census (the
   `kernel` test binary numbers already in this file, above) is what L2's coverage order should
   read from — `:wat::core::first`/`conj`/`Option/expect`/`=`/`Vector`/`match` is the ranked
   order measured there.
