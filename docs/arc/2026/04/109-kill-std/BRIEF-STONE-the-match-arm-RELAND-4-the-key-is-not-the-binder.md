# BRIEF — RELAND 4: a map-pattern KEY is a declared field; a binder name is not

> Floor: **5208 passed, 16 failed** (108 → 38 → 16). Probe rows 1-4 still PASS, `#[ignore]` = 0.
> RELAND 3's discriminator held and is worth keeping: `tagged-template-pattern?` returns false when
> the pattern is `unquote-splicing-form?`, so a spliced CALL `` `(~@step ~a) `` is not an arm while a
> spliced ARM `` `((~ctor binders) body) `` still is. Structure alone cannot separate them; the
> unquote FLAVOUR can. Do not undo that.

## THE DEFECT — the codemod writes the BINDER into the key slot

```
malformed match form: map-pattern key `:_cur` is not a field of
                      `:usr::my-sift::SiftRulesResponse::Deductions`
```

`(Variant _cur)` → `[Variant {:_cur _cur}]`. In the positional form the binder name is **arbitrary**.
In a map pattern the key must be the **declared field** and the binder is the value. The codemod
carries a hardcoded table for a handful of core variants —

```clojure
":wat::core::Some" (Vector :- [String] "value")
":wat::core::Ok"   (Vector :- [String] "value")
```

— and for everything else assumes **binder-name == field-name**. That is a guess, and it is the
third time today a heuristic has stood in for an authority that exists elsewhere.

★ **The failures are not random.** Every broken key is `_`-prefixed: `:_cur`, `:_err`, `:_bytes`.
That is the "I do not use this value" convention, which makes those binders *systematically* unlike
their field names. The guess works exactly where the programmer happened to echo the field.

## THE AUTHORITY EXISTS — and may not be reachable from where the codemod stands

`:wat::runtime::field-names-of` is registered and wat-callable. **Two things are unverified and must
be established before relying on it:**

1. whether it answers for an ENUM VARIANT (as opposed to a record/struct), and
2. whether the type is registered AT CODEMOD TIME. `:usr::my-sift::SiftRulesResponse` is synthesized
   by `defservice`; the codemod reads target files as TEXT and does not expand them, so a
   generated type may not exist to be asked.

## ⛔ THE RULING THIS NEEDS — not a strike decision

Pick one and say why:

- **(a) load-then-rewrite** — the codemod loads the target program (expanding macros), queries
  `field-names-of`, then rewrites. Correct for every type, but it changes the tool's nature from a
  text/form rewriter into a compile-then-rewrite tool, and a file that does not currently load
  cannot be migrated.
- **(b) the macro emits the new form** — `defservice`'s templates already emit
  `[~op-variant-kw {:req ~req-binder} …]`. Generated arms need no codemod at all; only USER-WRITTEN
  arms matching a generated type remain, and those are the ones failing.
- **(c) refuse and report** — the codemod converts only arms whose field names it can resolve, and
  emits a list of the rest for a human/LLM to write. Smallest tool, honest, leaves a residue.

**Do not pick by convenience.** (c) is the conservative one and matches `where_tree`'s own doctrine
— over-approximate never under-approximate — but it leaves work undone; (b) is cheapest if the
failing population really is only user-written arms over generated types. **Measure which before
choosing.**

## THE REMAINING 16 — do not assume one root

```
13  wat::services   peers_bijection ×4 · sift_rules ×4 · sift_rules_arena ×4 · defservice_op_enum
                    — the key/binder defect above, PLUS `defservice — program body eval failed`
                      on peers_bijection, which may be a different cause. DIAGNOSE SEPARATELY.
 2  wat::rete       wat_scripts_grid_axes_live ×2 — UNDIAGNOSED
 1  wat::lint       every_wat_scripts_file_loads — UNDIAGNOSED (was an UnresolvedReference two
                    relands ago; confirm whether it is the same file and the same cause)
```

## STOP TRIGGERS

- **STOP-1 — a key is invented.** If the field name cannot be resolved, the arm must be REPORTED,
  never guessed. A wrong key type-checks as a *different* field or fails loudly; both are worse than
  an unconverted arm.
- **STOP-2 — RELAND 3's unquote/unquote-splicing discriminator is weakened.** It is what keeps
  `->>` working; regressing it re-breaks the threading cluster.
- **STOP-3 — the 3 non-services failures are folded into the key/binder root.** Diagnose each.
- **STOP-4 — hand-editing a `.wat` form.** R21 unchanged; `.rs` strings and the `.jsonl` fixtures
  RELAND 3 found remain the string exception.
