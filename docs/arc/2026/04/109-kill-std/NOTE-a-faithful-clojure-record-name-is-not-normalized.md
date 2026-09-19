# Faithful-Clojure translates in every position but one — the record NAME — so no `defrecord` the converter emits can be read back

**Filed 2026-09-10, arc 278 (`strike-no-rule-that-cannot-compile`), grounded here in arc 109
because this is the faithful/colon-mode surface. Measured, not fixed.**

> # ⛔ AMENDED THE SAME DAY, AND THE CORRECTION MATTERS MORE THAN THE FILING
>
> **THE FRAMING BELOW IS STRUCK. This is not a gap, a defect, or a "pre-existing hole" — it is
> the CURRENT BOUNDARY OF A DRAWN, TRACKED CAMPAIGN, and it is exactly where that campaign says
> it should be.** The builder, on reading the original: *"oh - your whole complaint is just that
> we aren't symbol heads yet?.... we have been working on this path for months..."* Correct, and
> I had not read the record before filing.
>
> **The owning work is `docs/arc/2026/06/251-types-as-forms/DESIGN-STONE-251.8-symbol-proper.md`**,
> drawn 2026-08-13 on the builder's ruling *"i think we need to implement symbol proper... that's
> been missing since the beginning as we've been using 'colon-quoted symbols'."* Its status line:
> **8a strike-ready; 8b–d are the campaign shape, not yet briefed.** And its fourth stone is this
> NOTE's entire subject:
>
> > **`251.8d — THE READER/PRINTER FLIP.`** *"`::` retires as a reference spelling; a colon means
> > keyword, full stop. The 1392-file corpus flip lands here, as a spelling change over a
> > substrate that already type-checks both."*
>
> So `wat.source/File` failing in the record-NAME position is **not rot**: the two
> `to-faithful-clojure-*` codemods are the migration tools built FOR 8d, emitting its target
> syntax against a substrate that has reached 8a. Their output not reading back is the expected
> state of a campaign three stones from that flip. 251.8 even cuts the adjacent case
> affirmatively — *"the corpus flip is NOT in this stone… it must not begin while a dotted call
> head is unchecked."*
>
> **⭐ THE LESSON, AND IT IS THE ONE WORTH KEEPING.** This arc has spent a week finding claims
> that were true when written and rotted silently — *"no ward is aimed at 'this was right when it
> was written.'"* Here I ran that exact error **in reverse**: I aimed a defect-shaped filing at
> work that is deliberate, tracked, in-flight, and correct. **A measurement of an unfinished
> campaign is a PROGRESS reading, not a finding — and which one it is cannot be determined from
> the code, only from the record.** I measured five variants carefully and read zero design docs.
>
> **What survives, and why this NOTE is kept rather than deleted:** the five-variant isolation
> below is a good measurement, and as a *progress reading* it is genuinely useful — it says
> precisely how far the normalizer has come toward 8d (head: done; type position: done;
> definition-site name: not yet). Re-run it to see the campaign move. Read it as a milestone
> marker, never as a bug report.
>
> **⚠ One thing I still do not know, stated as a question rather than a finding:** both codemods'
> headers say *"Gate = round-trip parse of the output"* in the present tense, for a gate that
> cannot hold until 8d lands. Whether that wants a forward-looking word added is a judgement for
> whoever owns 251.8 — it is not a defect I am asserting.

## What was actually measured

The two `to-faithful-clojure-*` codemods ran for the first time ever at `2a9de244a` and both emit,
byte-identically, from `wat/source.wat`:

```clojure
(wat.core/defrecord wat.source/File
  [path   :- wat.type/String
   source :- wat.type/String])
```

Both codemods' own headers claim *"Gate = round-trip parse of the output."* **That output does not
parse.** Driven at `2a9de244a` on `./target/release/wat`:

| # | form | verdict |
|---|---|---|
| **A** | `(wat.core/defrecord wat.source/File [path :- wat.type/String])` | ⛔ `#wat.macro/ProgramBodyEvalFailed` |
| **B** | `(wat.core/defrecord :wat::source::File [path :- wat.type/String])` | `DuplicateType :wat::source::File` — i.e. it **got through**, and only collided with the stdlib's own declaration |
| **C** | `(:wat::core::defrecord wat.source/File [path <- :wat::core::String])` | ⛔ identical failure to A |
| **B′** | `(wat.core/defrecord :zz::Fresh [path :- wat.type/String  n :- wat.type/i64])` | ✅ prints `"ok"` |
| **A′** | `(wat.core/defrecord zz.rec/Fresh [path :- wat.type/String  n :- wat.type/i64])` | ⛔ identical failure to A |

⭐ **B′ is the load-bearing row.** Faithful head, faithful `:- ` arrow, faithful `wat.type/…` in
*type* position — all of it translates and the program runs. **Swap only the record NAME to
faithful form (A′) and it dies.** A and C failing identically shows the head spelling is
irrelevant: this is the name argument alone.

## The site

`wat/Record.wat:171` — `fqdn-str (:wat::core::keyword/to-string fqdn)` — raising
`:wat::core::keyword/to-string: expected keyword, got wat::WatAST`. The macro receives the record
name as an un-normalized AST symbol and asks it for a keyword's string.

## ⛔ The mechanism is NOT established here, and the two candidates have different cures

The normalize pass that rewrites `a.b/c` → `:a::b::c` is arc 251 stone 251.1b
(`src/resolve/mod.rs:314`), and its own test is named
**`namespaced_symbol_head_normalizes_to_keyword`** (`src/resolve/mod.rs:317`) — *head*. Neighbouring
tests establish that it deliberately skips quoted forms, `match` patterns, and quasiquote templates.

That leaves two readings, and this NOTE does **not** pick between them:

1. **Ordering** — macro expansion runs before normalize, so `defrecord` always sees the raw symbol.
   Cure: normalize before macro expansion, or normalize macro arguments.
2. **Position** — normalize runs first but does not rewrite a *definition-site* name (as opposed to
   a reference). Cure: teach it the definition-site positions.

⚠ Reading 2 is complicated by B′: `wat.type/String` in *type* position DOES end up correct. That
may be normalize doing its job, or `defrecord`'s own expansion handling the `:- ` field syntax
itself. **Distinguishing these is the first work, and it is not done.** Anyone who fixes this
without first settling which reading is true is guessing at a cure.

## Why it matters

- Both `to-faithful-clojure-net.wat` and `to-faithful-clojure-rete.wat` are **recorded migrations**
  — `CLAUDE.md` names `wat-scripts/fixes/*.wat` as the shape to copy. Their stated round-trip gate
  does not hold, and until `2a9de244a` neither had ever run, so nobody could discover that.
- The failure is a macro-internal type error (`expected keyword, got wat::WatAST`), not a located
  syntax diagnostic. A user writing faithful-Clojure by hand gets a message about
  `keyword/to-string` and no indication that the *record name* is the thing to change.

## Scope

Orthogonal to whether rete rules compile, which was `strike-no-rule-that-cannot-compile`'s subject —
that strike's gate is green and this is out of its enumerated blast radius. Filed, not fixed.
