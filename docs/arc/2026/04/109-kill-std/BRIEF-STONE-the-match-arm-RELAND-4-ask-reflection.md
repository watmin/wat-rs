# BRIEF — RELAND 4: the codemod ASKS, it does not guess

> ⛔ **SUPERSEDES `BRIEF-STONE-the-match-arm-RELAND-4-the-key-is-not-the-binder.md`.** That brief put
> a three-way ruling to the builder — load-then-rewrite / macro-emits / resolve-or-report. The
> builder refused all three and ruled the capability instead. **296 stone L landed**
> (`480f38d05`): `:wat::runtime::type-of` answers with `#wat.runtime/TypeInfo`, and an Enum body
> carries its variants with **declared field names in declaration order**. The ruling dissolved;
> the codemod asks.

## THE DEFECT (unchanged)

`(Variant _cur)` → `[Variant {:_cur _cur}]`. In the positional form the binder is arbitrary; in a map
pattern the key must be the **declared field**. The codemod carried a hardcoded table for a few core
variants and otherwise assumed binder-name == field-name. Every broken key is `_`-prefixed —
`:_cur`, `:_err`, `:_bytes` — the "I do not use this value" convention, i.e. exactly the binders
least likely to echo their field.

## THE WORK

**Delete the hardcoded field table. Ask `type-of`.** For each variant arm being converted, resolve
the variant's declared field names, in order, and pair them with the positional binders.

```clojure
(:wat::runtime::type-of :probe::Box)
;; :body -> #wat.runtime/TypeBody.Enum {:variants [#wat.runtime/TypeVariant
;;            {:name :Full :fields [#wat.runtime/TypeField {:name :payload …}]} …]}
```

## ⛔ THE ONE THING THAT IS NOT YET ESTABLISHED — establish it FIRST

**Measured:** `type-of` answers for a type declared in the program being run (stone L's own fixture
declares `:probe::Box` in-file and is answered), and for stdlib types.

**NOT measured, and it is the crux:** the codemod loads the STDLIB and then reads its targets as
**text**. A type declared *in the file being rewritten* — including one synthesized by a macro like
`sift-rules-defsvc` — is therefore **not registered** when the rewrite runs.

**So the strike's first act is to determine whether the target's DECLARATIONS can be registered
without type-checking its bodies.** Declarations register at load; the bad arms are a CHECK error in
a body. If declarations-only registration is possible, the codemod becomes: register the target's
declarations → ask `type-of` → rewrite the text. If it is not, STOP-2.

## STOP TRIGGERS

- **STOP-1 — a key is invented.** If a variant's fields cannot be resolved, the arm is **REPORTED**,
  never guessed. This is the whole stone. ★ A wrong key does not reliably fail: `:_cur` errored only
  because no such field exists — had the variant HAD a field named `_cur`, the arm would have bound
  the **wrong field silently**. Reporting cannot do that; guessing can.
- **STOP-2 — declarations cannot be registered without checking bodies.** Then the codemod cannot
  ask about in-file types, and the honest shape is: ask for what IS reachable (stdlib + already
  loaded), report the rest. Say which of the 13 services failures fall in each bucket — do not
  silently fall back.
- **STOP-3 — RELAND 3's discriminator is weakened.** `tagged-template-pattern?` returns false when
  the pattern is `unquote-splicing-form?`; that is what keeps `->>` working. Regressing it re-breaks
  the threading cluster.
- **STOP-4 — the hardcoded field table survives.** It is the guess wearing a table's clothes, and it
  is now redundant for every type `type-of` can answer for. If any entry must stay, name which and
  why.
- **STOP-5 — the 3 non-services failures are folded in.** `grid_axes` ×2 and the wat-scripts loader
  are still undiagnosed; diagnose each separately.

## EXPECTATIONS

| # | what | expected |
|---|---|---|
| 1 | floor | the delta named, not a total — the tree is red at 16; say which of those 16 close and whether any new one opens |
| 2 | the 4 probe rows | still PASS, `#[ignore]` 0 |
| 3 | ⛔ the hardcoded table is gone | `grep -c '"value"' wat-scripts/fixes/match-arm-to-bracket-map-pattern.wat` → 0, or each survivor named (STOP-4) |
| 4 | ⛔ keys are DECLARED fields | no `{:_` key remains anywhere: `grep -rc '{:_' --include=*.wat` → 0 |
| 5 | unresolvable arms REPORTED | a list, not a guess — and it may legitimately be empty |
| 6 | the spliced-arm case still works | `probe_arc170_c2_mixed_macro::mixed_via_macro_runs` PASS (STOP-3) |
| 7 | clippy | 0 |
