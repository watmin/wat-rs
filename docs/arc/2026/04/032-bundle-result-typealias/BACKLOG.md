# Arc 032 — `:wat::holon::BundleResult` typealias — BACKLOG

**Shape:** three slices. Status markers:
- **ready** — can ship now
- **obvious in shape** — will be ready when the prior slice lands

---

## Slice 1 — substrate: register the typealias

**Status: ready.**

Target: `src/types.rs::register_builtin_types`.

Add `TypeDef::Alias` registration mirroring the existing
`:wat::holon::CapacityExceeded` struct registration pattern:

```rust
env.register_builtin(TypeDef::Alias(AliasDef {
    name: ":wat::holon::BundleResult".into(),
    type_params: vec![],
    expr: TypeExpr::Parametric {
        head: "Result".into(),
        args: vec![
            TypeExpr::Path(":wat::holon::HolonAST".into()),
            TypeExpr::Path(":wat::holon::CapacityExceeded".into()),
        ],
    },
}));
```

**Rust unit tests** (where existing alias tests live in
`src/types.rs` or `src/check.rs`):
- `bundle_result_alias_registered`
- `bundle_result_alias_unifies_with_expanded_result`
- `bundle_result_used_as_return_type_in_scheme`

**Sub-fogs:**
- **1a — alias resolution in existing checker paths.** `expand_alias` /
  `reduce` should transparently resolve `:wat::holon::BundleResult`
  to its expansion per arc 004's existing aliased-type discipline.
  Verify tests pass without additional resolver work.
- **1b — does Bundle's own scheme need updating?** Bundle's
  scheme is constructed programmatically in check.rs. The
  returned TypeExpr could switch to `Path(":wat::holon::BundleResult")`
  for ergonomics, but it's not required — alias resolution will
  make both forms equivalent. Defer; verify the behavior but
  don't change the scheme unless forced.

## Slice 2 — call-site migration (wat-rs + lab)

**Status: obvious in shape** (once slice 1 lands).

Mechanical textual replacement:
`:Result<wat::holon::HolonAST,wat::holon::CapacityExceeded>`
→ `:wat::holon::BundleResult`
(and variations with whitespace).

Files in wat-rs:
- `src/runtime.rs`
- `src/check.rs` (comments only; Rust-level TypeExpr construction
  stays programmatic)
- `tests/wat_run_sandboxed.rs`
- `tests/wat_bundle_capacity.rs`
- `wat/holon/Trigram.wat`
- `wat/holon/Ngram.wat`
- `wat-tests/holon/Trigram.wat`
- `wat-tests/holon/coincident.wat`
- `wat-tests/std/test.wat`
- `README.md`

Files in lab:
- `wat/encoding/rhythm.wat`
- `wat-tests/encoding/rhythm.wat`

Python script for safety (literal substring replace, no regex,
no alternation — Chapter 32's poison pattern stayed avoided).
Full workspace test + lab test pass before the commit.

**Sub-fogs:**
- **2a — whitespace normalization.** The long form appears
  with and without spaces (`:Result<wat::holon::HolonAST,wat::holon::CapacityExceeded>`
  vs `:Result<wat::holon::HolonAST, wat::holon::CapacityExceeded>`).
  Script handles both variants. Wat keyword-whitespace rule
  (memory `feedback_wat_keyword_whitespace`) forbids spaces
  inside `<>`; but the long form pre-existed with mixed
  spacing, so the script's search patterns cover both.
- **2b — partial-match false positives.** The long form has
  no shorter prefix in the codebase (verified by pre-sweep
  grep). Literal substring replace is safe.

## Slice 3 — INSCRIPTION + doc sweep

**Status: obvious in shape** (once slice 1 + 2 land).

Writing:
- `docs/arc/2026/04/032-bundle-result-typealias/INSCRIPTION.md`
- `docs/arc/2026/04/005-stdlib-naming-audit/INVENTORY.md` — new
  row in the `:wat::holon::*` section.
- `docs/README.md` — arc index gains row 032.
- `docs/USER-GUIDE.md` — sweep for long-form usage; replace.
- `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/FOUNDATION-CHANGELOG.md`
  — CHANGELOG row.

**Sub-fogs:**
- **3a — BOOK chapter?** No — this is a one-word naming arc.
  Chapter 33 is already the recent big ledger sweep. Not worth
  a dedicated chapter.

---

## Working notes (updated as slices land)

- Opened 2026-04-23 after lab arc 003's retrofit made the
  44-char-wide type appear 14 times in freshly-shipped files.
  Builder invoked `/gaze` to name it; `:wat::holon::BundleResult`
  landed as the name.
