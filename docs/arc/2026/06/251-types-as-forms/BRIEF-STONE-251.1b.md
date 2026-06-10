# BRIEF — Stone 251.1b: NORMALIZE — the symbol head resolves (THE first behavior change)

Read `DESIGN.md`, `DESIGN-STONE-251.0.md` (the mechanism = **normalize-layer**, the
four-questions verdict), `DESIGN-STONE-251.1.md` (the a/b/c split). 251.1a (the warded
`src/resolve/` home) is committed at HEAD. This is **251.1b — the ONE behavior change**:
teach the resolve home to normalize a namespaced dotted-symbol ref to the keyword FQDN
entity it names. **This flips `probe_arc251_stone0_symbol_head` C01 RED→GREEN — the first
`wat.core/…` head that ever resolves.**

## The work (one paragraph)
A `WatAST::Symbol` whose name is a **namespaced ref** (`wat.core/+` — discriminated from a
bare local `x` by **containing `/`**) resolves to the canonical entity its keyword FQDN
names (`:wat::core::+`). The resolve walk **normalizes** (rewrites) the node to that keyword
FQDN, so the UNTOUCHED downstream dispatch (`eval_list` / `dispatch_keyword_head`) resolves
it. **Dual-read:** keyword-FQDN heads keep working (C02). This lifts resolve's current
validate-only ("does NOT transform the AST") limitation — a deliberate, named shift (per
251.0's normalize-layer verdict; native symbol dispatch is NOT this stone — types get genuine
forms at 251.3).

## The mapping (pinned)
A namespaced symbol `a.b/c` — split on the LAST `/` → ns=`a.b`, name=`c` — resolves to a
keyword-FQDN candidate against the registry (the same authority `resolve_references`
validates against):
- **PRIMARY:** `ns_to_wat_path(ns, name)` (`src/edn_shim.rs:1327`, `pub(crate)`) =
  `:` + ns(`.`→`::`) + `::` + name → `wat.core/+` → `:wat::core::+`.
- **FALLBACK (type members):** keep the `/` → `:` + ns(`.`→`::`) + `/` + name
  (`:wat::core::HashMap/length`) — for `Type/member` heads.
First candidate that resolves wins → rewrite the `WatAST::Symbol` node to
`WatAST::Keyword(fqdn, span)` (**preserve the span**). If NEITHER resolves → a **located
error** naming the unknown entity + namespace (NOT a bare `UnboundSymbol`; per 251.0's
contract).

## The hook (`freeze.rs:969`, step 7)
`resolve_references(&residue, &symbols, &macros)?` is step 7; `residue` then flows to check
(step 8) + freeze + eval. Normalize so the rewritten AST flows downstream:
- New module `src/resolve/normalize.rs` (intueri-style name on the home precedent).
- Reuse the resolve walk's **boundary discipline** — `check_form` already skips quote-family
  data (`:wat::core::quote` / `:quasiquote` / `:forms`) and retired `:define`; the normalize
  MUST NOT rewrite symbols inside quoted data. Either fold the rewrite into the existing walk
  (make it transform) or mirror its descent exactly.
- Apply to namespaced symbols in BOTH head and value position (uniform `/`-symbol rewrite) —
  `(wat.core/foldl wat.core/+ 0 xs)` needs both; a keyword in value position already resolves
  to the operator's fn value, so the rewrite composes. If value-position keyword-as-fn does
  NOT already work, STOP and report (don't force it).

## Scope / out of scope (affirmative cuts)
- IN: namespaced symbol refs (`/`-containing) → keyword FQDN, head + value position, dual-read.
- OUT: `wat.type/` namespace + parametrics (251.2/.3); `:-`/`ann-form` (251.4); HARD-CUT of
  keyword spellings (251.5 — dual-read HOLDS); native symbol dispatch (251.3, types only);
  the BARE_PRIMITIVES consolidation (251.1c). Bare symbols (no `/`) are untouched locals.

## STOP triggers (rejection criteria)
1. If making the walk transform ripples widely (a signature change breaks many callers), STOP
   — report; a standalone normalize pass producing the rewritten `residue` before
   `resolve_references` may be the cleaner seat.
2. If an unresolvable namespaced symbol can only surface as a bare `UnboundSymbol` (no located
   error reachable), STOP — the contract requires a located error.
3. If C02 (keyword head) regresses, STOP — dual-read is mandatory.
4. If the normalize would rewrite a symbol inside quote-family data, STOP — correctness bug.

## Gate (the kill — weigh against the disk)
- **Un-ignore** `contract_01_symbol_head_resolves_like_keyword` (remove the `#[ignore]`).
  `cargo test --release --test probe_arc251_stone0_symbol_head` → **C01 GREEN** (`(wat.core/+
  1 2)` = 3) AND **C02 GREEN** (`:wat::core::+` still = 3).
- `cargo build --release` clean; `cargo test --release --workspace --no-run` 0 errors.
- `cargo test --release -p wat --lib resolve` + the resolve home tests green (add 2-3: a
  namespaced symbol head resolves; an unknown namespaced symbol gives a LOCATED error;
  a bare local `x` is untouched).
- **Corpus baseline:** the keyword-spelled `.wat` corpus still freezes/resolves identically
  (dual-read — no regression). Spot-check a few stdlib/kernel programs.
- `cargo clippy --release -p wat -- -D warnings` clean in `src/resolve/`.
  (Skip full-workspace EXECUTION — the arc-213 process tests deadlock; resolve is pure.)

## Expectations
| what | command | expected |
|---|---|---|
| symbol head resolves | `cargo test --test probe_arc251_stone0_symbol_head` | C01 + C02 GREEN |
| compiles | `cargo test --release --workspace --no-run` | 0 errors |
| no keyword regression | resolve tests + corpus spot-check | green (dual-read holds) |
| located error, not UnboundSymbol | new resolve test | unknown `foo.bar/baz` → located error |
| bare local untouched | new resolve test | `(let [x 1] x)` still works |
| clippy in-home | `cargo clippy -p wat` | clean in src/resolve/ |

Runtime estimate: 45–75 min (the mechanism + located-error path + the walk-transform shift).
Return a SCORE: each gate row's result, the normalize module + hook you chose and why, the
located-error shape, honest deltas (esp. the value-position keyword-fn question), files +
line counts, any STOP hit. Do NOT commit — leave on disk for the orchestrator to weigh + ward.
