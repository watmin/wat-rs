# DESIGN — ③b-ii ⑤: the composition door gets a wat surface

**Ruled in the main chat 2026-09-10**, mid-landing, at WIP `085f3ed3e` (NOT pushed).
The builder's words: *"more intrinsics being demanded is a strong sign — compose-variant feels
appropriate."*

## The signal

Arc 255's dot-flip minted `:wat::runtime::variant-parent-of` so wat could ASK what a variant is.
**Only the decomposition side got a surface.** The composition side stayed Rust-only
(`identifier::compose_variant`), so wat code that needs to BUILD a variant name has no door — and
`wat/service.wat` does exactly what a language without a door forces: it hand-rolls the separator
inside a string.

```wat
admin-init-kw (:wat::keyword::from-string
                (:wat::string::interpolate "{b}::Admin::Init" :b fqdn-base))
```

Thirteen of these, composing every service's `Admin`/`Status`/`Reply` variant heads. They are
invisible to the corpus codemod because `rename-keyword-exact` **refuses to splice inside a string
literal by design** — arc 296 N RELAND 8 is the incident that made it position-aware, after a
whole-file rename corrupted `Option`'s unit variant by treating a declaration slot as a use site.
Weakening that protection to reach these thirteen would trade a known catastrophe for a convenience.

★ **The demand for the intrinsic IS the finding.** A separator hand-spelled in a string literal is
what a missing door looks like from the far side. `[[feedback_i_named_a_limit_that_was_a_missing_door]]`
— the second time in this campaign that "the codemod cannot reach X" turned out to mean "the
substrate has no verb for X".

## The verb

```
(:wat::runtime::compose-variant <enum-keyword> <variant-keyword>) -> :wat::core::keyword
```

Mirrors `variant-parent-of` exactly — same home, same literal-or-computed door (arc 166's
`eval_lookup_define`, NOT `is-type?`'s literal-only one; `service.wat` passes a COMPUTED enum
keyword, so a literal-only door would be unusable by its only caller — the mistake ① already made
once in this arc and had to fix).

```
variant-parent-of :  keyword          -> (Option :- [keyword])     DECOMPOSE
compose-variant   :  keyword, keyword -> keyword                   COMPOSE
```

The body is `wat_reader::identifier::compose_variant` and nothing else. That is the point: the
separator lives in ONE file, and now wat reaches it rather than re-spelling it.

⚠ **It composes; it does not validate.** `variant-parent-of` answers `None` for a non-variant
because it consults the type env. `compose-variant` is the inverse of a pure string operation and
must stay total — a caller composing a name for an enum that does not exist gets a name that does
not resolve, which is the caller's error and surfaces where the name is used. Making it partial
would be a different verb (`compose-variant-checked`) and no caller has asked for one.

## What `service.wat` becomes

```wat
admin-init-kw (:wat::runtime::compose-variant
                (:wat::keyword::from-string (:wat::string::interpolate "{b}::Admin" :b fqdn-base))
                :Init)
```

★★ **After this, `wat/service.wat` contains no variant separator at all.** `"{b}::Admin"` is a
genuine namespace join — the enum's own path — and the variant leaf is a keyword the substrate
joins. The file stops knowing what the separator is, which is the property the whole campaign has
been buying everywhere else.

One site (`status-started-str`) wants a String rather than a keyword; it takes
`:wat::keyword::to-string` over the composed keyword.

## Acceptance

```
the doc contract: @Purity Pure · @Determinism Deterministic · @Totality Total · @Category Reflection
purity_mandated_examples => a RUNNABLE @example is MANDATORY (Pure ∧ Deterministic)
  one literal, one COMPUTED (keyword::from-string) — the computed row is what service.wat needs
13 string-literal templates in wat/service.wat carry no variant separator afterwards
the corpus loads: the 818 check errors (456 of them "arm head is not namespaced") go to 0
floor via scripts/floor.sh · clippy -D warnings = 0
```

## ⚠ This stone lands ON a held landing

`085f3ed3e` is a local WIP save point, unpushed, carrying the flipped doors, the rewritten corpus
and three of the four classes fixed. This stone is the fourth. Nothing is pushed until the floor is
green, and the whole sequence commits as the landing it is.
