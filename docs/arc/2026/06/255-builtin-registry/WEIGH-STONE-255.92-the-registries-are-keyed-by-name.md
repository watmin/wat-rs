# WEIGH — STONE 255.92: the registries are keyed by `Name` — ACCEPTED, with its transition named

**Executor: grok via pulsare, solo, through one amendment.** Commits `ea5fd3339` … `cfe6570e1`, SCORE `3b40d9d5b`.
Weighed by the orchestrator on 2026-10-04.

## Re-run by the orchestrator at `3b40d9d5b`

| row | result |
|---|---|
| release floor | `.floor/2026-10-04T07-52-21Z`: **6417 passed / 24 skipped**, exit 0, doc-link exit 0 |
| test-name set | against `.floor/2026-10-04T04-53-19Z`: **MISSING 0**, ADDED 2 |
| clippy | exit 0 |
| ignores | 18 |
| cost (grok, six and six, isolated) | fuzz deftest mean **31.65 s** after, against **31.95 s** before at `4d4087f03`: no rise |

## What landed

- **Every name registry is keyed by `Name`:** `SymbolTable` (functions, unit variants, `def` values, binding
  metadata), `TypeEnv` (types, builtins, subtype edges and parents, source forms, parametric extensions, generic edges),
  `MacroRegistry`, `CheckEnv`'s variant types, `UseDeclarations`, the closure captures.
- **One name, two keyword joins (STOP-1, ruled by R-a and the pair ruling):** `:user::helper/of` and
  `:user::helper::of` are one entry; the two tests that pinned 255.8's opposite rule are rewritten to assert it, with
  their history.
- **A composite key is structure (STOP-2, ruled):** `TypeKey` = `Name` | `Var` | `Apply { head, args }`, built from the
  `TypeExpr`; the 52 rendered-text keys and their side maps are gone.
- **The collision instrument** ran on three floors: **0** collisions, so no two declarations claimed one pair.
- `UseDeclarations::covers` compares namespaces as data. Two keyword compares left `is_subtype` (heresy ledger
  63 → 65 → back), and the walk's vacuity bound moved 300 → 250 with the measurement recorded: sites fell because
  heresies were cured, and the bound still fails a walk that stops early.

## The transition, named (stone 3 deletes it)

The registries are keyed by `Name`, but **their callers still pass text**: `get(&str)` remains, served by a
**spelling index** (`src/name_map.rs:100`, `HashMap<String, u32>` over the inserted spelling, the keyword image and
the `Display`) and a thread-local `Name::enter` cache (`:79`). `CheckEnv`'s scheme map, the defclause and defined-value
lookups and the intrinsic table stay string-keyed; `aggregate_field_names` builds `format!(":{class}")` and falls back
to `canonical_identity`; variant matches compare by `Name::same_entered_name` (text, then enter). That is the bridge
from text callers to `Name` keys, and it is what keeps the cost flat. It is **not** the end state: **stone 3's gate
includes deleting `get(&str)`, the spelling index and the enter cache**, once the `REG`/`DISPATCH` literals are
`Name`s.

## Verdict

Accepted and pushed. Next: 255.93, stone 3: the `REG`/`DISPATCH` literals by a recorded Rust-literal codemod.
