# WEIGH — STONE 255.20: a name nothing declares does not type-check — ACCEPTED

**Executor commit `fb02434ef`.** Weighed by the orchestrator against disk on 2026-09-24.

## Re-measured

| row | measured | result |
|---|---|---|
| tree · floor | `git status`; `.floor/2026-09-24T06-55-44Z/clean.log` | clean; `6042 tests run: 6042 passed (9 slow), 22 skipped` |
| the reproducer (`Locus/bogus-xyz`) | pre-stone build (`74410c56f`) vs new | **old 0 → new 1**, `UnknownCallee` |
| fixture edits | every removed and added line in the three renamed `.wat.bad` files | only the `:wat::core::string::concat` → `:wat::string::concat` rename; nothing else moved |
| ledger | `LEDGER_TOTAL` | 220 → **215**, shrank |

Taken from the report without re-running:

- clippy 0 and delta NEW 3 / RECOVERY 0.
- The tripwire census: 7 names reached the prefix arm and 36 reached the silent accept. Classes:
  - (a) 22: declared by a registry row, the type env, or a surface member.
  - (b) 21 phantoms.
  - (c) **0.**
- The control fired on the reproducer. The tripwire was removed.
- The renamed fixtures' message sets are md5-identical to their pre-stone sets.

## What it closed

- The `:wat::kernel::`/`:wat::std::` prefix arm is gone. It hid three `:wat::kernel::` surface members
  (`StdOut/write`, `StdErr/write`, `StdIn/read-frame`) from the surface-method check, and those are now
  type-checked.
- A missing surface member is `UnknownCallee`.
- The silent accept now requires a declaring authority: a registry row, a known type, or a defined
  value.

## The first floor went red, and was handled correctly

Three reds, each captured whole (`scratchpad/s20/block-*.txt`) and caused by the stone:
1. The ledger shrank.
2. A phantom `:wat::core::add` in a golden fixture.
3. 20 phantom `:wat::core::string::concat` sites in a fixture.

The fixes were a ledger re-freeze and a re-run of the **recorded** migration
`rename-core-string-to-string.wat`. Its original usage never listed `.wat.bad`. After that the floor was
re-run, not re-run to green. ⭐ Each fixture keeps its own finding set, verified by md5 of the message set.

## ⚠ The census STOP-8 — the orchestrator's disposition, stated for the builder

`census --diff` rc=8: `probe_arc258_stone3_fix_source__contract-05-nested-do-if.wat` and
`…contract-07-end-to-end-clean.wat` went **rc 0 → 1**. Both are `include_str!` **golden output text**
for the fix-source tests, not programs: `(:wat.core/if true 1 2)`, a keyword head that names nothing.
Their rc=0 **was the lie.** The executor showed a program using that head checks clean on the pre-stone
binary and dies `UnknownFunction` at run time. This is the "refuses more" direction, and it is correct.
**Accepted by the orchestrator.** The census's next baseline carries 215 non-zero files.

## Findings carried forward, not fixed

- **A third silent door:** the 1-arg non-`:wat::` accessor placeholder (`unresolved_accessor_placeholder`,
  4 sites). It admitted 10 names in the `--check` sweep, including `:anything-at-all`.
- **Resolve's single-segment field-accessor rung** passes a keyword with no `::` (`:wat.core/if`). The
  None-branch comment *"Resolve pass validated the name"* is false for that shape.
- **Accepted heads still return `fresh()`.** A scheme-less registry row or a type in call position (e.g.
  `:wat::type::HashMap`) is accepted with an unconstrained result type. Typing them is not this stone.
- The `:wat::config::set-` prefix arm (`check.rs` ~:5159) is untouched, the same namespace-guess class.
- `:wat::core::string::concat` is not in the retirement table (only `:wat::core::String/concat` is), so
  door 1 cannot teach it. Stale mentions remain in `wat/core.wat` comments.
