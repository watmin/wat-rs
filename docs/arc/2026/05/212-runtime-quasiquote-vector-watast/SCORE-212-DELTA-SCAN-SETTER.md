# Arc 212 stone δ-scan-setter — SCORE: migrate scan_for_setter to children()

## Summary

`scan_for_setter` in `src/load.rs` was migrated from explicit `List + Vector` match arms to `form.children()` generic recursion. The List-head setter-keyword check (`:wat::config::set-*!` prefix + `!` suffix predicate firing `LoadError::SetterInLoadedFile`) is preserved verbatim in an `if let WatAST::List` guard. The Arc 167 `Vector` arm and the trailing `_ => {}` arm are collapsed into a single `for child in form.children()` loop outside the guard. Coverage is now extended to `StructPattern` uniformly — pre-arc-212, setters buried inside a `StructPattern` would have slipped past load-time refusal silently.

## Verification

- `cargo test --release --package wat --lib -- load::tests::setter_in_loaded_file_halts --exact`: **1 passed; 0 failed**
- `cargo test --release --test probe_declaration_form_lift`: **6 passed; 0 failed**

## Build

`cargo build --release` — **Finished `release` profile [optimized]; clean**

## Honest-delta note

None. Both named tests passed post-migration. No previously-silent setter in StructPattern was surfaced. Mode A.

## Mode classification

**Mode A** — migration applied; both named tests green; cargo build clean; SCORE written.
