# ⛔ NOTE (arc 109) — HALF the parametric constructors never got the bracket. It blocks ③, not ②.

**Filed 2026-08-20. MEASURED, head by head, against `target/release/wat` at `a254d0d7e`.**
Asked "is Tuple the last collection that needed this change?" — the answer is no, and the gap is
bigger than collections.

## The census

Every head probed with `(:wat::core::HEAD [types] values…)` and its value printed:

```
✅  (:wat::core::Vector [:wat::core::i64] 1 2 3)              → [1 2 3]
✅  (:wat::core::HashSet [:wat::core::i64] 1 2)               → #{1 2}
✅  (:wat::core::PersistentVector [:wat::core::i64] 1 2)      → #wat.core/PersistentVector [1 2]
✅  HashMap · Tuple · PersistentMap                            (①/①b, same shape)

❌  (:wat::core::List [:wat::core::i64] 1 2 3)                → Doctrine 1: ':wat::core::i64' is a TYPE keyword, not a value
❌  (:wat::core::LinkedList [:wat::core::i64] 1 2 3)          → Doctrine 1, same
❌  (:wat::core::Some [:wat::core::i64] 1)                    → ArityMismatch: expected 1 argument
❌  (:wat::core::None [:wat::core::i64])                      → MalformedForm
❌  (:wat::core::Ok  [:wat::core::i64 :wat::core::String] 1)  → ArityMismatch: expected 1 argument
❌  (:wat::core::Err [:wat::core::i64 :wat::core::String] "e")→ ArityMismatch: expected 1 argument
```

**Six wired, six not.** `src/check.rs` holds TEN `infer_*_constructor` fns; the bracket work
reached six of them — three at the dispatch site via `unwrap_type_param_bracket`
(`check.rs:3005/3145/3207`), three inside the fn via `is_type_bracket_candidate`
(`check.rs:14062/14165/14330`). `infer_list_constructor` (`:14473`),
`infer_linked_list_constructor` (`:14886`), `infer_some_constructor` (`:2172`),
`infer_ok_constructor` (`:2223`) and `infer_err_constructor` (`:2275`) were never touched.

## ★ The Option/Result family is the real finding, not List

`Option<T>` and `Result<T,E>` **are parametric types.** The builder's ruling (`3821db4ba`) is
*"the vec-of-types param is mandatory for any parametric type — no inference"*, and today
`(:wat::core::Some 1)` infers `T` from the value. That is precisely the inference the ruling
annihilates, and it is not a collection, so a census framed as "which collections?" misses it.
Any wall that says "a parametric constructor must declare its param-spec" fires here.

## Two things measured in passing

1. **`List` takes no leading type keyword either.** `(:wat::core::List :wat::core::i64 1 2 3)` also
   trips Doctrine 1 — so it is not merely bracket-less, it has no type-declaring form at all.
   ⚠ `parse_bracket_type_keyword`'s doc (`check.rs`) claims it mirrors *"the leading-type-keyword
   read already used by `infer_hashset_constructor` / `infer_list_constructor`."* For `List` that
   is not what the binary does. Stale or about a different path; do not build on that sentence.
2. **`(:wat::core::List …)` has ZERO corpus uses** in `wat/` or `tests/`. So the head may be
   effectively dead; whoever closes this should ask whether `List`/`LinkedList` want the bracket or
   want retiring, and not assume the first.

## Sequencing — this does NOT block ②-i-b or ②-iii

- **②-i-b** ships the `:-` production and the renderer; it changes no constructor's arity contract.
- **②-iii** rewrites `Head<A,B>` in TYPE positions. `Some`/`Ok`/`Err` carry no angle brackets at a
  call site, so the codemod never reaches them. `Option<T>`/`Result<T,E>` as *annotations* migrate
  fine — that is the renderer's path, already working.
- ⛔ **③ IS BLOCKED BY THIS.** ③ makes the param-spec MANDATORY and has the checker write each fix
  as a `remedy`. Firing that wall on a head whose constructor cannot ACCEPT a param-spec ships an
  error with no compliant form — the user is told to write something the substrate rejects. **Close
  this before ③, or ③'s wall must be scoped to the six that can comply and say so out loud.**

`[[feedback_a_pass_answers_only_the_question_the_instrument_asks]]` — ①b's SCORE said "all six
constructors take the bracket" and was true. Six was never the whole population; nobody asked.
