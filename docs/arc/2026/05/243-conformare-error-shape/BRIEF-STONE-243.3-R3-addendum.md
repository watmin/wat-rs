# BRIEF — Stone 243.3 R3 addendum sweep — close 4 newly-surfaced findings

You are sonnet. Stone 243.3 R3 addendum sweep. 4 comment-rewrites closing findings surfaced honestly during the R3 sweep (per `feedback_pre_existing_is_not_exemption` — every solvable finding in stone-touched files closes; no skip-pre-existing). R3 sweep main 6 fixes already applied (uncommitted in working tree). This addendum closes the 4 additional findings before orchestrator commits Stone 243.3 atomic.

**Anchor cwd:** `/home/watmin/work/holon/wat-rs/`. Verify with `pwd`. Reject `.claude/worktrees/`.

## Pre-spawn baseline (working tree state)

- `src/types.rs` + `src/check.rs` carry the R3 sweep's 5 confirmed fixes (R3.1/R3.2/R3.3/R3.7/R3.9 LANDED; R3.8 left unchanged pending this addendum)
- Lib: 890 PASS / 0 FAIL
- tests/function: 8 / 0
- probe_arc243_stone3: 3 / 0
- Workspace test-build: clean (exit 0)
- Clippy: 897

## The 4 fixes

### R3.13 — `types.rs:30-36` stale 058 Track 2 citation removed

Current text:
```rust
//! # Scope notes
//!
//! The name-resolution pass resolves call heads; field-position type
//! references are validated at use site, not at registration time.
//! Code generation for Rust-backed compiled binaries is out of wat-rs
//! scope (058 backlog Track 2 tracks this concern).
```

The "058 backlog Track 2 tracks this concern" citation is STALE — `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/` exists but contains no "Track 2" reference. Stale tracker citation = deferral dressed as affirmation per FM 11.

**Target text (rewrite the closing sentence):**
```rust
//! # Scope notes
//!
//! The name-resolution pass resolves call heads; field-position type
//! references are validated at use site, not at registration time.
//! Code generation for Rust-backed compiled binaries is outside wat-rs
//! scope by design — the substrate compiles to its own runtime.
```

### R3.14 — `check.rs:33-46` section header + bullets present-state rewrite

Current text:
```rust
//! # What this does NOT catch (explicitly deferred)
//!
//! - **Fn-value call-site typing.** Fn values don't carry
//!   structured signatures through [`crate::runtime::Function`] yet,
//!   so calling a fn stays Unknown at the check layer.
//! - **`:Union<T,U,V>` coproduct discipline.** `:Union` is a
//!   first-class type form in the grammar; full subtype / variant
//!   checks land when stdlib needs demand them.
//! - **Typed-macro parameter checks (058-032).** Macros expand before
//!   check; macro-definition-time checks (`:AST<T>` against body
//!   positions) are future work.
//! - **Numeric promotion.** `:i64` does not promote to `:f64` statically;
//!   mixing numeric types in arithmetic is rejected.
```

Header carries "explicitly deferred" framing; 3 of 4 bullets carry deferral language ("yet", "land when stdlib needs demand them", "are future work"). Numeric promotion bullet IS already honest (THE DECISION per arc 237.8a).

**Target text:**
```rust
//! # What this does NOT catch
//!
//! - **Fn-value call-site typing.** Fn values don't carry structured
//!   signatures through [`crate::runtime::Function`]; calling a fn
//!   stays Unknown at the check layer.
//! - **`:Union<T,U,V>` coproduct discipline.** `:Union` is a
//!   first-class type form in the grammar; its check-layer surface
//!   is intentionally permissive — full subtype/variant discipline
//!   is outside the check layer by design.
//! - **Typed-macro parameter checks (058-032).** Macros expand before
//!   check; macro-definition-time checks (`:AST<T>` against body
//!   positions) are outside the check layer by design (expansion-time
//!   discipline is a separate concern if pursued).
//! - **Numeric promotion.** `:i64` does not promote to `:f64` statically;
//!   mixing numeric types in arithmetic is rejected.
```

**Strip:**
- "(explicitly deferred)" from header
- "yet" from Fn-value bullet
- "land when stdlib needs demand them" → "outside the check layer by design"
- "are future work" → "are outside the check layer by design (expansion-time discipline is a separate concern if pursued)"

### R3.15 — `check.rs:3283-3289` "Reintroduction recipe" header rewrite

Current text (immediately after retired-walker context at lines 3277-3282):
```rust
// Reintroduction recipe (if a future arc needs walker-driven
// migration on a similar single-token keyword retirement):
// match `WatAST::Keyword(s, span)` for the retired FQDN; recurse
// into `WatAST::List(items, _)` children; emit one CheckError
// per offending site. Mirror arc 153's `walk_type_for_legacy_unit_name`
// or git blame this file at the arc 154 sweep 1a commit.
```

"if a future arc needs" is forward-promise framing; content IS teaching value (walker pattern documentation). Honest framing: present-state pattern preservation.

**Target text:**
```rust
// Walker pattern (preserved per substrate-as-teacher § Pattern 3 —
// mirror this shape for similar single-token keyword retirements):
// match `WatAST::Keyword(s, span)` for the retired FQDN; recurse
// into `WatAST::List(items, _)` children; emit one CheckError
// per offending site. Mirror arc 153's `walk_type_for_legacy_unit_name`
// or git blame this file at the arc 154 sweep 1a commit.
```

### R3.16 — `check.rs:3308-3312` second "Reintroduction recipe" header rewrite

Current text (immediately after retired arc 155 lambda walker context at lines 3299-3307):
```rust
// Reintroduction recipe: see arc 153/154 walker shapes; mirror
// `WatAST::Keyword(s)` match for the retired FQDN, recurse
// into `WatAST::List(items, _)` children, emit one CheckError
// per offending site.
```

Same pattern as R3.15.

**Target text:**
```rust
// Walker pattern (preserved per substrate-as-teacher § Pattern 3 —
// mirror this shape for similar retirements): see arc 153/154 walker
// shapes; mirror `WatAST::Keyword(s)` match for the retired FQDN,
// recurse into `WatAST::List(items, _)` children, emit one CheckError
// per offending site.
```

### Additional reintroduction recipe site — check.rs:3348-3358

Search for the THIRD "Reintroduction recipe" header in check.rs (the one inside arc 159 slice 3's retired-walker context). If found at ~3348, apply the same pattern rewrite. The arc 159 recipe is longer (multi-bullet); preserve all the bullets, only rewrite the header line "Reintroduction recipe (if a future arc needs walker-driven migration on a similar inner-let-binding shape change):" → "Walker pattern (preserved per substrate-as-teacher § Pattern 3 — mirror this shape for similar inner-let-binding shape changes):"

Report in your return paragraph whether the third site was found + rewritten.

## Cadence

1. Apply R3.13 to src/types.rs:30-36
2. Apply R3.14 to src/check.rs:33-46
3. Apply R3.15 to src/check.rs:3283-3289
4. Apply R3.16 to src/check.rs:3308-3312 (+ the third site at ~3348 if found)
5. `cargo build --release --tests --workspace` (expect clean — these are comment edits; should not affect compilation)
6. `cargo test --release --lib -p wat 2>&1 | tail -3` (expect 890/0)
7. `cargo test --release --test function 2>&1 | tail -3` (expect 8/0)
8. `cargo test --release --test probe_arc243_stone3_typeerror_pattern_a 2>&1 | tail -3` (expect 3/0)
9. `cargo clippy --release 2>&1 | grep -cE "^warning:"` (expect ≤ 897)
10. DO NOT COMMIT — orchestrator commits atomic with the full R3 sweep
11. Return paragraph ≤ 100 words: which fixes landed (R3.13/R3.14/R3.15/R3.16 — and whether the third recipe site was found); final gates; any deferral-language sites still suspected (report honestly).

## STOP triggers (REJECTION)

1. Lib < 890
2. tests/function < 8
3. probe_arc243_stone3 < 3
4. Clippy > 897
5. Workspace test-build fails
6. 15 min elapsed (mechanical scope)
7. holon-rs touched (STOP-5)
8. Scope creep into other files
9. New deferral language anywhere
10. AMBIGUOUS deferral text encountered — STOP, surface verbatim
11. INTERSTITIAL touched
12. Commit attempted

## Critical doctrine (read before strike)

1. **NO skip-pre-existing** per `feedback_pre_existing_is_not_exemption`
2. **NO deferral language** new (the 4 fixes REMOVE deferral; don't introduce new)
3. **Affirmative-out-of-scope** is the acceptable shape ("by design", "is a separate concern", "outside the X layer")
4. **Sonnet writes substrate** per `feedback_sonnet_writes_substrate`
5. **DO NOT commit** — orchestrator atomic-commits Stone 243.3 closure
6. **DO NOT cast vigilia or conformare** — orchestrator-cast post-sweep

## Predicted band

**10-15 min Mode A.** Four comment-rewrites; no functional changes; cargo cycle is just regression confirmation.
