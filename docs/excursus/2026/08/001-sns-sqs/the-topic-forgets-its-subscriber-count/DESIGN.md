# DESIGN — the topic forgets its subscriber count

## Why

`e0c552bf0` moved the fanout out of `publish` and into the worker. `nsubs` on
`:demo::topic::Record` is what is left behind: **the last fossil of the pair-shaped design.**

It is not generic dead code. It is a *second source of truth for a fact the topic does not own.*
The worker holds `sub-addrs` and derives `nsubs (:wat::core::count subs)` at
`sns-fanout.wat:425` — that is correct and stays. The topic holds a **number** that duplicates it,
written at every construction site and read by nothing.

⛔ **The two can disagree and nothing notices.** Before `e0c552bf0`, `publish` expanded by the
topic's `nsubs`, so a disagreement produced wrong fanout — loud. Now the field is unread, so a
disagreement is **silent**, and any future publish-side decision that reaches for it gets a stale
answer. That is the trap, and it is why this ships now rather than later: a dead `:durable` field is
precisely what becomes a graveyard reading as live code.

Verified this session: `:demo::Topic::StatsResponse::Ok` carries
`[depth ticks inbox-lost inbox-closed inbox-timedout]` — **no `nsubs` slot**, so no declared
surface changes. The only non-construction read is the field copy at `:220`.

## What it delivers

`nsubs` gone from `:demo::topic::Record`. The topic routes to an inbox and knows nothing about
subscribers. The divergence above becomes **unrepresentable**, not merely unobserved.

## The one contract decision

**This is a corpus migration and it goes through `wat/fix.wat`** — a recorded codemod in
`wat-scripts/fixes/`, never hand-edits, never python/sed. That is doctrine (CLAUDE.md), and the
site count is what makes it doctrine rather than preference.

## Three forms, one rule set — and the count is the finder's to report, not mine

| form | where | how it is disambiguated |
|---|---|---|
| kwarg pair `:nsubs <value>` | every `:demo::topic::Record` construction | keyword node named `:nsubs` whose parent's head is `:demo::topic::Record` |
| binder `nsubs <- :wat::core::i64` | `sns-fanout.wat:71` | **symbol** in the vector following `:durable` under `defservice :demo::topic` |
| accessor `:demo::topic::Record/nsubs` | `:220` | **subsumed** — it is the *value* of the `:220` kwarg pair, so deleting the pair takes it |

⛔ **I am not handing over a site count, and that is deliberate.** My own single-line grep said 19
constructions; it missed the multi-line one at `:219–221` entirely. `wat --grep` is the census —
run it first, diff it, then apply. Any number I wrote here would be a proxy for it.

## ⛔ The trap — FORM, not token

`nsubs` occurs **23 times in `sns-fanout.wat` alone**, and most of them must survive:

- `:425` `[nsubs (:wat::core::count subs)]` — the **worker's `let` binding**. This is the correct
  source of truth. A rule keyed on the symbol name alone **will hit it.**
- `:506 :527 :528 :533 :534` — references to that binding.
- ~7 comment lines. The tool walks forms, so comments are untouched by construction; prose is a
  separate manual pass.

The binder rule must therefore key on **parentage**: the vector's parent head is
`:wat::service::defservice`, not `:wat::core::let`. That distinction is the whole rule.

## The exemplars — cited, not described

Both halves already exist in applied, committed migrations:

- **`wat-scripts/fixes/alarm-after-to-delay.wat:78–97`** — matches a **symbol inside a binder
  vector**, disambiguated by parentage (`?hi 0` head, `?ti 1` type). It is the *only* recorded fix
  that matches kind `"symbol"` (81 match `"keyword"`), and it carries three forms in one rule set —
  the same three shapes as this stone. Its header states the identical warning: *"FORM, not token."*
- **`wat-scripts/fixes/declare-queue-drop-knobs.wat:39–50`** — the exact **inverse**: kwarg pairs
  added to a `:durable` record's constructions, anchored on the head keyword, idempotent by
  presence check.
- **`wat/fix.wat:243` `fix-text-deletion-edit`** + `:1002/:1006` `node-start-offset` /
  `node-end-offset` + `:367` `fix-text-apply` — the deletion primitives.
  **`:505` `strip-arrow-ascription`** is the worked composition: it deletes `->` **plus** its type,
  position-agnostically. A three-token binder deletion is the same move with one more token.

## Out of scope = rejected

- **No promotion to `wat/`.** Settled by reading, not by asking: `fix-text-deletion-edit` already
  exists and `strip-arrow-ascription` already composes it into a multi-token deletion. This stone is
  a local codemod in `wat-scripts/fixes/`. No stdlib change, so no builder's ruling is owed.
- **The worker's `nsubs` stays.** It is the correct derivation.
- **Comment prose** mentioning `nsubs` — a separate manual pass, because the tool walks forms.
- Not touched: the residual drain slope, `scan-index`'s +45 % level shift, the counter-carrier tax.

## No probe, and here is why in as many words

The load-bearing assumption is *"a codemod can delete a symbol + `<-` + type from a binder vector,
without touching a same-named symbol in a `let` vector."* I did **not** write a probe for it,
because the precedent is stronger evidence than a probe I would write: `alarm-after-to-delay.wat`
does symbol-in-binder-vector matching with parentage disambiguation, is committed, and is green in
the corpus. **The corpus gate is the standing probe** — `every_wat_scripts_file_loads` parses and
type-checks all seven affected files, so an over-match on the worker's binding cannot ship quietly.
If the finder cannot separate the two by parentage, that is **STOP-1**.
