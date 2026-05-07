# Arc 157 — Substrate BRIEF (slice 1a-ii)

**Drafted 2026-05-07.** Slice 1a-ii of arc 157 — the relaxation
layer on slice 1a-i's strict-default redef discipline.

User direction (verbatim, captured in DESIGN.md):

> *"i want this to be user choosable... 2 config fields ...
> (:wat::config::set-redef! true) and
> (:wat::config::set-eval-redef! true) ... the redef must satisfy
> two constraints - the signature must not change - input args+type
> must be identical, the ret type must be the same"*

> *"the opinionated defaults are set-redef and set-eval-redef to
> false - users must opt into redefs"*

## Workspace state pre-spawn

- HEAD: `b10e998` (arc 157 slice 1a-i shipped end-to-end)
- Working tree: clean (verify `git status -s` returns nothing)
- Pre-baseline (verified): 2024 passed / 0 failed (2010 baseline +
  14 arc 157 slice 1a-i tests)

## Goal

Layer the redef-discipline relaxation onto slice 1a-i's strict-
default. Three coordinated pieces, all small:

1. Two SymbolTable carrier bools (`redef_allowed`,
   `eval_redef_allowed`), both default `false`.
2. Two `:wat::config::set-*!` primitives that toggle them.
3. Gating logic at the existing `DefRedefForbidden` fire site:
   - if flag is off → fire `DefRedefForbidden` (current 1a-i path)
   - if flag is on + types match → permit redef
   - if flag is on + types differ → fire `DefRedefTypeChange`

Type-stability is mandatory whenever redef happens — opt-in to
redef does NOT opt-out of type-stability. The contract downstream
callers depend on stays intact.

## Substrate edits

### `src/runtime.rs`

**SymbolTable carrier additions** (per
`feedback_capability_carrier.md`):

```rust
/// Arc 157 slice 1a-ii — controls compile-time / load-time `def`
/// redef. Default `false` (opt-in). Toggled via
/// `(:wat::config::set-redef! true)`. Type-stability check applies
/// whenever redef happens, regardless of flag value.
pub redef_allowed: bool,

/// Arc 157 slice 1a-ii — controls eval-time `def` redef
/// (interactive `eval-ast!` flow). Default `false` (opt-in).
/// Toggled via `(:wat::config::set-eval-redef! true)`.
/// Type-stability check applies.
pub eval_redef_allowed: bool,
```

Default values via existing `Default` impl: `false` for both.

**Config setter primitives** (mirror existing `set-capacity-mode!` /
`set-presence-sigma!` shape — sonnet finds the existing pattern in
`src/sigma.rs` or wherever `set-capacity-mode!` lives):

- `:wat::config::set-redef!` — accepts one `:wat::core::bool`,
  returns `:wat::core::Unit`. Side-effect: sets
  `redef_allowed` on the SymbolTable.
- `:wat::config::set-eval-redef!` — same shape; sets
  `eval_redef_allowed`.

### `src/check.rs`

**New CheckError variant:**

`DefRedefTypeChange { name: String, prior_type: String,
new_type: String, prior_loc: SourceLocation,
current_loc: SourceLocation }` — fires when redef permitted by
flag but the new expr's type doesn't equal the prior registered
type. Display message names both types and explains
type-stability is mandatory regardless of flag.

**Gating logic at the def-collision site:**

The existing `DefRedefForbidden` fire site in slice 1a-i is the
gate. Wrap it:

```rust
// pseudo-code
if defined_values.contains_key(name) {
    let prior = defined_values.get(name);
    if !env.redef_allowed_at_check_time() {
        // 1a-i path
        emit DefRedefForbidden { name, prior_loc, current_loc };
    } else if prior.type != inferred_type {
        // 1a-ii: type-stability violation
        emit DefRedefTypeChange { name, prior.type, inferred_type,
                                  prior_loc, current_loc };
    } else {
        // 1a-ii: opt-in redef + types match
        // permit; replace prior entry
        defined_values.insert(name, (inferred_type, span));
    }
}
```

**Where does the check-time flag live?**

Two candidates:
- (a) Read from `SymbolTable.redef_allowed` directly (CheckEnv
  has access to the SymbolTable via existing path).
- (b) Mirror the flag onto CheckEnv (`CheckEnv::redef_allowed`)
  populated at check-start from the SymbolTable.

Sonnet picks per existing precedent; document in report.

**Config-setter handling at check time:**

`:wat::config::set-redef!` is itself a top-level form. When the
type-checker encounters it, the check needs to know the bool
argument's value (compile-time-knowable since it's a literal).
Two-pass options:
- (a) First pass scans for set-redef! literals; second pass
  type-checks defs with the resolved flag.
- (b) Single pass; the set-redef! arm updates the env's
  redef_allowed in-line so subsequent defs see the new value.

Option (b) is the natural fit (program-order semantics) — but
sonnet decides if it works with the existing CheckEnv mutability.

### `src/runtime.rs` (eval-time)

**Eval-time gating:**

Slice 1a-i ships runtime binding via `register_runtime_defs` at
freeze step 9.5 (mutates `SymbolTable.runtime_def_values`). For
slice 1a-ii, the eval-time path means: code introduced via
`eval-ast!` (or similar runtime AST eval) that contains a `def`
form. Currently, the eval arm at `runtime.rs:2585-2602` returns
`Value::Unit` and discards — eval-time `def` is effectively a
no-op, which means there's no runtime collision to gate.

**If eval-time `def` registration isn't currently wired** (i.e.,
the only def-binding path is the freeze-time
`register_runtime_defs`), then `set-eval-redef!` has no effective
runtime path to gate yet. Two options:

- (a) Wire eval-time `def` binding: extend the
  `runtime.rs:2585-2602` eval arm to populate
  `runtime_def_values` (with mutability, requires the SymbolTable
  to be mutable from eval — possibly through `Arc<Mutex<_>>` or a
  thread-local pattern). Then gate via `eval_redef_allowed`.
- (b) Ship `set-eval-redef!` carrier + setter as scaffolding;
  leave the gate stubbed with a `// TODO: wire when eval-time
  def-eval is supported` comment AND a CheckError that explains
  it's not active. Defer the wiring to a future arc when an
  `eval-ast!`-based caller surfaces.

**STOP signal for sonnet:** if option (a) requires substantive
mutability infrastructure (Mutex / RwLock / Cell), STOP and
report. Per memory `feedback_zero_mutex.md`, mutating SymbolTable
mid-eval is a substrate-architectural question, not a slice 1a-ii
question. Default to option (b) if option (a) needs mutability
infrastructure that doesn't exist.

**Acceptable scope-out for option (b):** the
`set-eval-redef!` flag is functional on the SymbolTable but
ungated at runtime because eval-time `def` binding is not yet
wired. A new arc opens IFF a caller actually surfaces wanting
eval-time def redef.

### NEW tests in `tests/wat_arc157_def.rs`

5 new tests after the existing 14:

15. **Redef with default flags off, basic type:** redef of
    `:a` from `1` to `2` without flag set → fires
    `DefRedefForbidden` (existing 1a-i path; sanity that opt-in
    default still holds).
16. **Redef with `set-redef!` + same type:**
    `(:wat::config::set-redef! :wat::core::true)` then
    `(:wat::core::def :a 1) (:wat::core::def :a 2)` → succeeds;
    runtime resolution shows `:a` = 2.
17. **Redef with `set-redef!` + different type:**
    `(:wat::config::set-redef! :wat::core::true)` then
    `(:wat::core::def :a 1) (:wat::core::def :a "hello")` →
    fires `DefRedefTypeChange` naming `:wat::core::i64` and
    `:wat::core::String`.
18. **`set-redef!` flag false → strict default holds.**
    Explicit `(:wat::config::set-redef! :wat::core::false)` then
    redef → `DefRedefForbidden` (verifies the flag actually
    gates rather than being always-on after first set).
19. **`set-eval-redef!` flag wires to carrier.** A test that
    just verifies the `set-eval-redef!` form is registered and
    accepted at top-level (form executes without check error).
    Behavior gating may be scope-out per "eval-time scope-out"
    above; this test verifies the surface lands.

## Constraints

- **Substrate-only edits.** Likely 3 files: `src/check.rs`,
  `src/runtime.rs`, `src/special_forms.rs`. Plus
  `tests/wat_arc157_def.rs` (extend, not new file).
- **DO NOT COMMIT.** Working tree stays modified. Orchestrator
  commits after scoring.
- **Pre-existing 2024 baseline must stay green.** The 5 new tests
  add to the count.
- **STOP at unexpected red.** Distinguish:
  - **Expected:** new tests in `wat_arc157_def.rs` exercising
    gating + type-stability.
  - **Unexpected:** existing tests breaking; substrate panic.
- **No grinding.** No speculative scope expansion.
- Time-box: **45 min wall-clock** (2× predicted upper-bound 22 min).

## Pre-flight crawl (mandatory)

1. `docs/arc/2026/05/157-core-def-form/DESIGN.md` — § Re-binding
   discipline (Q2)
2. `docs/arc/2026/05/157-core-def-form/BRIEF-SLICE-1a-i.md` — for
   the 1a-i context
3. The 1a-i slice's commit `b10e998` — read `git show b10e998
   --stat` to see what landed
4. `src/check.rs` — find the `DefRedefForbidden` fire site (added
   in 1a-i). The gate sits there.
5. `src/runtime.rs::register_runtime_defs` (added in 1a-i) — the
   freeze-time path that wires runtime bindings.
6. `src/runtime.rs::SymbolTable` — find the runtime_def_values
   field (slice 1a-i) for the carrier-add precedent.
7. `src/sigma.rs` or wherever `set-presence-sigma!` is registered
   — closest precedent for a `:wat::config::set-*!` setter.
8. `src/runtime.rs` — locate `set-capacity-mode!` arm for setter
   shape.
9. Existing tests `tests/wat_arc157_def.rs` (14 tests).
10. `feedback_capability_carrier.md` (memory) — carrier discipline.
11. `feedback_zero_mutex.md` (memory) — STOP signal for the
    eval-time path if it requires Mutex.

## Pre-flight verification

```bash
cd /home/watmin/work/holon/wat-rs
cargo test --release --workspace 2>&1 | grep -E "FAILED|^test result" | tail -5
```

Confirms 2024 / 0 baseline.

## Verification (after edits)

```bash
cargo test --release --test wat_arc157_def 2>&1 | tail -10
```

Expect: 19/19 (14 prior + 5 new) pass. If test 19 (set-eval-redef!
behavior) is scoped out per the eval-time stop signal, expect
18/19 with sonnet's STOP report explaining.

```bash
cargo test --release --workspace 2>&1 | grep -E "test result" | tail -5
```

Expect: 2024 + 5 = 2029 pass; 0 FAILED.

## Reporting (~200 words)

- Carrier additions: 2 new bool fields; their default values
- Setter primitives: registration in special_forms.rs + eval arm
  shape (mirror of which precedent)
- Gating site: where in check.rs the gate sits; what the precedence
  is (flag check → type-stability check → permit/replace)
- Type-stability check: how prior type is compared to inferred
  type; what diagnostic the `DefRedefTypeChange` Display shows
- Eval-time gating: did sonnet wire option (a) or scope-out per
  option (b)? If scope-out, the test #19 behavior assertion
  becomes a "form is recognized" check
- Workspace count: was 2024, now should be 2029 (or 2028 if test
  #19 scoped-out)
- Honest deltas: especially around CheckEnv-vs-SymbolTable flag
  read path; any surprise in the set-redef! single-pass-vs-two-
  pass interaction with the existing check ordering

DO NOT commit. DO NOT write a SCORE doc. Orchestrator scores
after your report.

If genuinely impossible (e.g., the gating mid-pass requires
substantive mutability infrastructure), STOP and report — do
NOT ship a workaround.

## Time-box

45 min wall-clock. ScheduleWakeup will fire at 45 min if sonnet
hasn't returned.

## Why this matters

User direction 2026-05-07 (memory `feedback_stepping_stones_proactive.md`):
*"if building stepping stones explicitly makes next steps more
tractable.. we build the stepping stones … simple steps enable
complex steps."* Slice 1a-i shipped the foundation; slice 1a-ii
is the relaxation layer that makes hot-reload-style workflows
possible without sacrificing the strict-default safety. Each
slice's verification is cleaner per the proactive stepping-stones
discipline.

After 1a-ii, arc 157's substrate work is complete. 1b (consumer
sweep, likely empty) and 2 (closure paperwork) follow.
