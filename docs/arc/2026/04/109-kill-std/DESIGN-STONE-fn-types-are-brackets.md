# DESIGN — arc 109: Fn TYPES ARE BRACKETS. `Fn(…)->T` dies.

Sibling to `DESIGN-STONE-all-parametrics-take-a-type-vector.md`, **sequenced AFTER its step ③**.
Same disease, smaller body, and it is what finally lets the apparatus be deleted rather than merely
left unused.

> Builder, 2026-08-20: *"the Fn side... yeah... our Fn type exprs are awful."*

## The rule

```wat
[:-> wat.type/i64]                                    ; 0-ary, returns i64 — the arrow LEADS
[wat.type/f64 :-> wat.type/i64]                       ; 1-ary
[wat.type/Keyword wat.type/String
   :-> (wat.type/HashMap [wat.type/Keyword wat.type/String])]   ; 2-ary, parametric return
[wat.type/i64 wat.type/i64 wat.type/i64
   wat.type/i64 wat.type/i64 :-> wat.type/String]     ; 5-ary
```

Near-term the heads keep their rust-ish spelling (`:wat::core::f64`); the `wat.type/` flip is later
and separate, exactly as in the sibling stone.

## ★ ALL FOUR ALREADY CHECK — measured at HEAD, including the 0-ary

```
[:-> :wat::core::i64]                                          exit 0
[:wat::core::f64 :-> :wat::core::i64]                          exit 0
[:wat::core::Keyword :wat::core::String :-> …]                 exit 0
[i64 i64 i64 i64 i64 :-> :wat::core::String]                   exit 0
```

**The destination is not something to build.** `parse_fn_type_bracket` (`types.rs:4404`, arc 251.4c,
core.typed parity) already handles every arity the builder named. This stone is a MIGRATION, not a
feature — which is why it is an order of magnitude cheaper than its sibling.

## The adoption gap — an unadopted capability, not a missing one

```
Fn(…)->  the incumbent   141 sites    wat/ 60 · src/ 48 · tests/.rs+.edn 29 · tests/*.wat 22 · wat-scripts/ 15
[… :-> …] the destination   4 sites
```

Four users against a hundred and forty-one. This is the `insert-all` shape — reachable, correct, and
nobody moved. `[[feedback_no_consumers_does_not_mean_dead]]`

## ★★ THE COMPILER ALREADY SPEAKS THE DIALECT IT IS MEANT TO REPLACE

Measured. The bracket form goes in; the keyword form comes back out:

```
in    (:wat::core::defn :user::f [g <- [:wat::core::f64 :-> :wat::core::i64]] …)
out   :expected ":wat::core::Fn(wat::core::f64)->wat::core::i64"
```

`format_type` (`check.rs:15660`) and `format_type_inner` (`:15690`) both emit `Fn(…)->`. A user who
writes the good form is answered in the bad one — the same incoherence the sibling stone's rendering
ruling ends, and it lands here identically: **rendering follows.** The 29 `Fn(…)->` occurrences in
`tests/**/*.rs` and `.edn` move with it.

## ★★ ONE APPARATUS, TWO SPELLINGS — this is the reason the stone exists

`split_type_list_top_level` carries this guard, TWICE (`types.rs:4877`, `:4913`):

```rust
'>' if prev_char == Some('-') => {}   // Fn(...)->T arrow — not a bracket close.
```

**That guard exists only because `Fn(…)->T` puts an arrow inside a string a bracket-depth scanner is
walking.** And `parse_fn_body` (`:4773`) slices `"T,U)->R"` via `find_matching_close` (`:4924`) —
another string-reparse of structure that should have been a form.

So the sibling stone and this one are the same disease: **type structure encoded in a string that
every consumer must re-parse.** Kill only the angle form and the splitter SURVIVES to serve
`Fn(…)->`. Kill both and the whole apparatus has nothing left to do:

```
lexer angle machinery (lexer.rs:792, :942)     ← sibling stone ③
split_type_list_top_level  ×2                  ← needs BOTH stones
the `->` arrow guard       ×2                  ← THIS stone
parse_fn_body + find_matching_close            ← THIS stone
three flat split(',') sites                    ← sibling stone ③
```

★ That is the payoff. Alone, either stone leaves the apparatus standing and merely under-used —
a graveyard that reads like live code, which is the thing this project keeps being bitten by.

## Semantically inert, like its sibling

`parse_fn_type_bracket`'s own doc: *"Produces the SAME `TypeExpr::Fn { args, ret }` the keyword form
yields (`parse_fn_body`), so the two unify identically."* A codemod may rewrite all 141 and nothing
downstream can tell. The churn is loud; the meaning does not move.

## Why AFTER the sibling's ③, and not before

Both dialects flow through the same splitter. Migrating them concurrently means two structural churns
crossing inside one piece of depth-tracking code, with a floor that cannot tell which strike caused
which red. Sequencing them makes each waterfall attributable.

The sibling is ~3,236 sites; this is 141. Doing the large one first also means the arrow guard is the
LAST thing standing when this stone lands, so its deletion is a clean subtraction rather than a
negotiation.

## Scope

1. **Accept is already done** — no parser work. Verify, do not build.
2. **Codemod 141 sites** — wat-fix (R21) for `.wat`; a separate pass for `src/`'s 48 Rust string
   literals. ⚠ The discriminator: `Fn(` inside a **type keyword**. `:wat::core::Fn` is the only head
   that spells a type this way; ordinary parens are untouched.
3. **Rendering follows** — `format_type` / `format_type_inner` emit `[args :-> ret]`; 29 goldens move.
4. **Then DELETE** — `parse_fn_body`, `find_matching_close` (if it has no other caller — MEASURE, do
   not assume), the `->` arrow guard in both splitter copies, and the `:wat::core::Fn` keyword head.
5. **Reject** — `Fn(…)->` becomes a check error naming the bracket form as the remedy.

### Out of scope — affirmatively cut

- The `wat.type/` Clojure spelling. Later, and shared with the sibling.
- `[A :-> B]` syntax itself. It exists, it is correct, it is not being redesigned.

## The four questions

- **Obvious?** YES — `[args :-> ret]` reads as a signature; `Fn(A,B)->C` reads as a string that got
  into a type.
- **Simple?** YES, and it SUBTRACTS: two parse fns, two arrow guards, one keyword head.
- **Honest?** YES — and today's state is the counterexample: the compiler accepts the bracket and
  answers in `Fn(…)->`. Ending that is the point.
- **Good UX?** YES — one bracket grammar for function types, one for type-params, distinguished by
  position exactly as the sibling stone's forms are.
