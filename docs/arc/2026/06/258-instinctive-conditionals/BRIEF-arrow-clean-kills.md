# BRIEF — `-> :T` annihilation, sub-strike 1: the clean kills

**Pattern to mirror:** arc 258.5b (commit `4b2d185e`) killed `-> :T` on `recv'`/`select'`
by INFERRING the type instead of ascribing it. This strike does the same for the
remaining *inferable* `-> :T` sites. `match` (hard) + `readln` (oddity) are SEPARATE
sub-strikes — OUT OF SCOPE here.

**North-star probe (RED at HEAD, verified):** `wat-tests/core/expect-no-ascription.wat`
— bare `(Option/expect (Some 5) "msg")` / `(Result/expect (Ok 7) "msg")` fail with
"expected `-> :T … — 4 args; got 2`". GREEN when the arrow is gone and T is inferred.

## The four kills

1. **`if` 5-arg `-> :T` path — DEAD CODE, delete it.** Zero corpus call sites (all `if`
   are bare post-258). Remove the `args.len() == 5` path in `infer_if` (check.rs ~3975+)
   and the matching 5-arg handling in `eval_if` (runtime.rs:7018), `eval_if_tail`
   (3240), `step_if` (22617). `if` becomes strictly 3-arg.

2. **`Option/expect`** — infer the unwrapped `T` from the `Option<T>` arg; drop `-> :T`.
   - Layout `(Option/expect <opt> <msg>)` (was `(… -> :T <opt> <msg>)`).
   - `infer_option_expect` (check.rs): require 2 args; `T` = the arg's `Option<T>` element; result `T`.
   - `eval_option_expect` (runtime.rs:11407): value at `args[0]`, msg at `args[1]` (was `args[3]`/`args[4]`).
   - Comm-position handlers that hard-code the `-> :T` layout: check.rs ~1056/1077 (`validate_comm_positions`, `items.len() >= 5`, value at `i == 3`) and ~1119–1182 (`collect_consumed_names_in_let`, value at `items[3]`) — re-index to the bare layout.

3. **`Result/expect`** — same as Option/expect, infer from `Result<T,E>`. `infer_result_expect` + `eval_result_expect` (runtime.rs:11441) + the same comm-position handlers.

4. **`apply`** — infer the result from the applied fn's return type; drop `-> :T`.
   Layout `(apply <head> <a1>…<an> <args-vec>)`. `infer_apply` (check.rs:9500) + the runtime apply path.

## The codemod (call sites)

Rewrite every call site `(<head> -> :T …)` → `(<head> …)` for the three heads:
`Option/expect` (137), `Result/expect` (62), `apply` (2) = ~201 sites. **Use the
self-hosted migration toolkit** (`wat-scripts/fixes/` fix-wat) if a verb fits the
"drop `->` + `:T` after a head" rewrite; else a surgical throwaway codemod (read →
targeted structural replace → write), deleted before commit. Drive the cascade with
the test suite (the fail-count is the progress meter). If the retirement table
(`src/remedy/retirement.rs`) should teach the old `-> :T` form → a `Remedy`, add it.

## STOP triggers
1. If a call site's `expect` arg is NOT typed as `Option<T>`/`Result<T,E>` (so `T`
   can't be inferred) — STOP, list the sites (they may need a real fix, not a codemod).
2. If `apply`'s result-inference is non-trivial (variadic args-vec shape) — STOP, report.
3. If dropping `if`'s 5-arg path breaks a caller (there should be none) — STOP, name it.

## Gate (run every row; weighed against disk)
- `cargo test --test test expect_no_ascription` → both GREEN.
- `grep -rE '(Option/expect|Result/expect|apply) -> :' wat/ wat-tests/` → **zero** non-fn `-> :T` on these heads.
- `cargo test --test test` at the 270/2 floor (the ~201 codemodded sites pass).
- `cargo test --lib` at 962/36 floor.
- `cargo build` clean.
- `match` + `readln` `-> :T` UNTOUCHED (separate sub-strikes).
