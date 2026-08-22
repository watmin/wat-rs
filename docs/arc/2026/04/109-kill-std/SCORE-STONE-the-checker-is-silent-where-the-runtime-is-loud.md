# SCORE — `--check` is now as loud as the runtime on a malformed `fn`. SHIPPED.

Every row independently re-run by the orchestrator on a release build it built itself.

| # | row | result |
|---|---|---|
| 1 | `(fn :foo …)` | ⛔ FAILS — *"expected a vector `[name <- :T ...]` as the args-vector; got keyword"* |
| 2 | `(fn 42 …)` · `(fn "s" …)` | ⛔ FAILS — *"… got int"* · *"… got string"* |
| 3 | `(fn)` · `(fn [x])` · `(fn [x] ->)` | ⛔ FAILS — *"expected [name <- :T ...] -> :Ret body ...; got 0/1/2 element(s)"* |
| 4 | well-formed fn | ✅ checks |
| 5 | ★ `(fn :- [T] [x <- :T] -> :T x)` — the γ-i binder | ✅ checks |
| 6 | ★ `(fn {:doc "m"} …)` — metadata preamble | ✅ checks |
| 7 | the runtime is unchanged | ✅ `git diff src/function/eval.rs` EMPTY |
| 8 | `SigParse::SilentReject` no longer exists | ✅ `grep -rn SilentReject src/` returns nothing |
| 9 | `src/check.rs` | ✅ zero diff |
| 10 | floor | ✅ **4854/4854**, 0 FAIL, 19 skipped |
| 11 | clippy `-D warnings` | ✅ 0 |

**Every message is `eval.rs`'s own wording.** No new diagnostic text was authored — the sister
sequences finally agree, in the same words.

## ⛔ STOP-1 did NOT fire — the exemption was pure debt

The six macro templates in `wat/core.wat`, `wat/service.wat` and `wat/bracket.wat` that quasiquote
`(:wat::core::fn …)` — including `core.wat:1278`, where the entire args-vector slot is an unquote
node — do **not** route through `infer_fn` with a non-Vector slot 0. The floor is green.

That was the named risk that would have resized the stone, and the DESIGN said in those words that
**the floor answers it and no reading of mine could**. It answered.

## The floor baseline moves 4855 → 4854, and it is fully accounted

Two test fns removed, one added:

```
- infer_fn_fewer_than_3_args_returns_fresh_placeholder     → + infer_fn_fewer_than_3_args_returns_malformed_form
- infer_fn_non_vector_args_returns_silent_placeholder      → deleted outright
```

The deletion was authorized by the brief and earns it twice: it pinned the behaviour being removed,
AND it called `infer_fn` directly with a synthetic array, so it never proved anything about what a
caller sees. Replaced by caller-level `.wat` probes rather than another unit test making the same
mistake. **A count that drops must be explained, not observed** — the arithmetic is 2 − 1 = 1.

## The rider's honest delta

`infer_fn` receives no `list_span` (unlike `eval_fn`, which takes one), and threading one in would
have touched `check.rs` — forbidden by STOP-3. It derived the span from `sig_args`/`args`, falling
back to a Rust-source span only for the genuinely-empty `(:wat::core::fn)`. It reported this as a
decision it had to make rather than presenting it as specified.

★ It also found something the DESIGN had wrong: normal execution **already runs a full startup
type-check**, so the old bug meant a malformed `fn` failed at *call* time deep in `eval_fn`; it now
fails at *load* time with identical text. Strictly better than the DESIGN predicted.

And it caught its own first-draft comment still containing the literal string `SilentReject`, which
would have made row 8's grep lie. It fixed that before reporting.

## What this stone actually removed

Not a bug — an **exemption**, granted on a premise the disk refutes. Both silent paths were justified
by comments (*"parse won't even call check"*, *"handled by other checker arms"*) that are false at the
only two call sites, both `":wat::core::fn" => infer_fn(…)` from a dispatch that had already matched
the head. With `SilentReject` deleted the silent state is not caught — **it is unrepresentable**,
because there is no arm left to fall into. Convention → no-form, one rung up.
`[[feedback_a_comment_can_ship_a_gap_as_a_law]]`
