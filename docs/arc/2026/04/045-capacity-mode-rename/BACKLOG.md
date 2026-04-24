# Arc 045 — BACKLOG

Three slices.

---

## Slice 1 — Substrate rename `:abort` → `:panic`

**Status: ready.**

`src/config.rs`:
- `CapacityMode::Abort` enum variant → `CapacityMode::Panic`
  (touches the `enum` definition + every match arm + every
  reference site).
- Parse arm `":abort" => Ok(CapacityMode::Abort)` →
  `":panic" => Ok(CapacityMode::Panic)`.
- Error message strings: `"expected :error / :abort"` →
  `"expected :error / :panic"`.
- 4 unit tests using `:abort` in fixture strings: rename to
  `:panic`.

`src/runtime.rs:4980`: panic message string mentions `:abort` —
rename to `:panic`.

`tests/wat_bundle_capacity.rs:67, 155`: 2 fixture strings — rename
to `:panic`.

Verify: `cargo test --release --workspace` passes.

## Slice 2 — User-facing docs sweep

**Status: ready.**

Two patterns to apply across user-facing docs:

1. **Rename `:abort` → `:panic`** in narrative prose / mode
   tables / namespace descriptions.
2. **Delete redundant `(set-capacity-mode! :error)` lines** in
   demo / example blocks.

Affected files (rough enumeration):
- `README.md` — Status block, §Capacity guard mode table, plus
  any inline examples.
- `docs/USER-GUIDE.md` — §Config setters (optional), §Setup
  examples, §12 Error handling, §14 Common gotchas, examples
  throughout.
- `docs/CONVENTIONS.md` — §Namespaces table mention, §Sandbox
  Config inheritance example, anywhere else.
- `wat-tests/README.md` — likely no `:error` examples but check.
- `wat/std/test.wat` — usage-comment examples I just touched in
  arc 044 — drop the `:error` lines per the new rule.
- `examples/with-loader/wat-tests/*.wat` — header examples may
  still have `:error` lines.
- `docs/README.md` — arc index entries naming the variants.
  Apply the same "current-name + parenthetical rename note"
  pattern from arc 044's slice 1.5.

For arc-index entries describing prior arcs that mentioned
`:abort`: keep the historical reference visible but use current
name. E.g. arc 040's index entry currently says "`:error` /
`:abort`" — should become "`:error` / `:panic` (was `:abort`,
renamed in arc 045)" or similar.

## Slice 3 — INSCRIPTION + cross-references

**Status: obvious in shape.**

- `INSCRIPTION.md` capturing the substrate rename + demo cleanup
  rationale + scope decisions ("historical records stay frozen
  even though they reference :abort").
- `docs/README.md` arc index extended.
- 058 FOUNDATION-CHANGELOG row in lab repo.

---

## Cross-cutting

- Verification after each slice:
  - Slice 1: `cargo test --release` (substrate change must not
    regress).
  - Slice 2: grep for `:abort` and `(set-capacity-mode! :error)`
    across user-facing docs after the sweep — should return
    zero (excluding intentional retirement-prose).
- Commit per slice. Push per commit.

## Sub-fogs

- **Variant ordering in error messages.** `"expected :error /
  :abort"` becomes `"expected :error / :panic"`. Consider
  whether `:panic` should come first since the active default is
  `:error` (named first reads as "expected default or :panic").
  Decision at slice 1: keep `:error` first since it's the
  default; matches existing convention.
- **`DEFAULT_CAPACITY_MODE` constant**. Per `src/config.rs:57`,
  default is `CapacityMode::Error`. The rename touches `Abort`
  not `Error`; default stays the same. No `DEFAULT_CAPACITY_MODE`
  line edits.
- **wat-tests `.wat` fixture using `:abort`**. tests/wat_bundle_capacity.rs
  embeds wat strings — those are TEST inputs being parsed by
  `:wat::config::set-capacity-mode!`. After slice 1's parse-arm
  rename, the test must use `:panic` or it'll fail with
  UnknownVariant. Slice 1 covers both edges in the same commit.
