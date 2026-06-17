# Arc 277.1c — the `concat-abuse` lint rule (the `format` RULE-half)

> **STATUS: SHIPPED (2026-06-17).** `concat-abuse` rule live in `wat/lint.wat` (helpers `concat-head?`,
> `concat-arg-counts` → `:(i64,i64)`, `concat-abuse?`, `make-concat-finding`, recursive
> `rule-concat-abuse-form`), wired into `lint-file` after the ladder rule. Report-only (`fix ""`,
> severity `"warn"`). Weighed on the orchestrator's own build: concat-abuse gate 1/1, ladder 1/1,
> deftest 259/1 (+2 deftests: Cases 5 & 6), deporder 0 violations, lib 929/36 — no regression. **The
> strange loop is live: `lint-stdlib` now surfaces 68 findings — the rule flags its OWN
> `make-concat-finding` msg, `violation->finding`, and the format macro-error concats** (the proof-by-diff
> fixtures the sweep will clean once the auto-fix lands). Opened + shipped 2026-06-17.

## Why

`format` shipped (arc 279) as the TOOL. The self-fixing-toolchain doctrine
(SELF-FIXING-TOOLCHAIN.md) says every tool ships with a **RULE** that detects the anti-pattern it
replaces — so no future hand-rolled template survives, and the linter catches its own author's hand.
The anti-pattern: a `string::concat` chain interleaving **string literals** with **non-literal args** —
a template built by hand. The cure: `format`. Examples already in the corpus (the proof-by-diff
fixtures): `wat/lint.wat`'s own `violation->finding` (`:320`) and `make-ladder-finding`'s `msg` (`:243`).

This is **report-only**. The auto-fix (rewrite the concat into a `format` call) needs the node's END
span to compute the edit `old-len` — that is the `ast-end-span` keystone (queue #3 / stone 277.1b).
Until then the rule reports + names the cure; `fix` is `""`.

## The detection (THE CONTRACT)

A rule mirroring `rule-nested-if-=-ladder-form` (`wat/lint.wat:260-281`). Helpers + rule:

- **`concat-head?`** `[node] -> bool` — a `list` whose head is a keyword/symbol (guard with the existing
  `kw-or-sym?`) whose `ast-name` is `":wat::core::string::concat"` **or** `":wat::core::String/concat"`
  (both spellings; mirror the dual-spelling already in `is_pure_total`). Non-list / empty / non-nameable
  head → false.
- **`concat-arg-counts`** `[node] -> Tuple(i64, i64)` — over the args (`drop children 1`): count
  `(n-lits, n-vals)` where a **literal** is `ast-kind == "string"` and a **value** is any other kind
  (symbol, list, int, …). (A `Tuple` of two i64; read with `first`/`second` — proven at macro-eval and
  fine at runtime too.)
- **`concat-abuse?`** `[node] -> bool` — `concat-head?` AND `n-lits >= 1` AND `n-vals >= 1`. (All-literal
  `(concat "a" "b")` → not abuse, nothing to interpolate. All-value `(concat a b)` → not abuse, no
  literal scaffolding. Only the **mix** is the hand-rolled template.)
- **`make-concat-finding`** `[form file n-lits n-vals] -> Finding` — `ast-span` → `:line`/`:col` (copy
  `make-ladder-finding` `:235-241`); rule `"concat-abuse"`; severity `"warn"`; `fix ""`; message:
  `"concat-abuse: string::concat interleaves <n-lits> literal(s) with <n-vals> value(s) — use (:wat::core::format \"…{name}…\" :name v …) instead"`.
- **`rule-concat-abuse-form`** `[form file] -> Vector<Finding>` — if `concat-abuse?` form → report ONE
  finding, do **not** recurse into it (mirror the ladder rule: a matched form is reported whole). Else
  if `lint-structural?` → `foldl` recurse over `ast->children`. Else → empty.

## Wiring

In `lint-file` (`wat/lint.wat:286-300`), the fold currently `concat`s `rule-nested-if-=-ladder-form`.
Add `rule-concat-abuse-form` to the same per-form fold — **after** the ladder rule's findings, so
existing "first finding is the ladder" deftest assertions (Case 1) still hold:
`(concat acc (concat (rule-nested-if-=-ladder-form form path) (rule-concat-abuse-form form path)))`.

## Proof (deftests in `wat-tests/lint.wat` + the Rust gate)

- **Case 5 — detects-concat-abuse:** a SourceFile body `(concat "x: " a " of " b)` → `lint-source`
  yields a finding with `Finding/rule == "concat-abuse"`.
- **Case 6 — no-false-positive-concat:** two clean files — `(concat "a" "b")` (all-literal) and
  `(concat a b)` (all-value) — → 0 findings total (no abuse, no ladder).
- **Rust gate:** `tests/probe_arc277_lint_concat_abuse.rs` (un-ignore) — the mixed concat surfaces
  `>= 1` finding.

## Out of scope (rejected, not deferred)

- **The auto-fix** — needs `ast-end-span` (stone 277.1b); ships report-only, `fix = ""`.
- **Rewriting the corpus's concat-abuse sites** (incl. `violation->finding`) — that is THE SWEEP
  (queue #4), after the auto-fix lands. This stone only makes the rule *detect*.
- **A `concat` of two values with no literal** — deliberately NOT flagged (no template to extract).

## Four questions

- **Obvious?** YES — "concat mixes literals and values → you meant a template → use format" reads
  exactly as the message says.
- **Simple?** YES — one rule, the established `(form → Vector<Finding>)` shape, copied from the ladder
  rule; one new `concat`-clause in `lint-file`.
- **Honest?** YES — report-only and it SAYS so (`fix ""`); it flags only the genuine mix, not every
  concat; the deferred auto-fix is named to its keystone, not hand-waved.
- **Good UX?** YES — the finding names the literal/value counts and the exact cure; warn-severity (a
  suggestion, not an error); no config.

## Blast radius

`wat/lint.wat` (the new helpers + rule + the `lint-file` wiring) and `wat-tests/lint.wat` (2 deftests)
and `tests/probe_arc277_lint_concat_abuse.rs` (un-ignore). No Rust source edits. No new files beyond
the probe (already committed).
