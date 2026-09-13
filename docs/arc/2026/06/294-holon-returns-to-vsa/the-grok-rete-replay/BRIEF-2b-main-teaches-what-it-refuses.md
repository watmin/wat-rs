# BRIEF 2b — main's record teaches what main refuses

Anchor: `/home/john/work/holon/wat-rs`. Verify with `pwd`. **Never use worktrees.** Do not touch
`~/work/holon/` (the frozen root).

## ⛔ TREE STATE

```
branch   replay/grok-rete   <-- YOU ARE HERE
HEAD     (the commit that lands this brief)   floor 5404/5404 · clippy 0 (pilot SCORE, verified)
```

## Why

The composition check (`FINDINGS-composition.md`, findings 5–8) found main-owned defects: text and
probes that tell the reader something main's own binary refuses. None of them is rete. Main owns all
syntax and all behaviour outside the branch's subsystem, so they are fixed here, on main's side.
Every claim below was measured on main's binary `bootstrap/wat-main-a3218644d`.

## D1 — seven refusals teach the retired variant separator

The variant separator is `.`. The door composes it: `crates/wat-reader/src/identifier.rs:362`,
`compose_variant` → `format!("{enum_path}.{variant_name}")`, and `decompose_variant` splits on `.`.
`src/match_arm.rs:133`, `is_namespaced_variant` = `decompose_variant(path).is_some()`.

These refusal texts still tell the user to write `::`:

| site | text today |
|---|---|
| `src/match_arm.rs:108` | `variant arm head {k} is not namespaced; write <enum>::<Variant>` |
| `src/match_arm.rs:187` | `write a qualified Type::Variant FQDN` |
| `src/check.rs:2877` | `expected (:wat::runtime::variant-parent-of :Ns::Enum::Variant)` |
| `src/check.rs:6955` | `variant {path} must be <enum>::<Variant>` |
| `src/check.rs:7253` | `variant constructor pattern {path} must be <enum>::<Variant>` |
| `src/check.rs:7508` | `keyword sub-pattern … must be <enum>::<Variant> or :None` (the bare `:None` is retired too, `match_arm.rs:177`) |
| `src/check.rs:7755` | `variant constructor pattern {} must be <enum>::<Variant>` |

Measured: `bootstrap/era/probe-H/sep-colon.wat` (`[:u::E::A {:x x} x]`) → the first row's refusal;
`sep-dot.wat` → rc 0. So the text names the spelling it just refused.

**The method, per site:** drive the refusal with a small input, read which predicate fires, and
write the remedy that predicate accepts. Then apply the remedy to the same input and confirm it
passes. Each site guards a different predicate, so each remedy is derived there, never copied
across from another site. Pin each site in ONE new test file,
`tests/diagnostics/refusals_teach_the_dot_separator.rs`: one test per site, each asserting the
refusal names the dot form AND that the remedied input is accepted.

**Two more places teach `::`:**
- the lint's own help text, `tests/lint/one_variant_separator.rs:260` (`compose_variant(…) ->
  {enum}::{variant}`);
- the door's own doc comment, `crates/wat-reader/src/identifier.rs:366-371` ("`compose_variant`
  writes `::`; this reads `::`").

Bring both in line with the code.

## D2 — c03's template cannot be expanded, and its test never expands it

`tests/macros/probe_arc241_stone17_defmacro_canonical_c03.wat` defines
`` `(:wat::core::Vector ~@items)`` for `variadic-wrap`. Expanded on main's binary with
`(:test::variadic-wrap 1 2 3)` (`bootstrap/era/probe-H/c03-main-run.wat`), it fails with
`first argument must be a (Head :- [T]) type param-spec`. `contract_03`
(`tests/macros/probe_arc241_stone17_defmacro_canonical.rs:48`) only asserts that the file starts up.

1. Make `contract_03` invoke the macro and observe a 3-element result. It goes RED on today's
   template. Record that red verbatim; it is the proof the row now sees the defect.
2. Then write the template in a spelling main accepts that still means "wrap the items". Read the
   test's intent first: the file is about the `& items` rest binder.

## D3 — two probes whose child programs die at startup on main

Both print `#wat.kernel/RecvOutcome.Lost … StartupError` and exit 0, while their headers say
`EXPECT (green): "echo:z"`. `--check` passes both, because it checks the parent program only.

- `wat-scripts/probes/arc-170/probe-m1-ann-erase.wat`: the child's arms `[:probe::CMsg::Setup …]`
  and `[:probe::CMsg::Work …]` use `::`. `CMsg` is declared in the child only (line 43).
- `wat-scripts/probes/arc-170/probe-m1-ann-erase2.wat` is erase with `PMsg`/`CMsg` renamed to
  `PoolMsg`:
  - line 47 still names `:probe::CMsg` in the child's `serve` signature;
  - the child's arms carry `{:deps addr}` and `{:pair s}`, where its `PoolMsg` declares `addr` and
    `s`. The parent's sends (lines 72-73) already use `{:addr …}` and `{:s "z"}`.

  Finding 5 explains the `deps`/`pair`: the landing's match-arm leaked `probe-m1-phantom-d.wat`'s
  `PoolMsg` across files.

Fix both. Each must print `echo:z` when run on `./target/release/wat`.

## Blast radius

Message strings in `src/match_arm.rs` and `src/check.rs` · the doc comment in
`crates/wat-reader/src/identifier.rs` · the help text in `tests/lint/one_variant_separator.rs` · the
new `tests/diagnostics/refusals_teach_the_dot_separator.rs` (the `diagnostics` test target is rooted at `tests/diagnostics/mod.rs`, Cargo.toml
`[[test]] name = "diagnostics"`; add `mod refusals_teach_the_dot_separator;` there) · `tests/macros/probe_arc241_stone17_defmacro_canonical{.rs,_c03.wat}` · the two
`probe-m1-ann-erase*.wat`. No change to any predicate.

## ⛔ STOP triggers — each is a REJECTION. Change nothing at that site; report the verbatim evidence.

- **STOP-1:** a refusal's predicate ACCEPTS the `::` spelling it names. Then its text is not stale.
  Report the input and the rc.
- **STOP-2:** the remedy a predicate accepts cannot be stated as a spelling (it depends on context
  the message cannot see). Report it; do not guess a text.
- **STOP-3:** D2 has no spelling under the param-spec wall that expands to a 3-element result.
- **STOP-4:** the floor is red. Paste the failing test's whole block verbatim, and do not re-run.

## Tier

Commit on green: `scripts/floor.sh` (read the Summary) and
`cargo clippy --release --all-targets -- -D warnings` at 0. **Do not push.** Yield with
`SCORE-2b.md`, one row per EXPECTATIONS-2b line. Copy the shape of `SCORE-1-pilot.md`.
