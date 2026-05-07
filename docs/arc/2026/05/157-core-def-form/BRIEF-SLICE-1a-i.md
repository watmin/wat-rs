# Arc 157 — Substrate BRIEF (slice 1a-i)

**Drafted 2026-05-07.** Slice 1a-i of arc 157.

User direction (verbatim): *"let's introduce a new form…
(:wat::core::def :some-name :some-value)"* + *"clojure is our
guiding light - we're just building a strongly typed clojure on
rust"* + *"def should only be declared once."*

User direction on slicing (proactive stepping stones, recovery doc
§ 5): *"if building stepping stones explicitly makes next steps
more tractable.. we build the stepping stones … simple steps
enable complex steps."* Slice 1a was split into 1a-i (this brief,
def + position) and 1a-ii (redef config + gating, follow-up).

## Workspace state pre-spawn

- HEAD: `1b27855` (arc 155 closure shipped; arc 156 backed out)
- Working tree: arc 157 directory uncommitted with DESIGN.md +
  BRIEF-SLICE-1a-i.md + EXPECTATIONS-SLICE-1a-i.md (this commit).
- Pre-baseline (verified): `cargo test --release --workspace` =
  **2010 passed / 0 failed**.

## Goal — slice 1a-i: foundation only

Mint `:wat::core::def` as a top-level value-binding special form.
Default behavior: **strict — error on every redef collision**. No
config flags exist yet; that's slice 1a-ii.

This is the stepping stone. 1a-i ships a complete, useful form on
its own (def with strict-default redef-error). 1a-ii will layer
opt-in gating around the foundation 1a-i ships.

Three coordinated pieces (all small):

1. **`:wat::core::def`** — special form binding `:name` to the
   result of evaluating `<expr>`. Type inferred from `<expr>`.
2. **Position predicate** — recursive top-level rule: file form
   list, top-level `do`, top-level `let` body all splice; nothing
   else does.
3. **`defined_values` carrier** on SymbolTable — maps name →
   (registered type, source location). Slice 1a-ii will gate
   writes to this map; for 1a-i, every collision is an error.

## Substrate edits

### `src/special_forms.rs`

Register `:wat::core::def` with sketch `&["<name>", "<expr>"]`.
Add a section comment cross-referencing arc 157.

DO NOT register `:wat::config::set-redef!` or
`:wat::config::set-eval-redef!` — those land in slice 1a-ii.

### `src/runtime.rs`

**SymbolTable carrier addition** (per
`feedback_capability_carrier.md`):

```rust
pub struct SymbolTable {
    // ... existing fields ...

    /// Arc 157 — names bound via `:wat::core::def`. Maps name →
    /// (registered type, source location of binding). Used for:
    /// - resolving keyword references at type-check
    /// - rejecting redef in slice 1a-i (every collision = error)
    /// - slice 1a-ii will add opt-in gating + type-stability
    pub defined_values: HashMap<String, (TypeScheme, SourceLocation)>,
}
```

Default values in `SymbolTable::new()` / `Default` impl: empty
HashMap.

**Eval arm — `:wat::core::def`:**
- Evaluate `<expr>` in current env.
- Look up `<name>` in `defined_values`. If present → runtime
  error (`DefRedefForbidden`) naming prior location. (No gating
  yet; every collision errors.)
- Bind `<name>` → value in module's value env.
- Register `(name, type, span)` in `defined_values`.

DO NOT add `redef_allowed` / `eval_redef_allowed` fields — those
land in slice 1a-ii.

### `src/check.rs`

**New CheckError variants:**

1. `DefNotTopLevel { wrapper: String, span: SourceLocation }` —
   fires when `def` is found inside a non-splice form. Display
   names the offending wrapper; mentions `let` for nested local
   binding.
2. `DefRedefForbidden { name: String, prior_loc: SourceLocation,
   current_loc: SourceLocation }` — fires when a `def` finds the
   name already bound. Display message can be slice-1a-i-specific
   ("redef forbidden; opt-in flag lands in arc 157 slice 1a-ii")
   or generic ("name already bound at <prior>"). Sonnet picks
   per the discipline already in `docs/SUBSTRATE-AS-TEACHER.md`.

(NO `DefRedefTypeChange` variant — type-stability lands in 1a-ii.)

**Position predicate** (recursive top-level rule):

```rust
fn is_top_level_position(path: &[FormParent]) -> bool {
    // path = parent chain, root-most last (or first — match
    // the existing convention in check.rs)
    // The form is at top-level position iff every parent is one of:
    //   FileRoot
    //   :wat::core::do form (whose own parent is also top-level)
    //   :wat::core::let body (whose own parent is also top-level)
    // Anything else (if/cond/match/and/or/Result-try/Option-try/
    //   fn body/define body/struct/enum/etc.) → false
    path.iter().all(|parent| matches!(parent,
        FormParent::FileRoot
        | FormParent::Do
        | FormParent::LetBody
    ))
}
```

(Adapt to the actual parent-tracking shape in `check.rs` —
sonnet crawls `infer_let` / `infer_do` to see how parent context
is threaded. The exact enum / data structure name is sonnet's
call; the SHAPE is the rule above.)

**Inference / position-check arm for `def`:**

1. Verify `<name>` is a keyword. Reject otherwise with
   `DefNonKeywordName` (or reuse an existing
   "name must be keyword" CheckError if one already exists —
   sonnet picks).
2. Position check: walk up to root via parent chain. If any
   non-splice ancestor → emit `DefNotTopLevel` naming the first
   offending wrapper.
3. Infer type of `<expr>`.
4. Lookup `<name>` in `defined_values`:
   - If absent → register `(name, inferred_type, span)`.
   - If present → emit `DefRedefForbidden`. (No gating; every
     collision errors. 1a-ii relaxes this.)

**Reference resolution:**

Subsequent keyword references must resolve to `defined_values`
when the name was bound via `def`. Likely lives in the existing
keyword-lookup path; sonnet adapts. Honest delta if integration
is non-trivial.

### `src/freeze.rs`

Add `:wat::core::def` to the top-level recognition list (alongside
`:wat::core::define`, `:wat::core::defmacro`,
`:wat::core::define-dispatch`, `:wat::core::struct`,
`:wat::core::enum`, etc.). Mirror existing pattern.

### NEW `tests/wat_arc157_def.rs`

Harness shape per `tests/wat_arc154_kill_let_star.rs` /
`tests/wat_arc155_fn_rename.rs`.

11 tests covering 1a-i scope:

**Basic binding (4 tests):**
1. `(:wat::core::def :pi 3.14159)` — binds; subsequent reference
   resolves to value 3.14159 with type `:wat::core::f64`.
2. Computed value: `(:wat::core::def :a 1) (:wat::core::def :b
   (:wat::core::+ :a 1))` — `:b` = 2; types propagate.
3. Type registered: subsequent reference type-checks against the
   inferred type (e.g. binding `:pi : f64` then using `:pi` in an
   `:i64`-context fails with TypeMismatch).
4. Type error in expr: `(:wat::core::def :bad
   (:wat::core::+ "x" 1))` surfaces at the def site.

**Position rule — legal (4 tests):**
5. `(:wat::core::def :a 1)` at literal top-level → succeeds.
6. `(:wat::core::do (:wat::core::def :a 1) (:wat::core::def :b
   2))` at top-level → both registered; splice works.
7. `(:wat::core::let [config 42] (:wat::core::def :get-config
   (:wat::core::fn [] config)))` at top-level → `:get-config`
   registered as a closure capturing `config`; calling it returns
   42.
8. `(:wat::core::let [x 1] (:wat::core::do (:wat::core::def :a
   x)))` at top-level → recursive splice (let containing do
   containing def) works; `:a` registered as 1.

**Position rule — illegal (3 tests):**
9. `(:wat::core::if cond (:wat::core::def :a 1)
   (:wat::core::def :b 2))` at top-level → rejected with
   `DefNotTopLevel` naming `:wat::core::if`.
10. `(:wat::core::define (:my::f -> :wat::core::Unit)
    (:wat::core::def :a 1))` → rejected; wrapper named
    `:wat::core::define` body.
11. Two `(:wat::core::def :a 1)` forms in a row → second fires
    `DefRedefForbidden` naming the first's location.
    (No gating in 1a-i; type-stability + opt-in test cases land
    in 1a-ii's test suite.)

If sonnet finds reference-resolution integration non-trivial
(test #1's bare `:pi` lookup interacting with the existing
keyword-resolution chain), STOP at first red on tests 1-4 and
report — orchestrator decides whether scope expands or 1a-i
ships partial.

## Constraints

- **Substrate-only edits.** EXACTLY 5 files: `src/check.rs`,
  `src/runtime.rs`, `src/special_forms.rs`, `src/freeze.rs`,
  NEW `tests/wat_arc157_def.rs`. NO consumer wat edits. NO
  other crate.
- **DO NOT COMMIT.** Working tree stays modified. Orchestrator
  commits once 1a-i scorecard verifies clean.
- **`def` is NEW** — workspace SHOULD NOT break. Pre-existing
  baseline (2010+ green) MUST stay green; the 11 new tests add
  to the count.
- **STOP at unexpected red.** Distinguish:
  - **Expected:** new tests in `wat_arc157_def.rs`.
  - **Unexpected:** pre-existing test breaks, substrate panic,
    parse error.
- **No grinding.** No speculative scope expansion (no config
  setters, no gating, no type-stability — those are 1a-ii).
- Time-box: 60 min wall-clock (2× predicted upper-bound 30 min).

## Pre-flight crawl (mandatory)

1. `docs/arc/2026/05/157-core-def-form/DESIGN.md` — full read.
   Pay especially close attention to § Scope (Q1) — Clojure
   top-level rule and the recursive splice predicate.
2. `docs/arc/2026/05/154-kill-let-star/INSCRIPTION.md` — closest
   precedent for adding/modifying a top-level special form.
3. `docs/arc/2026/05/155-fn-rename/INSCRIPTION.md` — multi-piece
   substrate slice pattern.
4. `docs/SUBSTRATE-AS-TEACHER.md` — diagnostic-as-migration-brief
   pattern; CheckError variant + Display discipline.
5. `feedback_capability_carrier.md` (memory) — SymbolTable
   carrier discipline; new fields land alongside existing
   carrier fields.
6. `feedback_substrate_already_typed.md` (memory) — paid-for
   lesson: don't add type annotations when inference suffices.
   Confirm: `def` does NOT take `-> :T`.
7. `src/special_forms.rs` — registration shape; section comments.
8. `src/runtime.rs::SymbolTable` struct definition.
9. `src/check.rs` parent-context-tracking pattern for the
   position predicate. The `infer_let` / `infer_do` arms are
   the closest precedent for how parent context threads.
10. `src/freeze.rs` top-level form recognition list.
11. `tests/wat_arc154_kill_let_star.rs` — test harness shape.

## Pre-flight verification

```bash
cargo test --release --workspace 2>&1 | grep -cE "FAILED"
```

Must be 0 (already verified: 2010 passed / 0 failed).

## Verification (after edits)

```bash
cargo test --release --test wat_arc157_def 2>&1 | tail -20
```

Expect: 9-11 of 11 new tests pass. Tests 1-4 may have honest
delta if reference-resolution integration is non-trivial.

```bash
cargo test --release --workspace 2>&1 | grep -E "test result|FAILED" | head -10
```

Expect: workspace count = 2010 + 9-11; FAILED = 0.

## Reporting (~250 words)

Per BRIEF: pre-flight crawl confirmation; edit summary per file
with LOC delta; verification (new test pass count + workspace
delta); path classification (Mode A / B / C); honest deltas:

- Parent-context tracking shape — does `check.rs` already thread
  parent context, or did sonnet add new infrastructure?
- Keyword-reference resolution — where in the lookup chain does
  the `defined_values` consult go? Did anything pre-existing
  depend on a different name for that map?
- `Default` impl on SymbolTable — derived or manual? Sonnet
  added field accordingly.
- Position predicate — was `let` body splice straightforward, or
  did the recursive parent-walk interact awkwardly with anything
  in `infer_let`?

DO NOT write a SCORE doc — orchestrator scores after 1a-i lands
+ atomic commit ships.

## Time-box

60 minutes wall-clock (2× predicted upper-bound 30 min).
ScheduleWakeup will fire at 60 min if sonnet hasn't returned.

## Why this matters (slice 1a-i specifically)

The user's direction: *"clojure is our guiding light - we're just
building a strongly typed clojure on rust."* Slice 1a-i ships the
foundational `def` form with the Clojure-faithful position rule
and the strict-default discipline that makes the foundation safe.
It's the stepping stone that makes slice 1a-ii (opt-in gating)
trivially small — 1a-ii operates on the SETTLED foundation 1a-i
ships, rather than introducing infrastructure AND using it
simultaneously.

Beyond arc 157, this `def` form is the first stone for the
planned `define` retirement (`define` becomes a macro
`(defn name args body) → (def :name (fn args body))`). Out of
arc 157's scope but enabled by it.
