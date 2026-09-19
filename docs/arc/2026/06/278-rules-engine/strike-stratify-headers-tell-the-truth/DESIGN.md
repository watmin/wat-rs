# DESIGN — the stratify headers say "lockstep"; one of them is the file's thesis

## Why

L2-3 (`aa10ef8bd`) drove it and the `accum-over-derived` axis (`05d33d022`) confirmed the facts
agree at depth with Clara live. What is left is prose that is false, in the file whose whole
subject it is.

> ⚠ **EVERY LINE NUMBER IN THIS FILE IS PRE-EDIT.** The fix inserted ~17 comment lines into
> `stratify.rs`, so the citations below no longer resolve — this file is the record of what was
> true when the strike was drawn, not a map of the tree today. **Post-edit anchors:** the divergent
> `+1` term is `stratify.rs:247-253`; `consume_types`' `:exists` / accumulate-`:from` arms are
> `:195-196`; `produced_type` is `:65`. Landed prose carries the post-edit addresses; the
> executor recomputed them rather than transcribing the BRIEF's verbatim ones, which is the
> deviation its SCORE names first.

## The audit — ALL EIGHT claims, not the two the builder asked for

`grep -i 'mirror\|lockstep'` over both files returns **eight** claims. Fixing two while six
unchecked siblings sit beside them is patching the stem, so each was re-read against its oracle
twin on 2026-09-07:

| site | claim | verdict |
|---|---|---|
| `stratify.rs:12-16` | *"Faithful Rust port … moves in lockstep"*, list **includes `stratify-sweep`** | ⛔ **FALSE** — driven |
| `stratify.rs:202-205` | *"Mirrors `stratify-sweep`"* **and its formula** | ⛔ **FALSE** — driven |
| `stratify.rs:19` | `fact_type_head` mirrors the inline `ast-name` + colon-strip | ✅ holds |
| `stratify.rs:250` | `native_stratify_fix` mirrors `stratify-fix` | ✅ holds — sweep, `!changed` return, cycle check, recurse |
| `stratify.rs:277` | `native_stratify` mirrors `stratify` | ✅ holds — same `len+1` bound, same empty init |
| `stratify.rs:285` | `native_rule_stratum` mirrors `rule-stratum` | ✅ holds — both `max(max s[p], max s[n]+1)` |
| `stratify.wat:157-158` | *":exists inner and accumulate :from ARE [positive reads] — lockstep with native `rule_consumes`"* | ✅ **TRUE** — see below |
| `stratify.rs:35` | `rule_produces` mirrors `rule-produces` | ⚠ **UNPROVEN — rowed, not fixed** |

### ⛔ The one I got wrong, and it is the interesting one

I told the builder `stratify.wat:157-158` was false. **It is true.** Native's `rule_consumes`
*does* include `:exists` inner and accumulate `:from` — `consume_types` (`stratify.rs:178-179`)
recurses into both. It is in lockstep with the oracle exactly as claimed.

What makes it dangerous is that it is **accurate and incomplete in the same breath**: native routes
those same types into a SECOND list, `rule_bag_consumes`, and the sweep gives that list a `+1`.
Nothing near this sentence says so, so a reader who checks the claim finds it true and stops.
`[[an-accurate-comment-can-be-a-defects-alibi]]` — true description, wrong register. It gets a
**pointer, not a correction.**

### ⚠ The fourth site, rowed rather than fixed

`produced_type` (`stratify.rs:48-64`) resolves a head through the `SymbolTable`: if the head names
a function whose return type is a non-`wat::core::` `Path`, it returns **that return type** instead
of the head. The oracle's `rule-produces` takes the first child and strips the colon — no symbol
table, no resolution. A rule whose RHS is a userfn head (`probe_arc278_then_user_forms_userfn`
proves that form is supported) would therefore produce a different type name on each side, and
`produced` feeds stratification directly.

**That is a READING, not a drive, and it is not fixed here.** It is the same shape as L2-3 before
it was driven, and it earns its own measurement strike — write neither "mirrors" nor "diverges"
into `:35` until someone runs both.

## The one contract decision

**The corrected headers cite the gate, not the conclusion.** Each replacement names
`kernel/tests/stratify_numbers.rs` and the grid axis, so the next reader can re-derive the claim
instead of trusting this sentence — which is the failure being fixed. A header that says "these
diverge" with no instrument named is the same class of assertion as the one it replaces.

## Out of scope = REJECTED

- **Any `+1` moved on either side.** The facts agree; there is nothing to cure.
- `:35` / `produced_type` — rowed above, drives first.
- Every ✅ row — verified this session, left exactly as they are.
