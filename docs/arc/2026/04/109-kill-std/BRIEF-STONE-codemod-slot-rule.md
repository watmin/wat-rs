# BRIEF — arc 109: the codemod's SLOT RULE — a declaration name is a binder, not a reference

The corpus codemod rewrites every angle-bracketed keyword the same way. That is right for a type
**reference** and wrong for a declaration **name**, and it corrupts 84 declaration sites.

Measured on a `/tmp` copy of `wat/spawn.wat`, at HEAD:

```clojure
;; references — ALREADY CORRECT (they inherited `:-` from ②-i-b's renderer, for free)
[p   <- (:wat::kernel::Peer :- [I O])
 acc <- (:wat::core::Vector :- [O])]

;; declaration name — CORRUPTED: a LIST in the name slot
(:wat::core::defn (:wat::kernel::recv-all-loop :- [I O])
;; must be SIBLINGS:
(:wat::core::defn :wat::kernel::recv-all-loop :- [I O]
```

α accepts `name :- [T…]` in every declarator, so there is a correct destination waiting.

## ★ The flaw is written down as a design decision — read it first

`wat-scripts/fixes/parametrics-take-a-type-vector.wat:136`:

> *"seq-edits — left-to-right walk over a child vector … **no position-tracking needed:
> `type-shaped-keyword?` fires the same regardless of where the keyword sits, so there is no state
> to thread between siblings**."*

That is precisely wrong for the one slot where position decides everything. **Correct the comment
as part of the change** — leaving it would re-teach the next reader the mistake.

## Rooms

```
wat-scripts/fixes/parametrics-take-a-type-vector.wat
  :108  leaf-edits   — emits the replacement for one keyword
  :127  node-edits   — structural → seq-edits, leaf → leaf-edits
  :138  seq-edits    — the sibling walk. THREADS NOTHING TODAY. This is the room.
  :136  the comment above, which must change with it
wat/fix.wat
  :123  fix-seq      — the PROOF that context-threading works here: it already threads
                       `prev-arrow?` through exactly this shape of recursive sibling walk.
                       Copy that shape; do not invent one.
```

## The work

**Thread one flag.** `seq-edits` gains a parameter — "the previous sibling was a declarator head" —
computed as: this is index 0 of a structural node AND that item is a keyword whose name is one of
the declarator heads. `leaf-edits` takes the flag and branches.

**In the name slot, emit the binder form.** For a type-shaped keyword at a declaration name:

```
reference:  (:wat::kernel::Peer :- [I O])       ← what the renderer already produces
name:        :wat::kernel::Peer :- [I O]        ← the SAME text, minus the outer parens
```

★ **A binder is the reference form without the application.** So reuse
`keyword/to-type-form-colon` + `ast->source` exactly as the reference path does, then strip the
leading `(` and trailing `)`. Do **not** build the name text by a second route — a second renderer
is a second thing to drift.

**The declarator heads, measured** — these are the only heads that carry a parametric name in the
corpus, and the count each contributes:

```
defn 52 · defenum 11 · defsurface 11 · defrecord 7 · defstruct 2 · defservice 1
```

Take the list explicitly, including the `:wat::holon::` variants of `defrecord`. **If the walk sees
a parametric name under a head not in your list, STOP and report it** — a seventh head means my
census was wrong, and silently rewriting it would be the same class of defect this stone fixes.

## STOP triggers

1. **STOP-1** — if a type REFERENCE's output changes at all, STOP. References are already correct;
   this stone only adds a second case. Re-run the dry-run and diff.
2. **STOP-2** — if you need a second renderer for the name text, STOP. Strip the parens off the one
   the reference path uses.
3. **STOP-3** — a parametric name under an unlisted head. Report, do not rewrite.
4. **STOP-4** — **do not apply the codemod to the corpus.** Dry-run on `/tmp` copies only. ②-iii is
   a separate stone and the orchestrator runs it.

## How this lands

You are a rider. **Text edits only**, in `wat-scripts/fixes/parametrics-take-a-type-vector.wat`.
Do not build, commit, stash, or revert.

⚠ You **can** test this one, unlike the last three stones: the codemod is a `.wat` SCRIPT, not
stdlib — it is read from disk at run time, not baked in by `include_str!`. So
`./target/release/wat ./wat-scripts/fixes/parametrics-take-a-type-vector.wat` runs YOUR edit against
the CURRENT binary. Dry-run on a `/tmp` copy of `wat/spawn.wat` (it has four `<…>` in four lines —
one binder, three references) and diff. Say in your report what you ran and what it produced.

Report: the diff; the declarator-head list you used; the dry-run output for `spawn.wat:614` and
`:615–616`; and anything on disk contradicting this brief.
