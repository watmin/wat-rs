# REFUTE — `defenum` has the wrong shape, and it must INSERT the empty vector

**Rows 2, 3-defstruct, 4 and 14 are ACCEPTED and not in question.** `defrecord`, `defstruct` and —
the long-owed one — `defn`'s arg-spec `<-` alignment are all correct:

```
[self    <- :wat::core::i64          ← unbuilt since the FIRST style table. Now aligned.
 work-fn <- :wat::core::String
 c       <- :wat::core::f64]
```

**`defenum` is the one that is wrong**, and it was unruled when the stone was drawn — the BRIEF said
*"if the shape resists, STOP and report"*, and the strike chose a shape rather than reporting. **That
is the only fault, and it is a small one.** Both corrections below are now RULED.

## ⛔ CORRECTION 1 — the variant TAG and its field vector are ONE UNIT

```
SHIPPED                                    RULED
(:wat::core::defenum :fix::Ev … :Shutdown  (wat.core/defenum some/Name wat.enum/Pure
  :Admin                                     Member1 [key1-long :- wat.type/i64
  [msg <- :wat::core::String]                          key2      :- wat.type/f64]
  :Pair                                      Member2 [key3 :- wat.type/String]
  [idx  <- :wat::core::i64                   Member3 [key4 :- wat.type/Keyword])
   name <- :wat::core::String]
  :Nil [])
```

A tag and its vector split across two lines read as two unrelated things. **They are a pair** — the
same idea as a `let` binder, a kwarg, and a map entry, all of which are already ruled to stay
together.

★ **The corpus already writes it the ruled way** — `wat/sqlite.wat:52-55`, hand-maintained:

```
  :Transient  [fault <- :wat::sqlite::Fault]   ;; SQLITE_BUSY/LOCKED — retry
  :Constraint [fault <- :wat::sqlite::Fault]
  :Fatal      [fault <- :wat::sqlite::Fault])
```

**The tags are PADDED so the vectors line up.** That is `AlignStride` at stride 2 — the capability
this very stone built.

### The shape, precisely

```
head line       defenum + name + purity, and NOTHING else
                (`:Shutdown` must NOT ride it — the shipped version put the first variant
                 somewhere different from every other variant)
each variant    the TAG starts a line; its field vector is GLUED to the tag
tags            PADDED so the vectors begin in one column      — AlignStride, stride 2
inside a vector one field per line, aligned under the first, `<-` in one column
                — the treatment `defrecord` just received      — AlignStride, stride 3
```

⚠ **The multi-field case is the part `sqlite.wat` cannot teach** — every variant there has exactly
one field. The builder's `Member1` shows it: the continuation aligns under the FIRST FIELD, inside
the `[`, which is the existing `:align` kind. **No new capability.**

## ⛔ CORRECTION 2 — a bare variant gets `[]` INSERTED

> **Builder:** *"our formatter should insert empty vec... not omission."*

Both spellings type-check today (measured, exit 0 each); there are **81 bare variants in `wat/`
alone**, and `wat/spawn.wat:195-203` mixes both inside one enum.

```
(:wat::core::defenum :fix::Ev :wat::enum::Pure
  :Shutdown            ->     :Shutdown   []
  :Admin [msg <- …])          :Admin      [msg <- …])
```

**Then every variant is a tag-plus-vector pair and the irregularity is gone** — which is exactly why
`defenum` needed special-casing in the first place.

### ⚠⚠ THIS IS A CATEGORY CHANGE FOR wat-fmt — do NOT let it pass unremarked

Every rule to date decides **whitespace and line breaks only**.
`[[NOTE-the-double-underscore-contagion]]` drew the line explicitly: *"`wat fmt` formats LAYOUT. It
never touches a NAME."* **Inserting `[]` adds a TOKEN to the source.**

It is authorised, and the argument is narrow and must stay narrow:

```
a RENAME        can capture, shadow, change meaning      -> NOT the formatter's
inserting `[]`  both spellings legal and EQUIVALENT      -> value-preserving, like whitespace
```

⛔ **This licenses the `[]` case ONLY.** A second insertion needs its own argument.
`[[NOTE-a-bare-enum-variant-is-an-optional-slot-and-must-die]]` carries the reasoning and the order:
the formatter IS the migration, and arc 109's checker change costs nothing once the corpus is
formatted.

## THE ADDED ACCEPTANCE ROWS

```
E1  ★ a variant TAG and its vector are on ONE line
E2  ★ nothing but name + purity rides the head line — the FIRST variant is not special
E3  ★★ tags are PADDED so the vectors begin in one column          (sqlite.wat's shape)
E4  ★★ a MULTI-FIELD variant: fields one per line, aligned under the first, `<-` in one column
E5  ★★★ a BARE variant gets `[]` INSERTED
E6  ★★ an ALREADY-empty `[]` is unchanged — insertion is not duplication
E7  idempotent — a second pass must not insert a second `[]`
E8  ★★ wat/spawn.wat:195-203 (bare AND field-carrying mixed) renders correctly and TYPE-CHECKS
E9  the previously-accepted rows do not regress: defrecord, defstruct, defn's `<-`, row 14
```

⛔ **E7 is where an insertion rule characteristically fails.** The output of pass 1 contains the
`[]` the rule inserts; pass 2 must recognise it as already-present. **A rule that inserts
unconditionally produces `:Shutdown [] []`.**

⚠ **E8 is a real file, not a fixture.** `spawn.wat` mixes both spellings and its enum is
`:- [I O A]`-parametric — if the rule mis-handles it, the stdlib stops type-checking, which the
floor will catch but the fixtures will not.

## NOT DISPUTED

`AlignStride {form, stride}` as a sibling of `AlignPairs` is the right shape and stride 3 is
correct. `defn-args.wat` asserting stride 3 is what closed row 4. `kwargs` no longer treating a
`defrecord` name as a pair-run key is the right cause-fix for the name falling off the head line.
Every prior rule still wins, and the 614 doc examples remain **0 over 120, worst 104**.
