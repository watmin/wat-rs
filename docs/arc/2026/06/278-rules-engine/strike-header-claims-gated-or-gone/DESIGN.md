# DESIGN — five rotted header counts, and the gate that already exists for them

**Status:** drawn 2026-09-08. Resolves **`2I1`**, **`2I2`**, **`2R1`**, **`R1`** (all L1) and
**`R2`** (L2). ⭐ **One class, one existing mechanism, no new policy.**

## Why — the codebase already wrote this doctrine and built the gate

`tests/lint/rete_header_claims_are_asserted.rs`, first line:

> **"A HEADER'S STRUCTURAL CLAIM IS AN ASSERTION HERE, OR IT IS NOT MADE."**

Its own origin is the story this vigilia has now retold five more times: a 2026-08-30 doc pass wrote
claims about the tree's layout, `intueri` checked them and found *"most of them false within hours
of being written"* — *"only these four `*_pass`"* (six), *"the eleven session fields"* (fourteen),
*"`kernel/tests.rs` is their only caller"* (deleted the same day). The header draws the conclusion:

> *"Every one is CHEAP to check — a grep, a field count — and none was checked. **An assertion no
> gate can check rots undetected by construction.**"*

**It gates six claims today.** The vigilia found five more of the identical class.

## The five

| row | claim | truth |
|---|---|---|
| **`2I1`** | `NAMING_RULE_EXCEPTIONS` is **nine** (`vocabulary.rs:104-105`), **eleven** (test doc `:1845`), **fourteen** (test fn NAME `:1846`) | the assertion at `:1853` says **14** and is the only enforced one — **and it is the true one** |
| **`2I2`** | `import_export`'s *"Its **194 lines** are phase COUNT rather than depth"* (`export.rs:2274`) | body spans `:2278-2535` — **258 lines**. Rotted by 64. The qualitative claim (9 phases, nesting peaks at 3) still holds |
| **`2R1`** | `enum_variant_ctor`'s shape justified by *"the **three** callers"*, said three times (`matcher.rs:115,124,127`) | **four** — `purity.rs:976`, `expr_ir/mod.rs:1087`, `validate/mod.rs:1056`, **and `validate/typing.rs:335`** |
| **`R1`** | the outcome enum is not pushed down because *"it has three callers (… and the query path at **`fire/rules.rs:425`**)"* (`outcome.rs:22-25`) | **`:425` contains no call** — it is a session-field data literal; the query call is at **`:433`** |
| **`R2`** | *"Session stays 8 fields"* is called **THE ONE CONTRACT** of `DESIGN-STONE-intern-zero-mutex` | **true today (8)** — and **nothing asserts it**, while the sibling claim one door over, `FireCtx`'s field count, **is** gated by this very file |

⭐ **`2I1` contains its own cure in miniature:** four numbers describe one array, and the one that is
*mechanically enforced* is the one that is *right*. That is the whole argument for this strike.

## What this delivers

Each of the five ends in exactly one of two states, per the gate's own first line:

1. **Asserted** — a new test in `rete_header_claims_are_asserted.rs`, in the shape of the six already
   there, and the prose corrected to match the truth it now enforces.
2. **Not made** — the count is deleted from the prose, because the qualitative claim around it stands
   without a number.

## The one contract decision, pinned

⛔ **A COUNT THAT SURVIVES MUST BE ASSERTED. A COUNT THAT IS MERELY CORRECTED IS A ROT RESTARTED.**
Fixing `194` to `258` and walking away buys nothing — the next edit re-rots it, and this gate's own
header records that outcome happening *within hours*. **If a number is worth writing, gate it; if it
is not worth gating, cut it.**

⚠ **Judgement is expected on which.** `2I2`'s line count is a poor gate (it moves on every edit and
means nothing) — that one is a strong **cut**. `2R1`'s caller count and `R2`'s field count are exactly
what this gate already does — those are **asserts**. Say your reasoning per row.

## Out of scope = rejected

- **Changing `enum_variant_ctor`'s return shape** (`2R1`) — the doc's *justification* is wrong; the
  shape may still be right. This strike fixes the claim, not the design.
- **Re-deciding `R1`'s deferral** — the citation is wrong; whether the enum should be pushed down is
  a separate question and not this one.
- **Auditing every count in `src/rete/`.** Five rows, five claims. A general sweep is its own strike.
