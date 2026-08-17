# ⛔ NOTE (arc 255) — an INTRINSIC cannot be passed as a value; a USER fn can. Same syntax, two answers.

**Found 2026-08-16, live, by probe.** The builder's read on seeing it: *"hrm... this feels... like a
bug?...."* — and it is. Recorded here rather than in the stone that surfaced it, because the cause is
255's and the fix is 255's.

## The measurement — two runs, one difference

```wat
(:wat::core::defn :user::my-str [x <- :wat::core::i64] -> :wat::core::String
  (:wat::core::str x))

(:wat::core::mapv :user::my-str   (:wat::core::Vector :wat::core::i64 1 2 3))   ;; → ["1" "2" "3"]
(:wat::core::mapv :wat::core::str (:wat::core::Vector :wat::core::i64 1 2 3))   ;; → CHECK ERROR
```

The error, verbatim:

```
#wat.check/NoMatchingClauseAtCallSite {
  :message "no clause of `:wat::core::mapv` matches arity 2 with types
            [:wat::core::keyword, :wat::core::Vector<wat::core::i64>];
            clauses attempted: (2: [:wat::core::Fn(T)->U, :wat::core::Vector<T>]);
                               (2: [:wat::core::Fn(T)->U, :wat::stream::Stream<T>])"
  :called-arg-types [":wat::core::keyword" ":wat::core::Vector<wat::core::i64>"] }
```

**`:user::my-str` is an `Fn(T)->U`. `:wat::core::str` is a `keyword`.** Identical syntax, identical
position — the answer depends only on whether the callee happens to be written in wat or in Rust.

## The cause is this arc's thesis, one layer over from where the arc states it

A user `defn` lands in `sym.functions` carrying a `TypeScheme`, so the checker can resolve the
keyword in value position **to** something. An intrinsic is a match arm. This arc's own DESIGN says
it:

> *"Rust builtins … registered **nowhere** — a 454-arm compile-time `match`."*

There is no `Function` entry for `:wat::core::str`, so there is nothing to resolve it to, and the
checker correctly reports what the token literally is: a keyword.

255 frames its goal as *"parity with user forms — the reflection is seamless."* **This is the same
gap in the TYPE system rather than the reflection system: parity with user forms in VALUE POSITION.**

## ★ WHY THIS ONE MATTERS MORE THAN THE OTHER THREE

`NOTE-a-capability-declaration-cannot-be-verified-to-name-anything.md` lists three consumers waiting
on the membership door: the undefined-func class (`+'2`, `Bogus`), the annotation-position gap
(109's sibling note), and W1's capability wall. All three are **substrate-internal** — real, but
invisible to anyone writing wat.

**This is the fourth, and it is the first a user trips over in ordinary code.** `(mapv str xs)` is
what every Clojure programmer writes first. It does not compile, and the diagnostic says `str` is a
keyword — which is true, unhelpful, and impossible to act on without knowing the substrate's
internals.

That makes it the arc's payoff *demonstrable in three lines*, which none of the other three are.

## What it costs today

The workaround is a lambda, and it works:

```wat
(:wat::core::mapv (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::String
                    (:wat::core::str x))
                  xs)
```

Measured green. But note where that lands: the **stdlib's own source** has to write the wrapper —
`wat/string.wat`'s `join<T>` (the 279.2 → chain-D consumer) cannot say `(mapv str xs)` and must
carry the lambda plus a comment explaining why, so a later reader does not "simplify" it into
something that will not compile.

**An ergonomic gap visible in the stdlib's own text is a gap the language is teaching.**

## Scope — what this note does NOT claim

- **It does not claim the checker is wrong.** Given no `Function` entry, `keyword` is the honest
  answer. The defect is upstream: the entry does not exist.
- **It does not rule the fix.** Whether an intrinsic becomes first-class-as-a-value falls out
  automatically once builtins register into `sym.functions` (slice 255.1), or whether it needs
  something further, is unmeasured. **Do not assume 255.1 closes it — re-probe these two lines after
  the carve and find out.** That re-probe is cheap and is the honest gate.
- **It does not touch `join`.** Chain-D ships with the lambda; this is not its blocker.

## Kin

- `NOTE-a-capability-declaration-cannot-be-verified-to-name-anything.md` — consumers 1–3, same door.
- `NOTE-arc-255-IS-HALF-BUILT-the-june-registry.md` — what is actually built, and the unruled
  (a)/(b) fork this note does not re-open.
- Task #97 — *a colon-quoted symbol in VALUE position leaks a function's opaque clause table into
  user output.* **Same position, adjacent defect**: #97 is about what a user *fn* keyword renders as;
  this note is about what an *intrinsic* keyword type-checks as. Do not merge them — different
  populations, different fixes — but whoever touches value-position keywords should read both.
- `CHAIN-rendering-before-the-string-home.md` — stone D, the consumer that surfaced this.

---

> Found because the builder distrusted a probe result that I had already written off as "that's just
> how it is." The asymmetry took one extra probe to prove and would otherwise have shipped as a
> lambda in the stdlib with no note attached. **A workaround that works is the easiest kind of defect
> to stop seeing.**
