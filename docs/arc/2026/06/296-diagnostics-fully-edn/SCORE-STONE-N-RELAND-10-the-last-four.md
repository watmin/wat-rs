# SCORE — STONE N RELAND 10: the last four

No commit. Floor captured. Lands on DRAW N RELAND-10 (`bb38c6dc5`).

The four counts-exact tests **PASS**. Fail-closed still **PASS**. `"disconnected"` was the
client's view of a corpse.

## The service's own error — not Lost

RELAND 9 already had `fire-rules` of the same two rules + one hot Temp green in main.
This strike drove the rest of the generated body **outside** the service:

| probe | result |
|---|---|
| `fire-rules` + `query` + `Option/expect`, 1 fact | **2** deductions, EXIT=0 |
| same, 240 facts (30 hot) | **60** deductions, EXIT=0 |
| `Journal/query-logs` in main, then that foldl on the 240 logs | **240** logs, **60** deductions, EXIT=0 |

So rete and journal are fine. The death is **inside the sift service handler**.

Prints in the generated arm (write-file, because the serve thread's stdout is not the
test's): last completed statement is **`SIFT-BEFORE-CONCAT`**. Rust instrumentation of
`eval_fire_rules_native`: **entered, returned Ok**. `fire-rules` is not the corpse.

The next form is `concat-chain` — `into` of `map` over `(:wat::rete::query fired (make-query …))`.

## Expand (STOP-1 of RELAND 9, still the instrument)

`macroexpand-1` of `sift-rules-defsvc`:

```
(:wat::rete::query fired
  (:wat::rete::make-query "usr::Hot" (:wat::core::quote [])
    (:wat::core::quote [(?fact <- :usr::Hot)])))
```

**`query-read` count = 0.** `query` is a **macro** (`wat/rete/syntax.wat`) that expands to
`query-read` + empty params. This AST is **spliced from an outer macro**
(`sift-rules-defsvc`) into a `defservice` body. The inner macro is **not re-expanded**.
At runtime the serve thread invokes `:wat::rete::query` as a function. The thread dies.
The client sees `RecvOutcome::Lost` / `"disconnected"`.

That is why fail-closed PASSES (never enters `concat-chain`) and why the same `query`
call from a `defn` in main PASSES (the expander walks a source form).

Not the purity-lattice / `Option::None` / `classify_closure` suspect. Named, measured.

## The fix

`wat/query.wat` `query-calls`: emit the expansion, not the macro:

```
(:wat::rete::query-read ~fired-sym ~lit (:wat::core::PersistentMap))
```

Re-expand: `query-read` ×2, no `query fired`. Not a call-site workaround (STOP-2): the
direct path already worked; the template was emitting a form that cannot be a call.

## The four

```
probe_arc278_sift_rules::sift_rules_defsvc_counts_exact_deductions_on_{thread,process}
probe_arc278_sift_rules_arena::sift_rules_arena_counts_exact_deductions_paged_on_{thread,process}
```

**4 passed.** Fail-closed ×4 still **passed**.

## Floor

Captured `.floor/2026-09-08T18-41-57Z/` (`.floor/latest`). **Not re-run.**

```
Summary [ 194.076s] 5238 tests run: 5237 passed, 1 failed, 18 skipped
exit=100
```

The one fail is **not the stone**. ARM, verbatim:

```
wat::lint wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime
1 of 702 wat-scripts/ files do not load:
  wat-scripts/scratch-pad/probe-reland10-fire-other-thread.wat
    DefRestrictedCallerNotAllowed :wat::kernel::spawn-thread
    ArityMismatch expected 3 got 1
```

That file was a diagnostic that never ran (restricted spawn). **Deleted.** The loader
test alone then **PASS** (153s). The floor's ARM is that scratch, not the four.

Doctests: **ok.**

## STOP rows

| STOP | result |
|---|---|
| STOP-1 `"disconnected"` reported as the cause | **held.** Client Lost. Service died invoking a macro as a fn. |
| STOP-2 call-site workaround | **held.** Direct `query` in a `defn` already worked. Template now emits `query-read`. |
| STOP-3 expectation changed | **held.** |
| STOP-4 wall weakened | **held.** |
| STOP-5 "pre-existing" without bisect | **held.** Diagnosed and fixed. No bisect: expansion named the form. |

## Working tree

```
wat/query.wat    query-calls emit query-read + empty params
wat-scripts/scratch-pad/probe-reland10-*.wat    measurement (the spawn-thread one deleted)
```

Do not commit unless a later brief says to.
