# DESIGN — STONE: a trailing run of PAIRS, and positionals stop riding unconditionally

> **Builder:** *"we need map literal vals to be expressed as pairs on the same line?.."*
> ```
> (:wat::hashmap::assoc
>   (:wat::core::HashMap :- [:wat::core::keyword :wat::core::i64])
>   :k x)
> ```

## THE GENERALISATION — one rule, and it reproduces every shape ruled so far

> **A maximal TRAILING run of `(keyword, value)` pairs is laid out ONE PAIR PER LINE, each pair
> whole on its line, values aligned. Everything before the run follows the ordinary leading-atom
> rule.**

```wat
(assoc m :k x)                              m is compound -> breaks;  `:k x` ONE LINE
(get X :k)                                  trailing run is [:k] — ODD, NOT a run;  :k own line
(Unreadable :file p :reason m :line n :col c)   four pairs, one per line, values aligned
(defservice name :satisfies X :durable [])  name is an ATOM -> rides;  then pairs
(HashMap :- [K V] :some-kw "some-string")   type-args slot, then ONE pair on one line
```

★ **All five are the builder's own rulings, and one rule produces all of them.**

## ⛔ WHAT THIS FIXES — two defects, both mine

### 1 — the kwargs rule fires where there are no keyword ARGUMENTS

`:k` in `(assoc m :k x)` is a **map key — a keyword VALUE**, not a parameter name. The shipped rule
has no test for this and fired anyway.

⭐ **The substrate already owns the predicate**, and the formatter must not invent a second one.
`src/check.rs:13444`, whose own comment says *"the same kwargs-vs-positional test the eval arm +
`build_insert_fact` use"*:

```rust
let is_kwargs = rest.len() >= 2
    && rest.len().is_multiple_of(2)
    && rest.iter().step_by(2).all(|a| matches!(a, WatAST::Keyword(_, _)));
```

**Applied to a TRAILING run** rather than the whole argument list, that is exactly the rule above —
and it is why `(get X :k)` correctly has no run while `(assoc m :k x)` does.

⚠ A second, disagreeing definition of "what is a kwarg call" would be a formatter that argues with
the language. Use the substrate's shape.

### 2 — STOP-3's "a positional must RIDE" was wrong, and it is my third ambiguity this session

I wrote *"a positional before the first keyword must still RIDE the head line"*, thinking of
`defservice`'s atom name. Applied to a COMPOUND positional it produced the align-under-the-paren
shape the builder rejected, and the arc's worst remaining line:

```
  (:wat::hashmap::get (:wat::hashmap::assoc (:wat::core::HashMap :- [...])
                        :k x) :k))          132 columns
```

**Positionals follow the ordinary leading-atom rule** — atoms ride, the first compound breaks. The
`defservice` shape still works because its positional is an atom. **Nothing special is needed; the
special case WAS the bug.**

★ Third time this session an acceptance row of mine could be satisfied by the defect:
the ret-spec (*"on its own line"*), the arg-spec, and now this.
`[[feedback_an_acceptance_row_is_a_pin_unless_it_derives_its_bar]]`

## THE ACCEPTANCE

```
1  ★ (assoc m :k x)        — m breaks, `:k x` on ONE line
2  ★ (get X :k)            — no pair run; :k on its own line; NOTHING rides that is compound
3  ★ the four-pair record  — one pair per line, values aligned (must not regress)
4  ★ defservice            — the atom positional still rides (must not regress)
5  ★ (HashMap :- [K V] :some-kw "s") — type-args slot, then one pair on one line
6  idempotent throughout
7  ★★★ the 614 doc examples again: changed / still-over-120 / worst shape verbatim.
   The previous run left ONE line at 132 columns, and it is exactly defect 2 —
   THAT LINE MUST BE GONE.
```

⛔ **Row 7 is not a repeat of a passing row.** Last run's single remaining over-120 line *is* the
defect this stone fixes. **If it survives, the stone did not land**, whatever else is green.

## OUT OF SCOPE

- **Level-2 cross-sibling-call alignment** — still open, still named.
- **A one-liner carve-out** for short pair runs — *"arguably one-liner worthy… exploded for now."*
- **The emitter's comment-indent defects** — corpus-only; the builder has re-scoped corpus files to
  a test bed, so they matter, but after this.
