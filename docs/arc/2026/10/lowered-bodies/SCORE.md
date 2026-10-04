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

Status: not started.
