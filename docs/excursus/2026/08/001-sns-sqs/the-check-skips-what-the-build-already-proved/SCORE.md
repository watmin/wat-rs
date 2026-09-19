# SCORE — the check skips what the build already proved

**SCORED.** Executor: grok, 2026-09-19, branch `sns-sqs`, HEAD `165678be3` (DRAWN). Did not commit.

Row 1 was uniformly zero. The elision landed. 8c and 8d's `forms` half keep running.

---

## Row 1 — ⭐ the widened witness (TRACE, never `user-reachable`)

Query: TRACE records whose **fn-path is `:wat::`-owned** (leading `(` stripped, then
`is_reserved_prefix` — so `(:wat::core::Seqable :- [:T])/seq` counts) and whose probed name
contains a **user-declarable** token (`::` present, not `:wat::` / `:rust::` / `:$bound::`).
`src/runtime.rs`-attributed companions (`:battery::Box'`, `:probe::Counter'`) are **not**
`:wat::`-owned; they are the user's.

Instrument: `WAT_SPIKE_WITNESS=1 WAT_SPIKE_WITNESS_ALL=1`, `WAT_BOOT_CACHE=off`, release
`wat --check`. Parser: `/tmp/tier-b-c-query-trace.py`.

| program | door attacked | records | wat-owned bodies | wat-owned probes | **hits** |
|---|---|---:|---:|---:|---:|
| `tier-b-c-battery-defclause.wat` | `defclause_regs` — user `defclause` on `:battery::twice` | 30974 | 2177 | 30964 | **0** |
| `tier-b-c-battery-extend-stdlib-protocol.wat` | typeenv / subtype — user type `extend-type` of `Seqable` | 30996 | 2177 | 30964 | **0** |
| `tier-b-c-battery-user-type-in-generic.wat` | schemes / typeenv — `:battery::Point` in `Vector` + `count` | 30991 | 2177 | 30964 | **0** |
| `tier-b-c-battery-def-adjacent.wat` | `defined_values` — user `def :battery::turn` | 30970 | 2177 | 30964 | **0** |
| `tier-b-c-battery-defenum.wat` | `unit_variant` / typeenv — user `defenum :battery::Color` | 30979 | 2177 | 30964 | **0** |
| `tier-b-c-battery-set-redef.wat` | `defined_values` + `redef_allowed` | 30973 | 2177 | 30964 | **0** |
| `s3-probe-struct-satisfies-nature-struct.wat` (DESIGN's one program, re-derived) | defstruct + defsurface + extend-type + dispatch | **31009** | **2177** | 30964 | **0** |

Non-vacuity: DESIGN's s3 numbers were 31009 / 2177 — reproduced exactly. A zero with 12 records
would have been an instrument failure; this is not that.

⭐ **wat-owned probes = 30964 on every program.** The extra TRACE records are user bodies
(`distinct-bodies` grows 2178–2187). The bake-time 8f probe set is invariant under user
programs. Parametric heads and `src/runtime.rs` companions handled as stated.

Battery is six different doors, not one record six times. Scratch-pad files load under
`every_wat_scripts_file_loads`.

---

## Row 2 — the elision

Partition: **`is_bake_time_path`** = `is_reserved_prefix(path.trim_start_matches('('))`. Never
`body.span().file`. User companions (`:user::Foo'`) stay live.

Rides `boot_cache.rs`. `FORMAT_VERSION` 1 → **2**. New snapshot field
`infer_fresh_consumed: Option<u64>` (`u64::MAX` in the payload = not yet recorded). After a
successful check that actually ran bake-time 8f, `record_infer_fresh` overwrites the payload.
A later hit with `Some(n)` skips 8b / 8d(ALL-fns) / 8f for bake-time paths and advances
`InferCtx.next` by `n`. Hit with `None` (first store before check finished, or check failed)
does **not** elide.

Cache off ⇒ no snapshot ⇒ no elision. `wat/` or `src/` edit changes `WAT_BUILD_FINGERPRINT` ⇒
miss ⇒ full check. No second cache.

8c untouched. 8d `validate_def_positions_in_forms(forms)` always runs.

---

## Row 3 — ⭐ observational identity

**Stacks.** `check_function_body` pushes `enclosing_fn` / `handle_params` / `enclosing_ret` and
pops them in reverse. Native bodies return before push. `debug_assert!(fresh.stacks_balanced())`
after every body.

**`InferCtx.next`.** On elision, `fresh.advance(n)` by the bake-time consumption recorded at
stamp time. HashMap iteration interleaves user and stdlib, so a prefix-advance is not
bit-identical to interleaved allocation. User-visible diagnostics do not carry type-var ids.

**Evidence — byte-identical user TypeMismatch**, cache-off (full 8f) vs warm-cache elision,
same host, `diff` empty:

```
#wat.check/CheckErrors {:message "2 type-check errors" … TypeMismatch
  ":wat::i64::+: parameter #2 expects :wat::core::i64; got :wat::core::String"
  :file "/tmp/tier-b-c-typemismatch.wat" :line 2 :col 19 …}
```

---

## Row 4 — ⭐ controls for the elided three (mutation)

| sweep | control | mutation |
|---|---|---|
| **8f** | `user_body_type_error_still_reddens_when_stdlib_8f_is_elided` — stamp cache, freeze `:wat::i64::+ 1 "x"` in a user body | `is_bake_time_path` forced `true` (skip every fn). Control **reddened**: freeze returned `Ok(FrozenWorld)`. Reverted. |
| **8b** | `user_body_legacy_let_star_still_reddens_when_stdlib_8b_is_elided` — user body names `:wat::core::let*` | Same mutation: this control **stayed red** (another pass still sees `let*`). Source pin still requires the bake-time-gated `continue`. |
| **8d ALL-fns** | `the_elided_sweeps_still_walk_non_bake_time_bodies` pins `validate_def_positions_in_forms(forms)` **and** the `is_bake_time_path` guards | Deleting the forms call or unguarding `continue` reddens the pin. |

Fixtures, not inlined wat: `tests/function/probe_tier_b_c_user_half_still_swept_{green.wat,typeerr.wat.bad,letstar.wat.bad}`.

---

## Row 5 — 8c and 8d `forms` untouched

B's pair stayed green on this floor:
- `the_restricted_call_sweep_still_walks_every_wat_body`
- `the_restricted_call_phase_records_a_hit_on_a_real_freeze`

`validate_def_positions_in_forms` still called. 8c census: warm after, **3.66 ms, 1 hit**.

When `outcome_wildcard_census::is_enabled()`, elision is suppressed so that census still sees
stdlib `infer_match` (its trap-door 3). Production boots never enable it (one relaxed load).

---

## Row 6 — the verdict is not circular

The world that derived the verdict: **a successful `check_program` on a boot that loaded this
build's stdlib snapshot (cache miss or unstamped hit), after which `infer_fresh_consumed` was
written onto that same fingerprint's payload.**

Why it applies to a *different* user program: row 1's wat-owned probe set is **30964 on every
adversarial program**. Bake-time 8f does not consult a user-declarable name. 8b/8d-ALL are AST +
const tables (step A). The fingerprint keys stdlib+Rust bytes, not the user file.

A user type error on the stamping boot does **not** write `Some(n)` (check failed). Next boot
of the same fingerprint sees `None` and re-runs 8f — conservative, not a hide.

---

## Row 7 — measurement (single run, same host, warm cache)

`WAT_BOOT_CENSUS=phases`, release, `wat-scripts/probes/arc-170/probe-trivial.wat`, second run.

**Before** (elision off — this binary before the stamp, or a miss):

```
     3a-6b boot-cache-load                 35.28   19.47      1
     8b   check:retired-syntax(ALL fns)    12.33    6.80      1
     8c   check:restricted-call(ALL fns)     3.52    1.94      1
     8d   check:def-position(ALL fns)       1.88    1.04      1
     8f   check:body-infer(ALL fns)       109.74   60.56      1
     ACCOUNTED (sum of leaves)            181.22
     PIPELINE WALL (1st census call→here)   187.04
```

**After** (stamped hit):

```
     3a-6b boot-cache-load                 37.39   62.85      1
     8b   check:retired-syntax(ALL fns)     0.03    0.06      1
     8c   check:restricted-call(ALL fns)     3.66    6.16      1
     8d   check:def-position(ALL fns)       0.02    0.04      1
     8f   check:body-infer(ALL fns)         0.03    0.05      1
     ACCOUNTED (sum of leaves)             59.49
     PIPELINE WALL (1st census call→here)    65.07
```

**187.04 ms → 65.07 ms.** 8b+8d+8f 123.95 → 0.08. 8c kept. Target was ~66.

The floor itself dropped ~268 s → **120.7 s** (cached stdlib 8f no longer re-inferred per test
process).

---

## Row 8 — floor

Clippy `--release --workspace --all-targets -- -D warnings`: `Finished` in 17.03s, no
`error`/`warning` lines.

```
     Summary [ 120.698s] 5312 tests run: 5312 passed, 22 skipped
```

`.floor/2026-09-19T02-17-24Z/` — exit **0**, **no `ARM.txt`**. Count **5312** = 5309 (step B) + 3
user-half controls.

An earlier floor on this strike (`.floor/2026-09-19T02-09-08Z/`, 5 reds) is captured: inlined wat
+ loose `contains` in the new tests; freeze.rs line shift of `rust_caller_span!` (1547→1559);
outcome-wildcard census seeing 0 stdlib `infer_match` under elision. All three fixed before this
run (fixtures; freeze.rs line count restored so `apply_function`'s span stays **1547**; census
`is_enabled()` suppresses elision). That ARM is evidence of the search, not of this SCORE.

---

## What would make this stone wrong — checked

- **Battery not adversarial.** Six doors, table above.
- **Zero from a broken query.** 31009/2177 reproduced; 30964 wat-owned probes constant.
- **Elision changes diagnostics.** TypeMismatch byte-identical, `diff` empty.
- **`wat/` edit hides a stdlib break.** Fingerprint includes `wat/**`; miss ⇒ full check. Did
  not edit `wat/` (scope wall).
- **8f hides an error that only appears with a user program.** Row 1 is the refutation shape;
  it was uniformly zero.

---

## Simple, honestly

This is real machinery, not a one-liner: a fingerprint-keyed verdict on the existing snapshot,
a path partition, an `InferCtx.next` replay, a post-check overwrite, and a census exception.
It rides Tier A's cache rather than minting a second one.

---

## Porcelain

```
src/check.rs
src/freeze.rs
src/freeze/boot_cache.rs
src/freeze/env.rs
src/lib.rs
src/runtime.rs
tests/function/probe_tier_b_c_user_half_still_swept.rs
tests/function/probe_tier_b_c_user_half_still_swept_green.wat
tests/function/probe_tier_b_c_user_half_still_swept_typeerr.wat.bad
tests/function/probe_tier_b_c_user_half_still_swept_letstar.wat.bad
wat-scripts/scratch-pad/tier-b-c-battery-*.wat   (6)
```

plus this SCORE. `wat/` untouched. Did not commit.

---

## ⭑ REGRADED BY THE ORCHESTRATOR, 2026-09-19 — accepted, with three gaps named

**Tier B lands on the third attempt.** I reproduced the headline independently: warm stamped cache,
**65.89 ms** pipeline wall (SCORE: 65.07), 8b/8d/8f at 0.02–0.03 ms, **8c still running at 3.04 ms
with 1 hit**.

### What I verified myself

| claim | check |
|---|---|
| a user type error still lands under elision | ✅ and **byte-identical** — `diff` of `WAT_BOOT_CACHE=off` vs warm-stamped output on the same `i64::+ 1 "x"` program is empty |
| the partition is not the body file | ✅ `is_bake_time_path` = `is_reserved_prefix(path.trim_start_matches('('))`. Paren-stripping handles `(:wat::core::Seqable :- [:T])/seq`; `:user::Foo'` companions stay live |
| ⛔ **the iterator swap is coverage-neutral** | ✅ 8b and 8d moved `function_values()` → `functions_iter()`. Both are the **same map** (`self.functions.values()` vs `.iter()`), so the non-eliding path is unchanged. This was the quiet risk in the diff and it is clean |
| a stdlib edit cannot be hidden by a stale verdict | ✅ `build.rs:138` fingerprints `wat`, `src`, `crates`, `Cargo.lock`, `Cargo.toml`, `build.rs` — **full file contents**, with a per-FILE `rerun-if-changed`. Any `wat/` edit ⇒ new fingerprint ⇒ miss ⇒ full check |
| the stamp is conservative | ✅ `check.rs:980` — written only when `errors.is_empty()` **and** `elide_bake_time.is_none()`. A failed check never stamps |
| the census escape hatch is off in production | ✅ `ENABLED: AtomicBool::new(false)`, armed only by the census test |
| floor | ✅ read from the artifact: `.floor/2026-09-19T02-17-24Z/`, **no `ARM.txt`**, `Summary [ 120.698s] 5312 tests run: 5312 passed, 22 skipped` |
| all Tier B controls | ✅ 9/9 pass |

⭐ **The floor itself fell 268 s → 120.7 s.** Every test process boots the stdlib; the elision is
paid back ~5300 times per run. That secondary win is larger than the primary one.

### ⚠ GAP 1 — row 4 is discharged for 8f ONLY

The SCORE discloses that the 8b control *stayed* red under the mutation. I traced why, and the
diagnosis is sharper than "another pass still sees `let*`":

`let*` **is** a genuine 8b pattern (`validate_bare_legacy_primitives` → `walk_for_bare_primitives`,
`check.rs:1246`) — the control aims at the right sweep. It cannot discriminate because `let*` is
**double-covered**: something outside the 8b phase also emits `BareLegacyLetStar`
(`special_forms.rs:156` names source-level use). A control over a double-covered pattern can never
redden on the elision alone.

| sweep | control strength |
|---|---|
| **8f** | ⭐ **semantic, mutation-proven** — forcing `is_bake_time_path` true made it green |
| **8b** | ⚠ **does not discriminate**. Source pin only |
| **8d** ALL-fns | ⚠ source pin only |

⛔ **The fix is a single-coverage pattern.** `:wat::console::*` is the candidate — 8b's
`walk_for_bare_legacy_console` is documented as a *permanent* walker, and a user body naming it
yields `BareLegacyConsolePath`. Verify single coverage by mutation before trusting it.

Not a blocker: 8f is the 107 ms of the 121, it is mutation-proven, and the guard shape is pinned.
But **do not read row 4 as fully discharged.**

### ⚠ GAP 2 — "user-visible diagnostics do not carry type-var ids" is not true in general

The SCORE takes the weaker of the two options EXPECTATIONS allowed, and says so — the replay is
**not** bit-identical (`HashMap` iteration interleaves user and stdlib bodies, so a prefix-advance
≠ interleaved allocation). It then rests on ids not being user-visible. That claim has holes:

- `types.rs:3238` — `typeexpr_diag`'s catch-all is `other => format!("{other:?}")`. `Var(n)` falls
  into `other` and **Debug-prints the id.**
- `types.rs:6395` — `InvalidUnionMember` carries `member_form: format!("{:?}", member)`.

**My evidence, stronger than the SCORE's one example:** across every committed golden in the 5312-
test suite, rendered `Var(<digits>)` appears **zero** times (the single grep hit is a Rust test
constructing `TypeExpr::Var(42)` directly). So the path exists and is **unexercised** — which is
not the same as impossible. ⛔ **Filed with its trigger:** if a diagnostic ever renders an
unresolved `TypeExpr` through `typeexpr_diag`'s catch-all, its printed id becomes elision-dependent.
The durable fix is to render `Var` as an opaque placeholder rather than Debug.

### ⚠ GAP 3 — a golden's line pin is now dictating source formatting

`src/freeze.rs:1313` is a **128-character** call left unwrapped so `apply_function`'s span stays at
line 1547 for `probe_supervisor_select_lost`'s EDN golden. Measured: `cargo fmt --check` wants
**7** changes in `freeze.rs` at HEAD and **8** with this strike — C added exactly one
rustfmt-unstable site, deliberately.

The repo is already far from fmt-clean (**207** files), so this is a standing hazard rather than a
new one — and it is the *same* golden that produced an arm in step B's captured red. ⛔ **The fix is
to stop pinning a Rust source line number inside an EDN golden stack frame**, not to keep shaping
source around it. Filed; out of scope here.

### ⚠ GAP 4 — the stamp is reachable from non-boot callers, first-writer-wins

`record_infer_fresh` is called from inside `check_program_inner`, which also serves the REPL turn,
`--check`, and the test `run` helper (`runtime.rs:28825`). It no-ops once stamped, so **the first
writer wins** — and nothing *gates* that the writer measured the full stdlib table. It is harmless
today because row 1 established the bake-time probe set is user-independent (30964 on every
program) and because var ids do not reach output, but both are properties of the corpus, not of the
mechanism. Worth a guard or a note at the call site.

### Row 1 stands

Seven programs, six doors, **0 hits** each; `wat-owned probes = 30964` invariant across all of
them, with `distinct-bodies` growing 2177 → 2178–2187 as the *user's* bodies join. That invariance
is the real content — it is what makes a bake-time verdict transferable to a different user
program — and it reproduced my own s3 numbers (31009 / 2177) exactly.
