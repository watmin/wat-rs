# Arc 170 slice 1b — `ClosurePackage` reshape ("the fn IS the program")

## Goal

Restructure `ClosurePackage` from `{ forms, entry: String }` to
`{ prologue: Vec<WatAST>, entry_form: WatAST }`. Retire the
synthetic-name machinery (`:__closure::__pkg_<n>` counter +
wrap-in-define). The fn-form AST evaluates to a fn Value
directly; no naming required.

This slice corrects a structural deficiency in slice 1's
deliverable. Slice 1 (commit `787c977`, SCORE `bb155ed`) shipped
the closure-extraction algorithm correctly but in a public shape
that re-introduced the entry-keyword ceremony DESIGN explicitly
killed (DESIGN.md lines 102-108 + 484-509).

See [`REALIZATIONS-SLICE-1.md`](./REALIZATIONS-SLICE-1.md) for the
discipline lesson + slice 1b origin.

## Read first (in order)

1. `REALIZATIONS-SLICE-1.md` — what slice 1 got wrong + why; the
   honest shape; what changes vs what stays. **Load-bearing.**
2. `CLOSURE-EXTRACTION.md` — v2 supersedes v1; v2 algorithm +
   shape + invariants are the spec for this slice. v1 stays below
   as historical context.
3. `DESIGN.md` lines 102-108 (the settled "fn IS the program"
   framing) + lines 484-509 (the DESIGN-time conversation log)
4. `SCORE-SLICE-1.md` — immutable; documents what slice 1
   shipped against the now-retired v1 spec
5. `docs/COMPACTION-AMNESIA-RECOVERY.md` § 6 (FM 5, FM 9, FM 11)
   — discipline floor

## Branch + commit policy

- **Active branch**: `arc-170-program-entry-points` (carries
  slice 1 commit + SCORE-SLICE-1 + slice 2 BRIEF/EXPECTATIONS
  + REALIZATIONS-SLICE-1 + DESIGN amendment + CLOSURE-EXTRACTION v2)
- Multiple WIP commits + pushes welcome on the branch
- DO NOT push to main; orchestrator merges atomic to main as
  one squash commit after slice 5 closure paperwork ships

## Substrate edits

### 1. Reshape `src/closure_extract.rs`

**`ClosurePackage` shape:**

```rust
// before (v1 — slice 1)
pub struct ClosurePackage {
    pub forms: Vec<WatAST>,
    pub entry: String,
}

// after (v2 — slice 1b)
pub struct ClosurePackage {
    pub prologue: Vec<WatAST>,
    pub entry_form: WatAST,
}
```

**`extract_closure` algorithm changes:**

- **Entry resolution (algorithm step 1)**:
  - For inline-lambda input: reconstruct the fn-form AST from
    the fn Value's params + body + ret_type. Set `entry_form` to
    that fn-form AST. Do NOT synthesize a name. Do NOT wrap in a
    define. Do NOT generate `:__closure::__pkg_<n>`.
  - For keyword-path input: set `entry_form` to a Symbol AST
    naming the keyword (`HolonAST::symbol(":my::worker")` or
    equivalent). The user's existing define for that symbol stays
    in `prologue` as a dep (it's already extracted as a user dep
    via the existing dep-closure walker).

- **Assembly (algorithm step 6)**:
  - `prologue` contains: type defs → capture binding defines →
    user dep defines (in topological order). Does NOT include
    the entry fn as a trailing define (whether canonical or
    synthetic).
  - For inline-lambda input: the fn-form AST goes ONLY in
    `entry_form`, not anywhere in `prologue`.
  - For keyword-path input: the user's existing define
    `(:wat::core::define :my::worker (fn ...))` is in
    `prologue` as a regular user dep (no special treatment);
    `entry_form` is the Symbol that resolves to it.

- **Synthetic-name counter retires**: the `__closure::__pkg_<n>`
  counter machinery (whatever its current implementation site
  in `closure_extract.rs`) is removed. The capture-binding name
  generator (e.g., `__captured_X`) STAYS — that's a different
  naming concern (avoiding collision with extracted symbols),
  not entry naming.

- **Body rewrite for captured locals** stays unchanged. The
  rewrite operates on the fn Value's body BEFORE it's emitted as
  `entry_form`; the rewritten body is what evaluates to a fn Value
  with the right capture references.

### 2. Update `tests/wat_arc170_closure_extraction.rs`

T1-T15 stay structurally (same test scenarios). Assertion
updates:

- Replace assertions on `pkg.entry` (String) with assertions on
  `pkg.entry_form` (WatAST shape):
  - **Inline-lambda inputs (T2 factory, T5/T6 with-captures, etc.)**:
    assert `pkg.entry_form` is a fn-form AST. Match shape:
    `(fn [<params>] -> <ret-type> <body>...)`. Verify params +
    ret-type + body match the input fn's signature.
  - **Keyword-path inputs (T1 top-level defn)**:
    assert `pkg.entry_form` is a Symbol AST whose name matches
    the input keyword.

- Replace assertions on `pkg.forms` (Vec<WatAST>) with
  assertions on `pkg.prologue` (Vec<WatAST>):
  - The trailing form (entry's define) is GONE from prologue
  - Type defs + capture binding defines + user dep defines all
    stay in prologue (same shape as v1's forms minus the entry)

- Add a NEW behavior-equivalence pattern (replaces v1's
  "freeze + apply_function on entry name"):
  ```rust
  let pkg = extract_closure(&fn_value, &parent_sym, &parent_types)?;
  let frozen = startup_from_forms(pkg.prologue, ...)?;
  let fn_value = eval(&pkg.entry_form, &env, frozen.symbols())?;
  let result = apply_function(fn_value, args, frozen.symbols(), span)?;
  // assert result matches the parent-world invocation
  ```

- T1-T15 should all still pass post-reshape with updated
  assertions. The underlying algorithm is correct; only the
  output shape changes.

### 3. Drop synthetic-name assertions in unit tests

The 2 in-module unit tests slice 1 added (synthetic-name
uniqueness + capture-name prefix) — review:

- **Synthetic-name uniqueness**: this test no longer applies.
  Drop it.
- **Capture-name prefix**: STAYS. Capture-binding name
  generation (e.g., `__captured_X`) is unchanged.

Net unit-test count: 1 (down from 2). Net integration-test count:
15 (T1-T15, unchanged). Total tests added by arc 170: 16.

### 4. CLOSURE-EXTRACTION.md is already amended

CLOSURE-EXTRACTION.md v2 is in place (see commit on branch). Slice
1b implements against v2 § "v2 — corrected algorithm + shape." No
further doc changes in this slice.

## What slice 1b does NOT do

- **No spawn-process consumer work** — that's slice 2's territory.
  Slice 1b ships the corrected substrate primitive; slice 2 wires
  it.
- **No `:user::main` signature changes** — also slice 2.
- **No wat-level callers added** — slice 1b stays Rust-internal.
- **No SCORE-SLICE-1.md edits** — immutable historical record per
  `feedback_inscription_immutable.md`. Slice 1b's SCORE
  documents the corrected ship; slice 1's SCORE stays as
  historical record of the deficiency that surfaced.

## Honest delta categories (if surfaced, report; don't bridge)

- **Q-impl-2 captured-fn-value gap (slice 1 honest delta A)**.
  Still applies. If slice 1b's reshape causes a real consumer to
  hit `Value::wat__core__fn` encoding for a CAPTURED value (not
  the input fn itself), surface as honest delta. Slice 1b is NOT
  the place to implement closure-of-closure recursive
  sub-extraction; that's a separate substrate-extension if the
  consumer demands it.
- **Value-kind encoding gaps (slice 1 honest delta C)**. Still
  applies (HolonAST, WatAST, RustOpaque, holon::Vector, Instant,
  Duration return `Internal` error).
- **fn Value → fn-form AST reconstruction**. The new approach
  needs to reconstruct the fn-form AST from the fn Value's
  params + body + ret_type. The fn Value's `Function::body`
  carries the body AST. The params + ret_type need to be
  re-emitted as the fn-form's signature shape. If the substrate
  doesn't have a clean Function-Value-to-fn-form-AST helper,
  surface as honest delta — the work is mechanical AST
  construction; should not require new substrate.
- **Body-rewrite ordering vs entry_form emission**. Slice 1's
  body rewrite operates on the AST inside the synthesized
  define. Slice 1b emits the (rewritten) AST as `entry_form`
  directly. Verify the rewrite happens BEFORE entry_form is
  set, not after. If the implementation needs restructuring,
  surface.
- **FM 5 trap.** If a TODO is tempting, STOP. Surface.

## Predicted runtime

60-120 minutes (opus). Time-box hard cap at 240 minutes.

Smaller scope than slice 1 (90-180 min) because:
- Algorithm stays unchanged (free-symbol walker, dep closure,
  portability check, topo sort all already correct)
- Spec doc CLOSURE-EXTRACTION.md v2 already drafted (this slice
  doesn't draft spec, only implements against it)
- Tests stay structurally; only assertions update
- The reshape work is localized: `ClosurePackage` shape,
  entry resolution branch (1 site), assembly assembly (1 site),
  test assertions

Larger scope than a "trivial reshape" because:
- The body-rewrite path needs to produce the rewritten AST as
  `entry_form` instead of as a wrapped define
- The fn-Value → fn-form-AST reconstruction step is new
  (reconstruct `(fn [params] -> :T body)` from `Function`
  components)
- All T1-T15 assertions need careful update (assertion shape
  changes, not just value updates)

## Branch state at slice 1b start

```
$ git log --oneline -8
6be0383 (HEAD -> arc-170-program-entry-points)
   arc 170 slice 2: BRIEF + EXPECTATIONS corrected — arc 168 precedent
4d419e9  arc 170 slice 2: BRIEF + EXPECTATIONS authored
bb155ed  arc 170 slice 1: SCORE — 14/14 rows pass, Mode A clean
787c977  arc 170 slice 1: Rust closure extraction substrate primitive
ae52ff7  arc 170 slice 1 BRIEF + EXPECTATIONS: Rust closure extraction
71ac618  arc 170 DESIGN v5 + CLOSURE-EXTRACTION.md: final scope
... (DESIGN drafts)
```

`cargo test --workspace` baseline at slice 1b start: `passed: 2108
failed: 0`.

(After this BRIEF + REALIZATIONS-SLICE-1 + DESIGN amendment +
CLOSURE-EXTRACTION v2 ship as commits, the count is unchanged —
docs only.)

## SCORE artifact

After slice 1b ships green, orchestrator writes SCORE-SLICE-1B.md
with the corrected scorecard (including the **DESIGN-intent
alignment row** that slice 1's scorecard missed). You report to
chat; orchestrator owns the SCORE artifact + commit.
