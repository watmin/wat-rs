# DESIGN — STONE: the checker knows `Option::Some`; the runtime does not

> **Builder, 2026-09-04:** *"the matcher gap.... brief and release"* — chosen over opening the enum
> redesign, because the redesign's own place in the roadmap is *after* crates, and this gap is
> what actually blocks the campaign.
>
> Governed by `[[RULING-the-registry-is-the-sole-authority]]` — but the operative doctrine here is
> the one 294 R9 named: **an anneal that does not finish leaves a bridge nobody demolishes.**

## The three spellings

```
:None                       legacy bare
:wat::core::None            what arc 109 slice 1h called "the FQDN form"; what the RUNTIME knows
:wat::core::Option::None    the `Enum::Variant` form EVERY USER ENUM uses, produced by
                            `register_enum_methods`, held by `sym.unit_variants`, and expected by
                            `is_resolvable_call_head`'s unit-variant door (Stone 241.9)
```

`Value::Option` and `Value::Result` are **native `Value` variants** (`src/value/value.rs:138,145`),
not generic tagged enum values — so they never went through the generic enum path, and the third
spelling was never wired for them.

## THE GAP — one-sided, and proven with controls

Four probes, all run this session:

```
1 CONTROL  user defenum, qualified spelling `:usr::Colour::Red`      → runs, prints "red"
             ⭐ the runtime knows the qualified form PERFECTLY. This is not a general gap.
2 CONTROL  builtin, BARE spelling, both arms                          → runs, prints "1"
3 CONTROL  builtin, BARE spelling, MISSING the None arm               → --check RED
             ⭐ exhaustiveness checking WORKS
4 CONTROL  builtin, qualified but NONSENSE variant names              → --check RED
             ⭐ the checker really RECOGNISES the qualified variants; it is not a wildcard hole
5 TEST     builtin, QUALIFIED spelling, both arms                     → --check CLEAN, run RAISES
             `PatternMatchFailed: no arm matched scrutinee of type wat::core::Option;
              exhaustiveness should be caught at type-check time`
```

**The checker was taught the qualified form. The runtime's hardcoded sites were not.** Probes 1 and
4 are what make this a located defect rather than a guess — without them the honest reading would
have been "the runtime doesn't do qualified variants," which probe 1 refutes outright.

★ And the raise's own text indicts the pair: *"exhaustiveness should be caught at type-check time."*
It was — the checker counted both arms as covering. The runtime then matched neither.

## The sites

```
src/runtime.rs:1715   if k == ":None" || k == ":wat::core::None"
src/runtime.rs:8324   WatAST::Keyword(k,_) if k == ":None" || k == ":wat::core::None"   ← the matcher
src/runtime.rs:8412   WatAST::Keyword(k,_) if k == ":wat::core::Some"
src/runtime.rs:8441   WatAST::Keyword(k,_) if k == ":wat::core::Ok"
src/runtime.rs:8467   WatAST::Keyword(k,_) if k == ":wat::core::Err"
src/runtime.rs:13011  matches!(s, ":wat::core::Some" | ":wat::core::Ok" | ":wat::core::Err")
```

Six. Twenty-one further sites in `src/check.rs` carry the same spellings but are **out of scope** —
the checker already accepts the qualified form (probe 4), so touching them would be changing code
that is not wrong.

## The house precedent — ADDITIVE recognition

`src/types.rs:1056` on `nil`, verbatim: *"the `:wat::core::nil` keyword is also accepted at value
position (**additive recognition**; both spellings evaluate to the nil singleton)."* Same move: the
qualified spelling is **added** beside the existing ones. Nothing is removed by this stone.

## THE FOUR QUESTIONS — flat YES/NO

| | Obvious? | Simple? | Honest? | Good UX? |
|---|:---:|:---:|:---:|:---:|
| **teach the runtime the qualified spelling** | YES | YES | YES | YES |

- **Obvious? YES** — every user enum already matches this way; the builtins are the exception.
- **Simple? YES** — six guards gain an alternative. No new mechanism, and a named house precedent.
- **Honest? YES** — a spelling the checker admits and the runtime refuses is a lie the type system
  tells. It is the same `--check` admits / run raises shape this arc has been closing all session.
- **Good UX? YES** — one spelling works everywhere instead of working only off the builtins.

## What this UNBLOCKS — and what it deliberately does not do

```
UNBLOCKS   a wat-fix codemod repointing `:wat::core::None`/`Some`/`Ok`/`Err` to the qualified form.
           That codemod demolishes the 6346-site bridge 294 R9 named — "the enum got registered;
           the bare alias never followed" — and removes `:wat::core::None` from the SIX NON-VERB
           ARTIFACTS blocking Phase 3a.
NOT DONE   the codemod itself. This stone makes the target spelling WORK; migrating the corpus is
           its own stone with its own dry-run and diff (R21).
NOT DONE   removing the bare spellings. Additive only. The bridge comes down in the codemod's
           stone, not this one — demolishing a bridge before the far side is walkable is how an
           anneal fails in the other direction.
NOT DONE   296 STONE-H (variants are maps), 109's clause brackets, or 251's spelling flip. The
           roadmap puts the syntax work after crates and STONE-H's own text says the wire survives
           251 — nothing here is urgent and this stone touches none of it.
```

## Scope

**In:** the six `src/runtime.rs` guards gain the qualified alternative · a probe proving each of the
four builtin variants matches under the qualified spelling · the arc-109 comment at `:8324` updated,
since it currently calls `:wat::core::None` "the FQDN form" and a third, more-qualified form now
also works.

**Out, affirmatively:** the 21 `src/check.rs` sites (already correct) · the codemod · removing any
bare spelling · the enum redesign in any of its three arcs.
