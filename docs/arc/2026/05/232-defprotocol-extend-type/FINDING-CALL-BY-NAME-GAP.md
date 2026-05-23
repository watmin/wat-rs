# FINDING — `call-by-name` substrate gap (arc 232 prerequisite)

**Status:** EMPIRICAL — disconfirmed the DESIGN's hypothesis that wat supports dynamic keyword-as-head invocation.

**Date:** 2026-05-23

## What the DESIGN assumed (lines 174-176)

> *"arc 232 needs `:wat::core::call-by-name` or equivalent (look up a fn by keyword + call it; may need substrate primitive). The substrate may already have call-by-name — investigate before declaring it a new primitive. Reflection per arc 201 likely covers it. (`(:wat::runtime::lookup-fn keyword)` returns the fn, then call it normally.)"*

## What the probe revealed

Probe at `tests/probe_diagnostic_dynamic_keyword_invocation.rs`. Three test shapes:

1. **Probe 1** — bound substrate-verb keyword: `(let [plus :wat::core::i64::+'2] (plus 2 3))`
2. **Probe 2** — runtime-constructed keyword: `(let [plus (keyword/from-string "wat::core::i64::+'2")] (plus 2 3))`
3. **Probe 3** — user-defn via mangled namespace: `(defn :ns::greeting ...) (let [verb (keyword/from-string "ns::greeting")] (verb "world"))`

**All 3 FAIL** identically with:

```
NotCallable { got: "wat::core::keyword", span: Span { file: "<runtime>", line: 0, col: 0 } }
```

## Why — the dispatch logic (verified `src/runtime.rs:4015-4050` + `5435-5460`)

`eval_list` head dispatch handles:
- **Literal keyword** at parse time → matches dispatcher tables (verbs / special forms / user `defn`s registered in `SymbolTable`)
- **Symbol head** `(foo args)` → `env.lookup(foo)` → must yield `Value::wat__core__fn` to dispatch
- **List head** `((fn ...) args)` → evaluate head; dispatch result if `wat__core__fn`
- **Arc 157 path** — `sym.runtime_def_values.get(name)` — `def`-bound values; must be `wat__core__fn`

**The missing path:** `Value::wat__core__keyword` bound to a local IS NOT dispatched as a verb lookup. The keyword's content is dead data; only the literal head keyword at parse-time hits the dispatcher's string-match table.

No `:wat::core::apply` / `:wat::runtime::invoke` / `:wat::runtime::call-by-name` primitive exists. Grep confirmed.

## Implication for arc 232

The DESIGN's defprotocol dispatcher pattern — build mangled FQDN keyword at runtime + invoke as head — **does not work as written**. Arc 232 has a substrate-extension prerequisite.

Three resolution paths:

**(a) Mint `:wat::core::apply [head <- :keyword] [args <- :Vector<:Holon>] -> :T`** — new substrate primitive that takes a keyword + arg list, looks up the dispatcher (same lookup path as literal-head case), and invokes. Smallest surface; explicit dispatch verb.

**(b) Reshape `eval_list` head dispatch** — when Symbol-binding head resolves to `Value::wat__core__keyword`, auto-resolve that keyword's content as a verb dispatch. Implicit; symmetric with literal-head case. Bigger semantic shift.

**(c) Macro-time closed dispatch** — defprotocol expands to a `cond` over known classifiers; extend-type re-expands the dispatcher with a new arm. **Loses defprotocol's "open extension" benefit.** Disqualified per DESIGN's stated goal.

## Recommended path

**Option (a) — mint `:wat::core::apply` primitive** in a substrate-extension stone (arc 232.0 or a separate prerequisite arc) BEFORE arc 232's main work begins.

Rationale:
- One new primitive; smallest substrate surface
- Explicit verb names the operation honestly (caller sees `(apply built-keyword args)` — dynamic dispatch announced)
- Mirrors Clojure's `(apply f args)` shape — convergence-honest
- Implementation: re-use the same dispatch path as literal-head keyword; takes Value::wat__core__keyword + Vec args
- Open question: argument shape — variadic `(apply head a b c)` vs vector-of-args `(apply head [a b c])`. Clojure does the second. Probably mirror.

## Arc 232 DESIGN update needed

Line 174-176 must be forward-corrected to reflect:
- Empirical finding: `lookup-fn` does NOT exist; arc 201 reflection layer does NOT cover invocation
- Substrate gap: dynamic-keyword-head dispatch is not supported
- Prerequisite: `:wat::core::apply` (Option a) ships in a stone BEFORE arc 232.1 defprotocol BRIEF
- defprotocol dispatcher pattern updated to use `(:wat::core::apply mangled-keyword [self ...])`

## Probe preservation

`tests/probe_diagnostic_dynamic_keyword_invocation.rs` STAYS as permanent design substrate. When the `apply` primitive ships, the probe expectations flip from FAIL to PASS and the probe becomes a regression guard against the gap reopening.

## Cross-references

- `docs/arc/2026/05/232-defprotocol-extend-type/DESIGN.md` lines 174-176 (the hypothesis this probe disconfirmed)
- `src/runtime.rs:4015-4050` (eval_list head dispatch — Symbol/List/literal head paths)
- `src/runtime.rs:5435-5460` (arc 157 def-bound-value path)
- `src/runtime.rs:4765` (`keyword/from-string` primitive)
- `feedback_assertion_demands_evidence` — the probe IS the evidence the DESIGN's hypothesis demanded
- FM 2-bis (in `docs/COMPACTION-AMNESIA-RECOVERY.md`) — empirical-probe-before-BRIEF discipline; this finding is exactly that pattern
