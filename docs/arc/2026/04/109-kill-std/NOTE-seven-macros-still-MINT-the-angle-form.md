# NOTE (arc 109) — seven macros still MINT the angle form by concatenation

**Filed 2026-08-23, after ②-iii shipped (`2a0d7fa2e`).** A POINTER with a measured population.
This is blocker 3d's real shape, and 3d was proven NOT to block the migration — the corpus floors
green at 4882/4882 with all seven live.

## ⛔ First — the count that was wrong twice

I reported *"two angle forms left in `wat/`"*. **It is seven**, plus one that was a different thing
entirely and is now fixed.

The regex was `:wat::[a-z:]+::[A-Za-z]+<[^>]*>` — it requires `<…>` **contiguous**. Every one of
these names is assembled by `string::concat`, so the `<` and the `>` live in **separate string
literals** and the pattern walked straight past them:

```clojure
(:wat::core::string::concat ":wat::kernel::Peer<"
  (:wat::core::string::concat sp-out-str
    (:wat::core::string::concat "," (:wat::core::string::concat sp-in-str ">"))))
```

`[[feedback_state_what_the_instrument_can_see_before_quoting_it]]` — the instrument selected
*contiguous* angle spellings and I quoted it as *all* angle spellings. The honest probe is
`"…Name<"` — the opening literal alone.

## The population — seven sites, two files

```
wat/bracket.wat   ":wat::kernel::Peer<"        ":wat::core::Option<"
                  ":wat::bracket::PoolMsg<"    "wat::bracket::PoolMsg<"
wat/service.wat   "wat::kernel::Peer<"         "wat::kernel::RecvOutcome<"
                  "wat::kernel::ThreadSelfPeer<"
```

All seven are macros building a type **NAME** at expand time and handing it to `keyword-node` /
`keyword/from-string`. The name then serves as a type IDENTITY.

## Why this is NOT a leftover, and why ②-iii shipped without it

This is exactly `defservice`'s `{b}::Op{p}` — NOTE-2iii's blocker 3d, which I told the builder was
"the last real obstacle" before re-running the codemod. **The re-run refuted it.** The migrated
corpus floors green with every one of these minting angle-form names, because the names are minted
and consumed *internally* and round-trip consistently. Nothing outside reads them.

So the question these raise is not "is the migration finished" — it is — but **"is a macro-built type
identity a NAME or a FORM?"** That is a design question with a real blast radius: it touches how
`defservice` and `bracket` mint every generated type, and it is ③'s territory (making the angle form
ILLEGAL), not ②'s.

⚠ **Do not "just fix" these with a sed.** Changing `":wat::kernel::Peer<" + args + ">"` into a form
means the emitted thing stops being a keyword and becomes a list, and every downstream consumer of
that identity — `keyword/from-string`, the annotation sites identity 2c already converted, the
`:messages` membership checks — has to be walked. That is a stone with a DESIGN.

## ✅ The eighth was a DIFFERENT thing and is FIXED

`wat/fix.wat:502` was not a macro minting a name for itself. It was a **codemod's replacement text** —
the string `fix-macro-param-types` writes *into other files*:

```clojure
new-text (:wat::core::if after-amp?
            ":wat::core::Vector<wat::WatAST>"      ;; wrote the RETIRED spelling
            ":wat::WatAST")
```

**A codemod emitting the retired form re-introduces angle types every time it runs** — a live
regression vector, and the file was migrated everywhere except its own output (two lines below it
correctly uses `(:wat::core::Tuple :- [...])`).

Fixed to `"(:wat::core::Vector :- [:wat::WatAST])"`. The destination was already proven legal —
`wat/Record.wat:109,225` carry exactly that shape post-migration. Its golden
(`tests/resolve/probe_arc251_fix_macro_param_types.rs`) pinned the old output and was updated
deliberately; the codemod's own test passes.

⚠ This was a legitimate hand-edit of a `.wat` and does NOT violate R21: that rule governs
*structural rewrites across many files*, and the codemod **by design does not rewrite string
literals**, so it structurally cannot fix its own output.

★ **The distinction worth keeping: one was OUTPUT, seven are INTERNALS.** I reported all eight as one
undifferentiated "remaining" until the builder asked why we weren't just fixing them.

## Kin

- `109/NOTE-2iii-is-blocked-*.md` — blocker 3d, refuted by the re-run
- `2a0d7fa2e` — ②-iii shipped with all seven live, floor green
- `294/SEAM.md` — `fix.wat:502` was listed there as unruled; it is now closed
