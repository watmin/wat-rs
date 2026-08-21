# SCORE — arc 109 γ-i, flight 2 (+ follow-up): SHIPPED

Every row re-run by the orchestrator against a release build it built itself. Nothing credited from
the rider's report.

## Scorecard

| # | row | result |
|---|---|---|
| 1 | `defn` takes the binder | ✅ |
| 2a | `(fn :- [X] [a <- :X b <- :X] -> :X …)` · `(f 1 2)` | ✅ checks |
| 2b | · `(f 1 "s")` | ✅ **rejects** — `X` unifies across positions |
| 2c | · `(f "p" "q")` | ✅ checks — `X` not pinned by the first use |
| 2d | · `(takes-str (f 1 2))` | ✅ **rejects** — the return is tied to `X` |
| 2e | · passed directly to a generic HOF | ✅ checks |
| 3 | BOTH spellings → contradiction, **all four cells** | ✅ (see below) |
| 4 | 251.7 no-param-list stays generic | ✅ |
| 5 | concrete-type HOF control | ✅ |
| 6 | parametric kwargs `defn` in binder spelling | ✅ |
| 7 | variadic `defn` in binder spelling | ⚠ **the row was vacuous — see below** |
| 8 | `def` of a non-fn · `check.rs` diff EMPTY · `function/parse.rs` EMPTY | ✅ |
| — | floor | ✅ **4855/4855**, 0 FAIL, 19 skipped |
| — | clippy `-D warnings` | ✅ 0 |

### Row 3, the four-way matrix — and I nearly scored it green at half

|  | binder only | BOTH spellings |
|---|---|---|
| **plain** | ✅ checks | ✅ rejects |
| **kwargs** | ✅ checks | ✅ rejects |
| raw `(def :g<T> (fn :- [T] …))`, bypassing the macro | — | ✅ rejects (Rust-side, defence in depth) |

⛔ The first pass closed the PLAIN path only, and my row-3 probe exercised exactly that path. **A check
covering one of two paths reads exactly like a check covering both** — the identical shape that
shipped `:-` for three of six constructors this morning. Caught by testing the rule rather than the
diff. `[[feedback_scope_the_check_from_the_rule_not_the_diff]]`

The rider's stated obstacle for the second path — *"defn's macro has no located-error channel"* — was
false: `:wat::core::macro-error` is that channel and `defn`'s own macro already used it at
`core.wat:632` and `:838`. The fix landed at the ONE binding where both halves of the contradiction
are already computed, upstream of the kwargs/plain branch, so it structurally cannot fire for one
path and not the other.

## ⚠ Row 7 was VACUOUS — my own row, and it went green while broken

I wrote row 7 as *"a variadic `defn` in binder spelling **registers**"* and tested it with a
**declaration-only** probe. A declaration-only probe cannot tell parsing from registering. The
declaration parses; the function then never registers, and the only symptom is an unresolved
reference at the CALL site. Surfaced by the builder asking how to spell a generic rest-binder.
`[[feedback_a_green_test_can_prove_nothing]]`

## The floor's 10 reds — measured, not dismissed

```
5 × tests/diagnostics/probe_diagnostic_value_snapshot_in_errors
4 × macro-error probes (arc249 threading, arc258 stone2b, arc279 format ×2)
1 × wat_lang/wat_core_cond
```

Exactly **10 golden `.edn` files in the tree reference `src/runtime.rs` or `wat/core.wat`** — a 1:1
match with the failing set. Regenerated through the repo's own `UPDATE_EDN=1` path, then **every
changed byte reviewed**: 18 changed lines, all of them `:line N`, shifting consistently by
`src/runtime.rs` **+50** and `wat/core.wat` **+79`. No message, span file, col, or structure moved.

⚠ `UPDATE_EDN` also rewrote an **11th** golden that was NOT failing — pure single-line→pretty-printed
reformatting, no value changed. **Reverted**: a commit should contain what it claims, and a re-gold
that quietly touches a passing golden is how a golden gets corrupted unnoticed.

## Findings filed OUT of this stone — named, not deferred

1. **Generic variadic `defn` never registers.** Settled against a pristine-HEAD build (siblings
   symlinked so the relative `holon-rs` dep resolves), with a monomorphic-variadic control proving
   the baseline binary works:

   ```
   variadic · monomorphic              ✅ registers   (HEAD and working tree)
   variadic · generic via ANGLE <X>    ⛔ unresolved  (HEAD **and** working tree)
   ```

   **PRE-EXISTING, not a γ-i regression** — the angle spelling fails identically on HEAD, and there
   are ZERO generic variadic `defn`s anywhere in `wat/`, `tests/` or `wat-scripts/`, so the shape has
   never been exercised. Silent: no error at the declaration, only at the call.

2. **The anonymous-`fn` silent-accept** (flight 1) — ANY stray token in the first slot makes the whole
   fn unconstrained and every call to it check vacuously. Pre-existing, reachable by a typo.

3. **let-polymorphism** — `locals` holds `TypeExpr`, not `TypeScheme`; `derive_scheme_from_function`
   is gated `func.name.as_ref()?`. Its own arc.

## Instrument failures this stone cost me — three, all the same one

Twice I read a non-empty **stderr** as "rejected" (the binary's own STALE warning), and once I read a
`0` from a wrapper that was not `cargo`'s exit — a pristine build that had failed on an unresolved
sibling dep, which I then reported as a baseline measurement. Each was caught by a control that
behaved impossibly. **A control that fails invalidates the subject's result too, not just its own.**
