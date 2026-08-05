# BRIEF — rete's `cond` is its OWN macro, expanding into rete's `if`

**Builder's ruling, 2026-08-05:** *"i think we need rete's cond to just be a macro itself that
expands into rete's if?"*

**He is right, and a probe proves it.** This brief replaces the macro-ALIAS approach currently
sitting uncommitted in the working tree.

Anchor `/home/watmin/work/holon/wat-rs/`; verify with `pwd`. Floor at draw time, my own
`--release` re-run: **`4356 tests run: 4356 passed, 262 skipped`**. Clippy clean.
`check-where-shapes.sh` → `9 pair(s), 98 rows`.

---

## ★★ THE EVIDENCE — read this before touching anything

The uncommitted work clones core's `:wat::core::cond` `MacroDef` and re-registers it under the
rete name (`src/freeze/env.rs`). A clone carries **core's template**. Measured, by
`macroexpand`, on the current tree
(`wat-scripts/scratch-pad/probe-cond-alias-expands-to-core-if.wat`):

```clojure
;; (:wat::rete::core::cond ((= :silver :gold) 0.5) ((= :silver :silver) 0.7) (:else 0.9))
;; expands to:
(:wat.core/if (:wat.core.keyword/= :silver :gold) 0.5
  (:wat.core/cond ((:wat.core.keyword/= :silver :silver) 0.7) (:else 0.9)))
;;  ^^^^^^^^^^^^                                   ^^^^^^^^^^^^^^
;;  CORE if                                        recurses into CORE cond
```

**Byte-identical to the core spelling's expansion.** So the rete name is a synonym that launders
straight back into `:wat::core::` after one step. Under law A armed, that expansion is refused —
and it is exactly the second-door shape arc 179 killed (`()`-vs-`nil`): *a second spelling of one
thing is a second door around every wall built on the first.*

**And the target is proven reachable.** `wat-scripts/scratch-pad/probe-rete-if-in-where.wat` —
a `(:wat::rete::where (:wat::rete::core::if ?a true false))` inside a real `defrule` — prints
**`hits=1`**. Rete `if` is a `Form` row that re-dispatches to core `if`'s genuine runtime arm, so
it fires. **Expanding into rete `if` is a correct target, not a hope.**

---

## THE WORK

### PART A — mint `:wat::rete::core::cond` as a REAL defmacro

Copy the shape of core's own macro (`wat/core.wat:1237-1269`) — it is the exemplar, and the
emitted spellings are the only thing that changes:

- every emitted `` `(:wat::core::if …) `` becomes `` `(:wat::rete::core::if …) ``
- every recursive `` (:wat::core::cond ~@…) `` becomes `` (:wat::rete::core::cond ~@…) ``
- the `macro-error` on a non-exhaustive clause list is **kept verbatim** (same text, same
  first-class primitive — a `cond` with no terminal `:else` must stay a located expansion error)
- the `:else` structural comparison, the `List?` head test, and the annotated-form strip are
  **unchanged logic** — only the emitted head keywords move namespace

**HOME: `wat/rete.wat`.** It already hosts rete-namespaced defmacros (`:wat::rete::query` at
`:2175`, `:wat::rete::defrule` at `:2231`), so the reserved-prefix privilege is established there.
**⛔ STOP-1: if `wat/rete.wat`'s load position means the macro is not registered before user
programs expand, STOP and report** — do not silently relocate it to `wat/core.wat` without saying
so; load position is part of a stdlib file's contract (`src/stdlib.rs`).

### PART B — DELETE the alias loop

`src/freeze/env.rs` — the `for op in RETE_OPS.iter()` block added by the previous strike
(registers a cloned `MacroDef` under `rete_name` for every `Form` row whose `core_name` names a
macro). **It must go**, for two reasons and the second is hard:

1. It is the mechanism this brief replaces.
2. With Part A landed it would try to `macros.register(":wat::rete::core::cond")` **on a name the
   wat-source defmacro already registered** — `MacroRegistry::register`'s redefinition guard
   (`src/macros/registry.rs:58`) fires. A collision, not a no-op.

**Net: 26 lines of Rust deleted, replaced by a wat macro.** That direction is the point — the
substrate says the semantics in wat and keeps Rust for what only Rust can do.

### PART C — KEEP, unchanged

- The `RETE_OPS` row for `:wat::rete::core::cond` (`src/rete/vocabulary.rs`) — the fence still
  needs the admission entry and the naming-rule tests still cover it. **Update its comment block**:
  the current text explains the alias mechanism, which will no longer exist. It must instead
  record that rete `cond` is its own macro emitting rete `if`, and carry the `where` gap below.
- The `classify_expr` guard widening through `resolve_core_name` (`src/rete/purity.rs:946`) —
  correct as-is, the byte-identical recipe `match`/`fn` use.

---

## ⚠ WHAT THIS DOES **NOT** FIX — state it, do not fix it

**A `where` body is never macro-expanded.** Grounded twice, by my own read:

- `wat/rete.wat:2315` — `defrule` quotes the conditions verbatim: `(:wat::core::quote ~when-vec)`
  *(cited as `:2251` while the brief was live — the new macro added 64 lines above it. Re-grounded
  at the weigh; a line number is a claim that rots the moment you edit the file above it.)*
- `src/rete/matcher.rs:1237` — `eval_test_core` calls `runtime::eval_inner` on that raw AST
- `src/macros/expand.rs:441` — the expander does **not** descend into `:wat::core::quote`

So a `cond` written literally inside a `where` — **core-spelled or rete-spelled** — still raises
`UnknownFunction` at fire time. Proven both ways on the current tree
(`probe-cond-in-where-baseline.wat`, `probe-cond-rete-where.wat`).

**⛔ STOP-2 — do NOT attempt to make `where` bodies macro-expand.** That is a separate, larger
mechanism change and it is the builder's ruling, not this strike's. Part A is correct and
necessary regardless of how that lands; it is the prerequisite that makes any later expansion fix
produce law-A-clean output instead of core spellings.

**Record the gap** in the `RETE_OPS` comment and in
`NOTE-a-where-body-is-never-macro-expanded.md` (create it; cite the three `file:line`s above and
both probes).

---

## ⛔ STOPs

- **⛔ STOP-1** — load position (Part A).
- **⛔ STOP-2** — do not touch the `where`-expansion gap.
- **⛔ STOP-3 — verify by `macroexpand`, not by reasoning.** The acceptance evidence is the
  expanded form printed by `probe-cond-alias-expands-to-core-if.wat` (retarget it if its name no
  longer fits). It must show `:wat.rete.core/if` and `:wat.rete.core/cond` with **zero**
  `:wat.core/if` or `:wat.core/cond` in the rete arm. Reading the code is not evidence here —
  this whole strike exists because the previous one's expansion was never read.
- **⛔** No `_` wildcard arm on an enum scrutinee.
- **⛔** Do not commit, stash, push, or touch git.
- **⛔** Every verification runs in the FOREGROUND and blocks. Your turn ends when the numbers are
  in your hands, not when a command is launched.

## Verify — FOREGROUND, blocking

```
cargo build --release
./target/release/wat wat-scripts/scratch-pad/probe-cond-alias-expands-to-core-if.wat   # STOP-3
./target/release/wat wat-scripts/scratch-pad/probe-cond-rete-scorecard.wat
cargo nextest run --release
cargo clippy --release --all-targets
./wat-scripts/perf/grid/check-where-shapes.sh
```

Read the **Summary line**, never a piped exit code.

## EXPECTATIONS

| # | what | expected |
|---|---|---|
| 1 ★★ | **the expansion is rete-spelled all the way down** | `:wat.rete.core/if` + `:wat.rete.core/cond`, zero core spellings in the rete arm |
| 2 ★★ | the tier ladder still evaluates | `silver → 0.7`, first-match wins, `:else` fires |
| 3 ★ | non-exhaustive is still a located expansion error | `macro-error "cond: non-exhaustive…"`, located |
| 4 ★ | core's `cond` untouched | its own expansion byte-identical to today |
| 5 ★ | the alias loop is gone | `freeze/env.rs` back to its pre-strike shape |
| 6 ★ | naming-rule tests | 4/4 |
| 7 ★ | other families unregressed | one `Alias`, one `Fallback`, one `Form` (`if`), one `Redispatch` |
| 8 ★ | floor / clippy / gate | ≥ **4356/4356/0** · clean · `9 pair(s), 98 rows` |

Rows 1, 2, 5 re-run by the orchestrator by hand.

**Runtime prediction: 35–60 minutes.** Time-box 120.

**Trap doors:**
1. **Translating the macro by eye and missing a spelling.** Row 1 is what catches it — and the
   annotated-form branch (`wat/core.wat:1268`) is the one most easily skipped, because it recurses
   without emitting an `if`.
2. **Leaving the alias loop in.** It collides (Part B reason 2).
3. **"Fixing" the `where` probes.** They are RED by design and they measure the gap. STOP-2.
