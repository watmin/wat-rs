# EXPECTATIONS — STONE: the fence refuses what it cannot prove, and the enum gets its name

Two halves that must BOTH land. **A row that passes with only one of them shipped is not a row.**

| # | what | command | expected — EXACT |
|---|---|---|---|
| 1 | it builds | `cargo build --release` | clean |
| 2 | ★★★ **C · an inline `match` in a `:then` is REFUSED** | the working `BOXES=1 label=bb` probe | now a compile-time refusal. **Even the EXHAUSTIVE one** — the fence cannot prove either, so it admits neither |
| 3 | ★★★ **and the diagnostic names the AXIS, not the incident** | same | it says a `match`'s exhaustiveness is **form-level** while the fence's totality axis is **head-level**. **"match is not allowed" teaches nothing and fails this row.** `R29 RVINA ERVDIT` |
| 4 | ★★★ **`cond` with a terminal `:else` still WORKS** | the cond probe | `BOXES=1`. **C refuses `match`, not every conditional.** A wall that takes `cond` with it is too wide. |
| 5 | ★★ `cond` without `:else` is still refused at expand | the no-else probe | `cond: non-exhaustive — needs a terminal :else arm` — unchanged, and NOT re-reported by the fence |
| 6 | ★★ the other axes still fire | the `first` probe | still refused. **C must not become the only thing the fence checks.** |
| 7 | ★★★ **A · `:wat::rete::core::enum::name` exists and works in a `:then`** | a rule rendering a variant | `(:user::K::Bb)` → `"Bb"` — **the variant name, no leading colon** |
| 8 | ★★★ **and the core intrinsic under it is reachable from ordinary wat** | a direct call | same answer outside a rule. **A rete row over a core op that only works inside rete is two things pretending to be one.** |
| 9 | ★★★ **the three lost captures are RESTORED** | `277-head-kind-census.wat` · `277-layout-shape-probe.wat` · `rules-corpus-03-source-to-facts.wat` | each captures the kind again, via `enum::name`. **`head-kind-census` censuses kinds again — that is the whole point of the stone.** |
| 10 | ★★ and the four kind-pinned sites are left alone | `g7_rule` · `probe-grep-cli` · `probe-grep-driver` | still the literal `"symbol"`. **`enum::name` is not a licence to churn sites that were correctly resolved.** |
| 11 | ★★★ **negative controls for BOTH halves, in `tests/`** | `tests/rete/…` | un-arm C → the match probe compiles again (RED). Un-arm A → the render test fails (RED). **A control for only one half leaves the other unguarded.** |
| 12 | ★★ the previous stone's checks still hold | the ARM-A and Alpha/Beta probes | both still refused, with declared types named |
| 13 | ★★ the enum gates still hold | `ast_kind_nodekind_sync` · `every_wat_scripts_file_loads` | `PASS` · `1 passed` |
| 14 | ★★ nothing moved in the formatter | the fmt digests | `deporder c69f0460e9f91672` · `spawn 1c4bbb19bee10093` · `io ba89b65cbd61abc6` · `fmt cae5250235f652ec` |
| 15 | ★★ `wat-grep` still passes | `wat_grep::*` | **7 passed** |
| 16 | floor (ORCHESTRATOR) | `scripts/floor.sh` | `5185+` run, **0 FAILED** |
| 17 | clippy (ORCHESTRATOR) | `-D warnings --all-targets` | `0` |

**Runtime prediction:** 75-120 min. C is one head and a diagnostic; A is an intrinsic, a vocabulary
row, a `infer_rete_form` arm, and three restored callers.

## Trap-doors named in advance

- **Row 2 refuses the EXHAUSTIVE match too, and that is deliberate.** The fence's axis is head-level;
  it cannot tell an exhaustive match from a partial one, so admitting the good one means admitting
  both. A fix that only refuses the *non*-exhaustive case has quietly built option B — the
  second exhaustiveness checker — and STOP-2 forbids it.
- **Row 4 is the wall's width.** `cond` is the obvious collateral: it is a conditional, it is in the
  admitted set, and it is ALREADY guarded at expand time. Taking it out is over-reach.
- **Row 8 is the "two things pretending to be one" check.** The rete row is an exposure of a core op;
  if the core op does not stand on its own, the layering is wrong.
- **Row 9 is the point of the stone.** C is the wall; A is the road; row 9 is someone walking it. C+A
  with no restored caller has fixed nothing the builder asked for.
- **Row 11 needs BOTH controls.** One half guarded and one half bare is how a wall rots.
- ⚠ **`wat/*.wat` is FROZEN into the release binary** — `wat/rete/compile.wat` is the fence, so
  `cargo build --release` between the edit and any measurement or you are testing the old fence.
