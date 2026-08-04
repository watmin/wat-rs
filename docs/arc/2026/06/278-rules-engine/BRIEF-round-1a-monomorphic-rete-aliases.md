# BRIEF — #57 round 1a: teach `ParamType` two types, then mint the nine monomorphic rete aliases

Anchor at `/home/watmin/work/holon/wat-rs/`; verify with `pwd` first; use
`git -C /home/watmin/work/holon/wat-rs` for any git read. Tree is clean at HEAD.

## The work in one paragraph

A `where` clause may only call rete-namespaced ops. The rete vocabulary table (`RETE_OPS`) holds 18
rows today — the whole `i64` module plus the form mirrors — and the string/float half of the
vocabulary the corpus actually uses has never been minted. It could not be: a row's type travels
only through `ParamType`, which has three variants (`I64`, `Bool`, `Keyword`) and cannot spell
`String` or `f64`. Add those two variants, then add nine rows. No new logic, no runtime change, no
semantics: each row names a rete FQDN and the core routine it surfaces.

## Read in order

1. **`docs/arc/2026/06/278-rules-engine/DESIGN-STONE-where-admits-only-rete-ops.md`** — read the
   `⛔⛔ SCOUT 2026-08-05` block at the top (why this is 1a and not "round 1"), then
   `★★ RULED — THE RETE SURFACE IS PER-TYPE, PERIOD` and `★ THE MINT ROUNDS`.
2. **`src/rete/vocabulary.rs:87`** — `ParamType` and its `to_type_expr`. This is the whole
   type channel; `check.rs:15820-15821` builds each row's `TypeScheme` from it and there is no
   other path.
3. **`src/rete/vocabulary.rs:125`** — `RETE_OPS`. The first row (`:wat::rete::i64::>`, an `Alias`)
   is your exemplar: copy its shape exactly.
4. **`src/rete/purity.rs:~176-182` and the `total` match arms** — where all nine ops are *already*
   classified. You are not deciding their axes; you are transcribing what is already there.
5. **`src/string_ops.rs`** — each op's real signature, in its doc comment
   (e.g. `(:wat::core::string::to-lowercase s) → :String`).

## Part 1 — two variants

Add `String` and `F64` to `ParamType`, with their `to_type_expr` arms returning
`TypeExpr::Path(":wat::core::String")` and `TypeExpr::Path(":wat::core::f64")`. Confirm those two
spellings against how the codebase already writes them rather than assuming.

## Part 2 — nine rows

All nine are `class: OpClass::Alias` and all nine carry
`meta: OpMeta { pure: true, deterministic: true, total: true }` — **transcribed, not decided.**
`purity.rs` already classifies every one: the `String/` family and `i64::to-f64` sit in the total
arm; `string::{length,trim,to-lowercase}` are named in the prose at `:176-182` as *"each verified
total by reading its own implementation."*

| `rete_name` | `core_name` | shape to VERIFY against `string_ops.rs` |
|---|---|---|
| `:wat::rete::String/concat` | `:wat::core::String/concat` | `(String, String) -> String` |
| `:wat::rete::String/starts-with?` | `:wat::core::String/starts-with?` | `(String, String) -> Bool` |
| `:wat::rete::String/ends-with?` | `:wat::core::String/ends-with?` | `(String, String) -> Bool` |
| `:wat::rete::String/contains?` | `:wat::core::String/contains?` | `(String, String) -> Bool` |
| `:wat::rete::String/empty?` | `:wat::core::String/empty?` | `(String) -> Bool` |
| `:wat::rete::string::length` | `:wat::core::string::length` | `(String) -> I64` |
| `:wat::rete::string::trim` | `:wat::core::string::trim` | `(String) -> String` |
| `:wat::rete::string::to-lowercase` | `:wat::core::string::to-lowercase` | `(String) -> String` |
| `:wat::rete::i64::to-f64` | `:wat::core::i64::to-f64` | `(I64) -> F64` |

**The signatures above are the orchestrator's reading and each one owes verification.** If any op's
real arity or type differs — `String/concat` being variadic would be the likeliest — that is STOP-2.

## ⛔ STOPs — rejection criteria, not permission slots

- **⛔ STOP-1 — mint EXACTLY these nine. Do not add a tenth.** Not `string::subs` (it is
  **genuinely partial** — `eval_string_subs` raises on out-of-range indices, which is round 2's
  fallback-carrying class and the reason it is excluded here). Not `=`/`not=` (per-type by ruling,
  and *which* types is unmeasured — that is 1c, an audit). Not the `PersistentVector/` family or
  the HOF combinators (parametric and higher-order; `ParamType` cannot express them and the design
  question is the builder's — 1b).
- **⛔ STOP-2 — if a real signature differs from the table above, STOP and report it.** Do not
  reconcile it yourself; a wrong `params`/`ret` produces a row whose `TypeScheme` lies.
- **⛔ STOP-3 — do NOT arm anything.** This strike only makes the rete spellings *exist*. The core
  spellings must keep working exactly as they do today. If any existing `where` starts being
  refused, you have armed something — STOP.
- **⛔ STOP-4 — do NOT touch `head_ok`'s door ordering** (the `sym.functions`-before-admission-test
  fix). That belongs with the HOF combinators in 1b and only matters once arming happens.
- **⛔ Do not add a `_` wildcard arm on an enum scrutinee.** Doctrine. Adding `ParamType` variants
  will make some matches non-exhaustive — fix them by naming the new arms.
- **⛔ Do not commit, stash, push, or touch git.** Leave the tree dirty; the orchestrator weighs.

## Verify — FOREGROUND, and block on it

```
cargo build --release
cargo nextest run --release
cargo clippy --release --all-targets
./wat-scripts/perf/grid/check-where-shapes.sh
```

Read the **Summary line**; never a piped exit code. **The floor to match is
`4348 run / 4348 passed / 0 failed / 262 skipped`** and the Clara gate must still report
**9 pairs / 98 rows**. This strike adds vocabulary; it must move no derived fact and no test.

Report: the verbatim Summary line; the Clara gate's verbatim last line; every file touched; each
of the nine signatures as you verified it (not as this brief guessed it); and anything you assumed.

---

## EXPECTATIONS — written before the strike

| # | what | command | expected |
|---|---|---|---|
| 1 | the rows exist | count `ReteOp {` in `RETE_OPS` | **27** (18 + 9) |
| 2 | ★ the new spellings RESOLVE | `--check` a scratch file calling `:wat::rete::String/starts-with?` | accepted |
| 3 | ★ **not vacuous** — a bogus rete spelling still fails | same file, `:wat::rete::String/starts-with-x?` | refused |
| 4 | ★ nothing armed: the CORE spelling still works in a `where` | the where-corpus | unchanged |
| 5 | ★ the Clara gate | `check-where-shapes.sh` | `9 pair(s), 98 rows — wat == Clara on every shape` |
| 6 | ★ floor | `cargo nextest run --release` Summary | `4348 / 4348 / 0 / 262`, exactly |
| 7 | clippy | `cargo clippy --release --all-targets` | clean |
| 8 | `string::subs` was NOT minted | `grep 'rete::string::subs' src/` | zero hits |
| 9 | the meta column is transcribed | read the nine rows | all `pure/deterministic/total = true`, matching `purity.rs` |

Rows 2, 3, 5 and 6 are re-run by the orchestrator by hand regardless of what is reported.

**Runtime prediction: 25–40 minutes.** Time-box 80. Two build+test cycles plus the Clara gate
(~10s per pair, JVM boot amortised once) dominate.

**Trap doors, named in advance:**
1. **Minting `string::subs` because it sits in the same namespace as its three total siblings.**
   It is partial. `total` is deliberately NOT blanket over that prefix, and `purity.rs` says so in
   prose at `:176`.
2. **Deciding the meta axes instead of transcribing them.** All nine are already classified; a row
   that disagrees with `purity.rs` creates two answers to one question.
3. **Arming by accident.** Adding a rete spelling must not remove a core one. Row 4 is the detector.
4. **Non-exhaustive matches on `ParamType`** — two new variants will surface them. Name the arms;
   never reach for `_`.
