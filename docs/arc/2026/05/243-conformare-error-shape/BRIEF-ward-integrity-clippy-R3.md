# BRIEF — Ward-integrity clippy R3 — rune the result_large_err sites OPEN-DEFERRAL → 243.7a

**Agent:** sonnet (`model:"sonnet"`). **Anchor cwd:** `/home/watmin/work/holon/wat-rs/`. `git -C` for git; ignore `.claude/worktrees/`. Do NOT commit. Do NOT touch any `//! vigilatum:` line (orchestrator re-stamps).

The two warded homes `function/` and `rust_deps/` carry un-runed `result_large_err` findings — their vigilatum stamps overclaim per the clippy-gate doctrine. The lint is CORRECT (large RuntimeError-by-value); the fix is the NAMED stone 243.7a (DESIGN-STONE-243.7a.md — box RuntimeError, a 605-site type-level retrofit, NOT doable here). So the honest closure is an OPEN-DEFERRAL exemption pointing at that named, open, in-reach stone. Apply exactly these allows. Touch ONLY the files listed.

## The exemption form (apply at each site)

Place on the line ABOVE the function (or impl/trait item) clippy flags, in this exact shape:

```rust
// rune:excusare(OPEN-DEFERRAL → 243.7a) — clippy is correct (RuntimeError is large-by-value); the fix is the type-level boxing retrofit in Stone 243.7a (named, open, in-reach), not a per-site change. Struck the moment 243.7a ships.
#[allow(clippy::result_large_err)]
```

## The 10 sites (9 warded-home + 1 flat-file illegitimate)

clippy line numbers may have shifted a few lines; find the actual flagged function by running `cargo clippy -p wat --release 2>&1 | grep -A3 result_large_err` and matching the `-->` location, then place the attribute on THAT function.

**function/ home (2):**
1. `src/function/eval.rs` ~34 — the fn returning `Result<Value, RuntimeError>`
2. `src/function/parse.rs` ~188 — the fn returning `Result<(Vec<String>, Vec<TypeExpr>, TypeExpr), RuntimeError>`

**rust_deps/ home (7):**
3. `src/rust_deps/custodia.rs` ~54 — `ensure_owner`
4. `src/rust_deps/custodia.rs` ~77 — the fn returning `Result<R, RuntimeError>`
5. `src/rust_deps/custodia.rs` ~89 — the fn returning `Result<R, RuntimeError>`
6. `src/rust_deps/custodia.rs` ~144 — `take`
7. `src/rust_deps/marshal.rs` ~51 — `from_wat` (this is a TRAIT method signature — place the allow on the trait method; if clippy flags the trait, allow at the trait-method level)
8. `src/rust_deps/marshal.rs` ~362 — the fn returning `Result<Arc<RustOpaqueInner>, RuntimeError>`
9. `src/rust_deps/marshal.rs` ~393 — the fn returning `Result<&'a T, RuntimeError>`

**flat file (1) — the bare reasonless allow excusare flagged ILLEGITIMATE-AT-BIRTH:**
10. `src/runtime.rs:13073` — there is ALREADY a bare `#[allow(clippy::result_large_err)]` here with NO reason. REPLACE it: add the same `// rune:excusare(OPEN-DEFERRAL → 243.7a) — …` reason line above the existing `#[allow]` (keep the allow, give it the legitimate reason). This converts an illegitimate bare suppression into an honest named-deferral.

## After

- `cargo clippy -p wat --release 2>&1 | grep -E "src/(function|rust_deps)/[a-z_]+\.rs"` → EMPTY (all 9 home sites now have documented allows).
- `cargo build -p wat` clean; `cargo test -p wat` green except the banked `probe_8_atom_round_trip`.
- Touch ONLY: `function/eval.rs`, `function/parse.rs`, `rust_deps/custodia.rs`, `rust_deps/marshal.rs`, `runtime.rs`. Nothing else.
- Do NOT re-stamp vigilatum (orchestrator does, after independently confirming clippy-clean-or-runed).

## Return

The 10 allows applied (file:line + the exact rune+allow placed; note site #10 was a bare→runed conversion), the `cargo clippy | grep` results for function/ + rust_deps/ (proving empty), the test tally. Do NOT commit.
