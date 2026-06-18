# Arc 277.1d — the concat-fix POSITION gate (format at runtime, interpolate in macros)

> **STATUS: SHIPPED (2026-06-17).** Weighed on own build + eyeballed: defmacro-body (concat s "::Op") -> (string::interpolate "{s}::Op" :s s); defn-body (concat "x: " a) -> (format "x: {a}" :a a). Gate 1/1, 277.1c 2/0, lib 929/36, deftest 264/1, deporder 0. The sweep is now SAFE. RED probe `tests/probe_arc277_1d_concat_fix_position_gate.rs`
> (`#[ignore]`'d). Makes the 277.1c-fix concat→format fix position-aware so the SWEEP is safe (the sweep
> broke the stdlib because `format` is expand-time-illegal). Rides arc 284 (`string::interpolate`).

## Why

The concat→format fix emits `(:wat::core::format …)` always — illegal inside a defmacro body (`format`
is a macro, refused at expand time, arc 249 F5; the sweep proved it, 277 R3). arc 284 shipped
`:wat::core::string::interpolate` — a pure-total intrinsic, **legal everywhere** (runtime AND expand
time). So the fix picks the head by POSITION:
- **runtime position** (not in a defmacro body) → `:wat::core::format` (the macro — evaporates, zero cost)
- **expand-time position** (inside a defmacro body) → `:wat::core::string::interpolate` (the intrinsic)

Same template + kwargs; only the head keyword differs.

### Why a COARSE `in-defmacro?` flag is correct (not quasiquote-precise)

The truly-precise gate would track quasiquote/unquote depth (a concat in an emitted-runtime quasiquote
region is runtime; in an expand-eval region it's expand-time). We DON'T need that: **`interpolate` is
legal in BOTH** runtime and expand-time. So "inside ANY defmacro → interpolate" is always CORRECT — the
worst case is an emitted-runtime concat gets `interpolate` (a call-time parse) instead of the zero-cost
`format` macro: a perf nuance on non-hot macro-emitted strings, never a bug. Simple + honest + safe.
(Outside a defmacro → `format` is both legal and zero-cost.)

## The contract — 5 small edits in `wat/lint.wat`

1. **`is-defmacro-form? [form] -> bool`** (new helper): `form` is a `list` whose head (kw-or-sym) `ast-name`
   == `":wat::core::defmacro"`.
2. **`rule-concat-abuse-form [form file in-defmacro?]`** — add the `in-defmacro?` param.
   - On a concat-abuse hit: `(make-concat-finding form file n-lits n-vals in-defmacro?)`.
   - On the structural recurse: each child recurses with
     `(:wat::core::or in-defmacro? (:wat::lint::is-defmacro-form? form))` — once inside a defmacro, stays
     true for all descendants.
3. **`make-concat-finding [form file n-lits n-vals in-defmacro?]`** — thread `in-defmacro?` to the fix:
   `fix = (:wat::lint::concat-format-fix form in-defmacro?)`.
4. **`concat-format-fix [form in-defmacro?]`** — pick the head:
   `head-str = (if in-defmacro? ":wat::core::string::interpolate" ":wat::core::format")`. The `new-text`
   builder uses `head-str` as the call head (everything else — `{name}` template + `:name val` kwargs +
   bare-symbol eligibility — UNCHANGED).
5. **`lint-file`** — the entry: `(rule-concat-abuse-form form path false)` (top-level forms start NOT in a
   defmacro).

(`make-ladder-finding`/the ladder rule are UNCHANGED — `contains?` is pure-total = legal everywhere, so
the ladder fix is already position-independent.)

## Proof

- `tests/probe_arc277_1d_concat_fix_position_gate.rs` (un-ignore): a source with a defmacro-body
  bare-symbol concat AND a defn-body one → `lint-fix-file` → the defmacro one becomes
  `(:wat::core::string::interpolate "{s}::Op" :s s)`, the defn one becomes
  `(:wat::core::format "x: {a}" :a a)`; no `string::concat` survives.
- `tests/probe_arc277_1c_concat_format_autofix.rs` still GREEN (its fixture is a defn → still `format`).
- Floors: lib 929/36, deftest (+1 for a new gate deftest → 264/1), deporder 0.

## Out of scope

- Quasiquote-precise positioning (unneeded — interpolate is universally legal; see above).
- THE RE-SWEEP — the immediate FOLLOW (orchestrator re-runs `wat-scripts/fixes/sweep-lint-fixes.wat`
  on the hardened, position-aware fix; derive the file set + grep both colon-forms, the 283 lesson). This
  stone makes the sweep SAFE; running it is the next step.

## Four questions

- **Obvious?** YES — "format at runtime (zero cost), interpolate in macros (legal there)" is the one-line rule.
- **Simple?** YES — one flag threaded through 3 fns + one helper; the head is a single `if`.
- **Honest?** YES — the coarse flag is correct because interpolate is genuinely legal everywhere; no guess.
- **Good UX?** YES — the fix now produces LOADABLE code in both positions; the sweep can run.

## Blast radius

`wat/lint.wat` (the helper + the `in-defmacro?` thread through 3 fns + the head choice) + a `wat-tests/`
deftest + un-ignore the probe. No Rust changes (rides arc-284 interpolate). The ladder rule untouched.
