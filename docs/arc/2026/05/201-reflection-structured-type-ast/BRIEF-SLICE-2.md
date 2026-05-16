# Arc 201 Slice 2 BRIEF — General-purpose HolonAST accessors

**Phase:** Second slice of arc 201. Slice 1 (commit `0706949`) shipped structured type-AST emission. Slice 2 mints the accessors that let macros walk the structured HolonAST.

**Originating signal:** macros that consume reflection (D2 / run-threads being the immediate consumer) need to walk `HolonAST::Bundle` children and extract `HolonAST::Atom` wrapped values. Today the substrate has CONSTRUCTORS (`:wat::holon::Atom`, `:wat::holon::Bundle`, `:wat::holon::Bind`, `:wat::holon::Permute`, ...) but NO general-purpose accessors.

## Goal

Mint general-purpose accessors on HolonAST:

- `:wat::holon::Bundle/children bundle -> :wat::core::Vector<wat::holon::HolonAST>` — returns the per-child HolonAST sequence (errors if input isn't Bundle)
- `:wat::holon::Bundle/head bundle -> :wat::holon::HolonAST` — returns the first child (errors if Bundle is empty or input isn't Bundle)
- `:wat::holon::Atom/value atom -> ???` — returns the wrapped value (sonnet decides return type: maybe `:wat::core::HolonAtom`, maybe `:wat::holon::HolonAST` wrapping a leaf, maybe a polymorphic shape via dispatch; investigate the HolonAST::Atom variant's payload and pick the cleanest surface)

These are **general-purpose**, not purpose-specific. Any consumer that wants to walk HolonAST sequence structure or unwrap atomic leaves can use these — not just signature reflection.

## Naming

The proposed names (`Bundle/children`, `Bundle/head`, `Atom/value`) are working drafts. Sonnet picks final names. If anything reads off — alternatives:
- `Bundle/children` vs `Bundle/items` vs `Bundle/parts` — pick what reads as "the structured sequence components"
- `Bundle/head` vs `Bundle/first` — pick what mirrors existing wat conventions (look at `:wat::core::first` precedent)
- `Atom/value` vs `Atom/unwrap` vs `Atom/payload` — depends on what the Atom variant actually wraps

If genuinely uncertain, run `/gaze` style inline (write all candidates with four-questions YES/NO each) and pick. Surface in SCORE.

## What to check before minting

- `HolonAST::Atom` variant payload shape — what TYPE is wrapped? Per src/runtime.rs lines around `wat__core__keyword` + `holon_to_watast` — probably a scalar/atomic value. The accessor's return type depends on this.
- `HolonAST::Bundle(Vec<HolonAST>)` — is that the actual shape? Confirm.
- Existing similar primitives — does any sibling already accept HolonAST as input and decompose? e.g., `:wat::holon::*` constructors. If accessor-shaped sibling already exists in some form, surface (don't duplicate).
- Per `feedback_no_new_types`: this slice ADDS new substrate verbs (the accessors); that's the slice's job. NOT new types/structs.

## Tasks

### 1. Investigate HolonAST shape

Read the HolonAST enum definition (likely in holon-rs or src/runtime.rs). Confirm:
- `Bundle(Vec<HolonAST>)` shape
- `Atom(...)` payload type
- Any existing accessor-shaped functions

Surface findings in SCORE.

### 2. Mint the accessors

For each accessor:
- Add `register` call in `src/check.rs` (type scheme)
- Add eval handler in `src/runtime.rs` with dispatch arm
- Mirror the construction patterns from existing `:wat::holon::Atom`, `:wat::holon::Bundle` registrations

Per `feedback_no_new_types`: no new types/structs/special-forms. These are new VERBS that operate on existing HolonAST.

### 3. Tests

`tests/wat_arc201_holon_ast_accessors.rs` (or sonnet picks name):

- `bundle_children_returns_vec_of_holonast`
- `bundle_children_errors_on_non_bundle` (Atom input → TypeMismatch)
- `bundle_head_returns_first_child`
- `bundle_head_errors_on_empty_bundle`
- `bundle_head_errors_on_non_bundle`
- `atom_value_returns_wrapped_value` (whatever shape Atom wraps)
- `atom_value_errors_on_non_atom`

Use the slice 1 structured emission as the source of HolonAST values to test against — e.g., construct a Bundle via `signature-of` then walk it via the accessors.

### 4. Build + test

```bash
cd /home/watmin/work/holon/wat-rs
cargo build --release --workspace --tests
cargo test --release -p wat --test wat_arc201_holon_ast_accessors  # (or chosen name)
cargo test --release -p wat --test wat_arc201_structured_signature_types  # ensure slice 1 still green
cargo test --release --workspace --no-fail-fast
```

Workspace baseline: failures ≤ baseline (commit `0706949` — 4 stable failures + lifeline flake).

### 5. SCORE

Write `docs/arc/2026/05/201-reflection-structured-type-ast/SCORE-SLICE-2.md` per § SCORE methodology + § Honest deltas.

## STOP triggers (true emergencies — surface, do not paper over)

1. **HolonAST::Atom payload doesn't fit a clean accessor signature** — e.g., the wrapped value is a Rust-internal scalar that doesn't lift cleanly to a wat type. Surface what you found; consider whether the accessor needs to be polymorphic via dispatch (slice 2 can ADD a dispatch entry if needed — that's still a verb addition, not a type addition).
2. **Naming-related ambiguity** that genuinely resists `/gaze` resolution — surface and ask
3. **An accessor-shaped sibling already exists** — surface; don't duplicate; reuse if appropriate
4. **Workspace baseline regresses** — STOP, surface the new failure
5. **Any urge to mint a new substrate TYPE / STRUCT / SPECIAL FORM** — STOP. This slice adds verbs (accessors), not new substrate types.

## HARD constraints

- DO NOT commit. Orchestrator commits atomically after independent verification.
- **cwd discipline:** FIRST action: `cd /home/watmin/work/holon/wat-rs/`; verify with `pwd`. Harness may launch into `.claude/worktrees/agent-<id>/` — ignore it; operate on the real repo per `docs/COMPACTION-AMNESIA-RECOVERY.md` § 7-bis.
- DO NOT mint any new substrate type, struct, or special form. New VERBS (accessors) are the slice's purpose.
- DO NOT modify slice 1 work (`type_expr_to_ast`, `parse_type_slot`, signature builders).
- DO NOT touch arc 117/133 sibling-binding walker.
- DO NOT touch INSCRIPTIONs / past SCOREs / DEFERRAL-VIOLATIONS / SUPERSEDED BRIEFs / AUDIT / recovery doc / past STONE BRIEFs/EXPECTATIONS/SCOREs.
- DO NOT modify INTERSTITIAL-REALIZATIONS.md or arc 201 DESIGN.md (orchestrator owns).
- DO NOT use `--no-verify` / `--no-gpg-sign` / skip hooks.

**Macro dialect (Clojure-style; confirmed arc 199 rejection):**
- `~` = unquote
- `~@` = unquote-splicing
- `,` = whitespace literal

## SCORE methodology

5 rows YES/NO + evidence:

| Row | What | Evidence |
|-----|------|----------|
| A | `:wat::holon::Bundle/children` minted with type scheme + eval handler + dispatch arm | grep + unit test demonstrates the verb returns Vec<HolonAST> from a Bundle |
| B | `:wat::holon::Bundle/head` minted similarly | grep + unit test |
| C | `:wat::holon::Atom/value` minted similarly | grep + unit test |
| D | All accessors error cleanly on wrong input shape (TypeMismatch or equivalent) | error-case tests pass |
| E | Workspace test failure count ≤ baseline (4) | full workspace cargo test failures ≤ baseline + flake variance |

## Honest deltas to capture in SCORE

- Final names chosen for each accessor (vs proposed working drafts)
- HolonAST::Atom payload shape (what does the accessor actually return?)
- Did any sibling accessor already exist? (Confirms uniqueness of new verbs)
- Any naming `/gaze` exchanges
- Workspace baseline preserved exactly?

## Time-box

30-60 min predicted. Hard stop 90 min.

## Workspace baseline (commit `0706949`)

- `cargo build --release --workspace --tests`: clean
- `cargo test --release --workspace --no-fail-fast`: 4 stable failures + lifeline flake variance

Post-slice-2 target:
- ≥ baseline + 7 new passes (accessor tests)
- ≤ baseline failures (purely additive)

## On completion

1. Write SCORE-SLICE-2.md per § SCORE methodology + § Honest deltas.
2. Return final summary to orchestrator: rows passed/failed + workspace delta + path to SCORE + naming decisions + Atom payload shape findings.

You are launching now. T-minus 0.
