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

---

## ⛔ CORRECTION 2026-08-20, same day — I PROBED HEAD NAMES AND ATTRIBUTED THEM TO FN NAMES

The census above is wrong about the collections. It probed `:wat::core::List` and
`:wat::core::LinkedList`, saw failures, and matched them to `infer_list_constructor` /
`infer_linked_list_constructor` by NAME. **Neither fn is what its name says, and neither head
exists.** Grounded on the actual dispatch table (`src/check.rs:2995–3090`):

| fn | the head(s) it actually serves | bracket today |
|---|---|---|
| `infer_list_constructor` (`:14473`) | **`:wat::core::Vector`** (`:3006`), plus the RETIRED aliases `:wat::core::vec` (`:2995`) and `:wat::core::list` (`:3085`), which emit a redirect TypeMismatch and fall through | ✅ **already wired** — `unwrap_type_param_bracket` at `:3005` |
| `infer_linked_list_constructor` (`:14886`) | **`:wat::core::List/of`** (`:3017`) — the LinkedList ctor | ❌ none |

So `infer_list_constructor` is **Vector's** inference fn carrying a legacy name, and it was
bracket-wired at step ①. And `parse_bracket_type_keyword`'s doc comment — which the census above
called stale — **is correct**; I was reading it against the wrong head.

Re-measured, with the heads that exist:

```
(:wat::core::List/of 1 2 3)                    → (1 2 3)      works; T inferred from elements
(:wat::core::List/of [:wat::core::i64] 1 2 3)  → Doctrine 1   the bracket is read as a VALUE
(:wat::core::list [:wat::core::i64] 1 2 3)     → "retired verb" redirect to :wat::core::Vector
```

**The corrected collection gap is ONE head: `:wat::core::List/of`.** There is no second List
collection — `list`/`vec` are retired spellings of `Vector`. The builder's instruction *"List and
LinkedList needs the same treatment as Tuple"* therefore resolves to a single constructor.

What survives from the census above, unchanged and re-verified: **`Some` / `None` / `Ok` / `Err`
have no param-spec form**, and that is still the larger finding, because `Option<T>` and
`Result<T,E>` are parametric types under a ruling that forbids inference.

`[[feedback_an_adjacent_implementation_is_not_the_subject]]` — a function's NAME is not its
subject. The dispatch table is. I had the arm in front of me (`":wat::core::Vector" => …
infer_list_constructor`) and read past it.

## The strike this becomes — and it is now TINY

`:wat::core::List/of` wants exactly Tuple's shape: a leading param-spec whose presence is declared,
never sniffed. **It depends on ②-i-b's `split_type_param_bracket`** — once that door exists, adding
this head is one call site in `infer_linked_list_constructor` plus its runtime twin, using the door
rather than minting a rule. Draw it AFTER ②-i-b lands, not before, or it duplicates the door.
