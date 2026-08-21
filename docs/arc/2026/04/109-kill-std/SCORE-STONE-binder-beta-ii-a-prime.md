# SCORE — arc 109, β-ii-a′: `defservice`'s binder becomes the source of truth

Rider: one flight, ~7 min, no STOP fired. Every row re-run by the orchestrator's own hand.

| # | what | result |
|---|---|---|
| 1 | a real parametric service declares with `:- [K V]` | ✅ runs |
| 2 | ★ the OLD `<K,V>` spelling still expands | ✅ — `lru-svc<K,V>` in the stdlib, and an adapted copy |
| 3 | ★★ **the binder's CONTENTS are load-bearing** | ✅ perturbing to `:- [X Y]` FAILS: *"body produces `lru2::State<K,?364>`; signature declares `lru2::State<…>`"* |
| 4 | `:- []` is the monomorphic state | ✅ after a fix — see below |
| 5 | floor | ✅ **4855/4855, 0 FAIL** |
| 6 | clippy `-D warnings` | ✅ 0 |
| 7 | zero golden churn | ✅ `wat/service.wat` is the only modified file |

Predicted 10–20 min; the rider took ~7. Its report was the most careful of the session: it verified
every head it used is already used **inside this macro body**, line by line, rather than trusting my
corpus-wide counts — which is a stronger check than the brief asked for.

## Row 3 is the row that matters

Rows 1, 2, 5 and 7 can all be green while the binder is silently ignored — the old path would carry
everything. The perturbation is what proves otherwise: changing `:- [K V]` to `:- [X Y]` makes the
generated `State` type disagree with the body that references `<K,V>`.

⚠ And one test I nearly mis-scored: `svc_mono` declares `:- []` while its body still references
`<K,V>` five times, and it type-checks CLEAN. That is **not** evidence the binder works — it is the
pre-existing *"a type name is validated in CALL position, never in ANNOTATION position"* gap
(`NOTE-type-annotation-names-unchecked.md`) swallowing the mismatch. The same filed hole that made
an earlier acceptance row of mine wrong. Row 3 is the honest instrument here; row 4 is not.

## ⛔ A DEFECT THE RIDER SHIPPED, AND THE RULE IT BROKE

`fqdn-parametric?` was written as *"a binder was written"*:

```clojure
fqdn-parametric? (if binder-present? true (string::ends-with? fqdn-str ">"))
```

So `:- []` meant **parametric with zero params**, and the macro crashed:
`string::subs — "index out of range: start=0, end=-1, char-length=0"` — a downstream reader trusted
the flag and did bracket arithmetic on `""`.

That is the empty-rung rule, broken in the one place it had not yet been stated: **an empty binder
is a first-class way to say monomorphic**, exactly as `(Tuple :- [])` is a first-class empty tuple.

## ★ AND THE BUILDER TURNED THE FIX INTO THE DESIGN

> *"so we need to make `:- []` the default assumed state, users can override with their own option
> expr?… let's make the `:- []` assumed default state… that feels like the better path"*

The narrow fix made `:- []` and a bare name reach the same state. The ruling goes further and
deletes the concept: **there is no monomorphic-vs-parametric distinction — there is a param list,
and it is usually empty.** So the block now has ONE source of truth and everything hangs off it:

```
fqdn-base        the name with any <…> stripped     — uniform, no branch on the binder
fqdn-tp-syms     binder → ast->children · legacy name → parsed · neither → []
fqdn-parametric? (not (empty? fqdn-tp-syms))        — DERIVED, not tracked
fqdn-tp          derived from the syms, ONE derivation for BOTH spellings
```

★ The legacy spelling now round-trips **name → syms → `"<K,V>"`**, so the compatibility shim the ~50
consumers read is produced by the same code in both paths. There is no second derivation that can
drift. `fqdn-parametric?` survives only as a convenience for its 7 readers; β-ii-b retires them.

## Two walls hit while collapsing it, both F5

The pure-combinator allow-list refused **`:wat::core::string::index-of`** at definition — the whole
stdlib down, same failure mode as the earlier top-level `defn`. Replaced with the arithmetic the
original code already used: `fqdn-base`'s LENGTH is the `<` index. No new primitive, per the design's
amendment.

Running tally of F5 refusals this stone: a top-level `defn`, a bare primitive keyword passed to
`mapv`, and `string::index-of`. **The allow-list is narrow and it is not documented anywhere a rider
can read** — that is worth its own note before β-ii-b sends someone back into this macro.

## What β-ii-a′ did NOT do

- **The ~50 emissions** still read the derived string. β-ii-b.
- **`proto-tp`** untouched — a `:satisfies` surface still names its params in its own keyword.
- **`:741`'s substring transport-param test** untouched — β-ii-d, and now cheap: it becomes a
  membership test over `fqdn-tp-syms`.
