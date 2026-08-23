# NOTE (arc 109) — ~18 macros still MINT the angle form (filed as SEVEN — that count was wrong too)

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

---

## ⛔ AMENDED, one hour later — SEVEN was wrong too. The count is ~18, and hand-counting is retired.

The builder asked why we were not simply converting the seven. Tracing them to their consumers
surfaced a fifth under-count: my `"…Name<"` probe missed every `string::interpolate` site, because
those spell the name **without a leading colon** and fill the angles with `{}`:

```
"wat::kernel::Peer<{o},{r}>"        "wat::core::Vector<wat::kernel::Peer<{r},{o}>>"
"wat::capability::Dialable<{o},{r}>"  "wat::service::Alarm<{o}>"
"wat::kernel::Address<{o},{r},{t}"    "wat::spawn::Locus/launch<"
```

An over-counting probe — any string literal holding both `::` and `<` — returns **20**, of which two
are prose inside an error message. So ~18 real sites across `wat/service.wat` and `wat/bracket.wat`.

**Five hand-counts, five wrong, every one under.** The count is not the point any more; the method is.

★ **And the reason no grep can finish this: most of these names exist in no file.** They are assembled
at expand time and handed to `keyword-node`. Only a wall at the type parser sees a minted name the
same way it sees a written one.

Builder's ruling: *"make parametrics via angle brackets illegal and just make every heretic scream —
set them ablaze … that's your census."* Briefed as
`BRIEF-STONE-set-the-angle-form-ablaze.md`. The door is `src/types.rs:4608`, the single `match` arm
where a keyword becomes a parametric. The stone's deliverable is the CENSUS, not a green floor.

**What the tracing did establish, and it survives:** the roles are real and they differ per site.
`bracket.wat:448-449` feeds ANNOTATION slots (`[self <- ~runner-self-kw ctx <- ~ctx-ty-kw]`) and can
take the form today. `"wat::spawn::Locus/launch<"` is a CALLABLE name with a type suffix — not a type
reference at all, and a form is not a name. That is why this was never one sed.
