# WEIGH — STONE 255.68: the type record on `main`, and every untyped constructor measured — ACCEPTED

**Executor: a Sonnet subagent, commits `faff7e384` (part 1) and `7b64b3f3e` (part 2, SCORE).** Weighed by the
orchestrator on 2026-09-28.

## Re-run by the orchestrator

| row | result |
|---|---|
| release floor at `7b64b3f3e` | **6215 passed / 24 skipped**, exit 0 |
| `WAT_CHECK_TYPES=1 wat --check tests/cli/wat_cli__check_types.wat` | prints `TYPE` lines per node (e.g. `(:wat::core::PersistentVector :- [...])` with its final substitution) |
| the same with the variable unset | 0 bytes of output: additive, as claimed |

## The STOP, and how it resolved

The first floor was red: 2 lint hits in the new test (Arm A), one heresy-ledger row moved by the `infer` → `infer_node`
wrapper split (Arm B, same 148 total), and 31 time-budget failures (Arm C). **Arm C had a measured cause: the agent had
started two floors 11 seconds apart** (`.floor/2026-09-28T18-30-40Z`, 8 failed + 14 timed out, 582s;
`18-30-51Z`, 10 + 24, 573s), and the timeout sets differed between the two runs. After A and B were cured with the
lint-approved idioms (an `.edn` golden; the ledger recording the rename), **one floor alone was 6215/6215**
(`19-04-14Z`), confirming C. A second, orchestrator-run floor agrees.

## The measurement (AST walk, not regex; joined to the type record by exact file:line:col)

**1,804 untyped constructor calls:** `PersistentVector` 1,339, `Tuple` 237, `PersistentMap` 122, `List` 96, `Vector` 10.

| concrete | generic | unresolved | not checked |
|---|---|---|---|
| **1,627** | **3** | **33** | **141** |

- **1,630 (90%) are codemod-able from the checker's own types** (concrete, or the enclosing definition's parameter).
- **33 unresolved:** the checker's type stays a variable. A person decides. The list is in the SCORE.
- **141 not checked:** 26 are files that do not freeze, and **115 sit inside `quote` / syntax-quote templates / nested
  program literals** (e.g. `wat/core.wat`'s 34 `Tuple` sites are inside `defservice` templates). Code inside a template
  has no type until it is expanded, so the type record cannot see it.
- **`u8` / `char`:** plain `#[wat_intrinsic]` functions. `(wat.type/u8 65)` **type-checks but fails at run time**
  (*"unknown function: :wat::type::u8"*): the T-door's run-time dispatch covers the collection heads but not these.

The agent caught and corrected its own mistakes: a tuple rendering (`:(A,B)`) that silently zeroed every tuple record,
two regex classifier bugs, and **invented line:col numbers in its first SCORE draft**, all removed before commit.
