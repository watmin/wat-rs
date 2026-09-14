# NOTE — `concat-abuse`'s suggested remedy is refused inside a macro body

> **Written 2026-08-26 from branch `grok-rete` (arc 278), for arc 277.** Found by trying to ACT on
> the rule's advice. Nothing changed in `wat/lint.wat` — 278 has no mandate over the linter.
>
> **The rule was RIGHT to flag the site.** Only its suggested fix is wrong, and only in one
> context. The flagged code is now corrected in `wat/gen.wat`, and `lint-stdlib` reports 0 for
> that file.

## What happens

`(:wat::lint::lint-stdlib)` reports, against `wat/gen.wat`'s `record` macro:

```
concat-abuse: string::concat interleaves 1 literal(s) with 1 value(s)
              — use (:wat::core::format "…{name}…" :name v …) instead
```

Taking that advice literally does not compile:

```
malformed defmacro: keyword head `:wat::core::format` refused at macro expand time
— not on the pure-combinator allow-list (default-deny F5 gate, arc 249 stone 249.2b-i)
```

`:wat::core::format` is a **macro** (`wat/core.wat:1639`). Inside a `defmacro` body the F5 gate
admits only heads that are expand-time evaluable, and a macro is not one.

## The fix that DOES work, and it is already blessed for exactly this

`:wat::core::string::interpolate` is on the allow-list, and its comment there
(`src/macros/eval.rs`, arc 284) names this case precisely:

> *pure-total interpolation intrinsic: same `{name}` + `:name val` grammar as the format macro,
> but interpolates at call time → **expand-time-legal in macro bodies***

`wat/core.wat:704` already uses it in the same position for the same reason:

```wat
(:wat::core::string::interpolate ":{name-raw}" :name-raw name-raw)
```

So the grammar the rule wants IS reachable from a macro body — under a different spelling. The
`wat/gen.wat` site now reads:

```wat
ctor (:wat::core::keyword-node
       (:wat::core::string::interpolate "{n}'" :n (:wat::core::ast-name T)))
```

and `lint-stdlib` drops from 87 findings to 86, with **gen.wat at 0**.

## The finding, stated narrowly

**The rule's message hardcodes one of the two spellings**, and it is the one refused at the only
kind of site a macro body can offer. A caller who follows it hits the F5 gate, and nothing in the
message hints that a legal twin exists — the fix took a compile failure and a read of
`src/macros/eval.rs` to find.

Suggested shapes, for 277 to choose:

1. **Name both in the message** — cheapest, no walker change: *"use `format` (program body) or
   `string::interpolate` (macro body)"*. Correct everywhere, needs nothing from the walker.
2. **Context-sensitive message** — emit `string::interpolate` when the enclosing top-level form is
   a `defmacro`, `format` otherwise. Better UX; needs the walker to carry the enclosing form kind.
3. **Leave it** — and accept that every macro-body hit costs its caller the same detour.

(1) buys most of the value for none of the machinery.

## ⚠ A correction to the record — `circumspicere` reported gen.wat CLEAN and it was not

`circumspicere` (2026-08-26, `docs/arc/2026/06/278-rules-engine/GEN-VIGILIA-2026-08-25.md`
finding 5) reported *"filtered to `file == "wat/gen.wat"` → 0. gen.wat is clean under both shipped
rules. The finding is that nothing would have said so."*

**That 0 was wrong.** There was 1 finding, and the flagged line is byte-identical at `a20f063a6`,
the ward's own HEAD (line 341 there, 643 now). A deterministic form-level rule fired then too. The
ward's tree-wide total also disagrees with this session's (91 vs 87 before the fix); not chased.

**This sharpens the ward's own point rather than weakening it.** Its argument was "nothing would
have said so". The truth is stronger: gen.wat was not clean, nothing said so, and the one cast that
looked directly at it got the number wrong. **A gate would have been right where a reading was
not** — which is the case for the ratchet finding 5 proposes, made against the ward itself.

## On the ratchet finding 5 proposes — now buildable, and NOT built here

With gen.wat at a true 0, a by-file ratchet (per `[[feedback_a_gate_freezes_names_never_a_count]]`)
would hold. It is not added from 278: arc 277 is **OPEN** (no INSCRIPTION, no SCORE), `lint-stdlib`
is its surface, and a gate belongs to the arc that owns the rules it freezes — particularly while
the rule set is still growing. Flagged, not taken.
