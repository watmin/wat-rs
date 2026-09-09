# ⛔ NOTE — the blanket census does NOT reproduce, and `registry().contains` is not its replacement

**2026-09-09, re-derived on a quiescent tree at HEAD `74951196a`** (clean, 0 unpushed, floor
5292/5292, clippy 0). `HAERESIS EST ITERVM ROGARE` — the settled number was re-asked and it moved.

## The instrument

`DESIGN-the-blanket-dies-in-three.md` and `BRIEF-STONE-resolve-asks-the-registry.md` scope the whole
queue from **143 of 845 files · 35 distinct names**. That instrument was recorded as preserved at the
session scratchpad; **the scratchpad was empty** — the reboot wiped it. Rebuilt from the brief's own
recipe, `src/resolve/walk.rs`:

```rust
if is_reserved_prefix(head) {
    return crate::intrinsic::registry().lookup_entry(head).is_some();   // the brief's recipe
}
```

Scope recovered exactly: `845 = wat-scripts/ (702) + wat-tests/ (81) + wat/ (62)`.

## The measurement — it does not reproduce

```
clean main, 845 files                       36 refuse   (the standing baseline)
registry().lookup_entry instrument         600 refuse · 309 distinct reserved names
registry().contains      instrument        595 refuse · 289 distinct reserved names
DIFFERENTIAL — pass today, break then      564 files
```

`contains` minus `lookup_entry` is 20 names — the membership facet (stone ①′) doing exactly and only
what it claimed. That axis is sound. **The 143/35 figure is not.**

## ★★★ WHY — resolve asks ONE store, and the stdlib's own surface is not in it

Of the 309 refused names, **46 are defined by a `defn`/`defmacro` in an EMBEDDED stdlib file** —
`:wat::core::map-indexed`, `:wat::core::take-while`, `:wat::core::remove`, `:wat::core::keep`,
`:wat::fix::structural?`, `:wat::test::assert-eq`, the `:wat::bracket::*` workers, `:wat::cache::Lru::*`.

The worked example, one process, two passes disagreeing about one name:

```
(:wat::test::assert-eq 1 1)      → resolve: ":wat::test::assert-eq"
                                    "call head — not a builtin, not a registered function"
(:wat::test::assert-eq 1 1 "x")  → check:   ":wat::test::assert-eq: expected 2 argument(s); got 3"
```

The **type-checker knows its exact arity** while **resolve calls it unresolved**. `wat/test.wat` is
embedded (`src/load/stdlib.rs:309`) and `register_stdlib_defines` (`src/declare/register.rs:711`)
does `sym.register_function` at step 6, before resolve at step 7 — so the ordering is not the
explanation, and `sym.get`/`has_function` read the same `functions` map. **The gap is real and its
mechanism is unmeasured.** It is the next thing to measure, not to assume.

> **The blanket is not merely hiding un-homed verbs. It is what makes the stdlib's own wat-defined
> surface resolvable at all.** `resolve` asking `registry().contains` replaces one blanket with one
> store, when the truth lives in a union — the fourth occurrence of
> `[[feedback_a_name_checked_against_a_partial_set]]` in this campaign.

## ② is real, and the DESIGN names the wrong pass

Within the 845 the normalize-pass refusals are exactly the four the DESIGN names:

```
9 :wat::type::Tuple · 7 :wat::type::i64 · 5 :wat::type::String · 2 :wat::type::Vector · 1 :wat::core::+
```

But the context string on every one is **`"namespaced symbol ref — not a builtin, not a registered
function (arc 251)"`** — `src/resolve/normalize.rs:461`, **not** `walk.rs`'s `"call head …"`.

The DESIGN says *"Type ARGUMENTS are being walked as CALL HEADS"* and points at `resolve/walk.rs`.
The four names are `WatAST::Symbol`s — `wat.type/Tuple` — and they are refused by
`resolve_namespaced_symbol`, which asks `is_resolvable_call_head` **"may this symbol be rewritten to
this keyword FQDN?"** — a call-head predicate answering a question about a TYPE position.

`walk.rs:87` carries the `:-` type-reference guard (arc 109). `normalize.rs` runs FIRST and has no
such guard. The walk.rs comment already names this exact class — *"the expander was taught this
first; the resolver is a SECOND, INDEPENDENT consumer of the same shape and was not"* — and this is
its **third** consumer. `[[feedback_a_slot_with_two_implementations_is_two_slots]]`

⚠ Corpus-wide (1952 files, incl. `tests/`) the family is **eight**, not four: `+ bool (4) · nil (2)
· f64 (2) · HashMap (1)`. Four is a property of the 845 scope, not of the defect.

## The DESIGN's own arithmetic

`DESIGN-the-blanket-dies-in-three.md` states **35 distinct names** and then decomposes
**11 (rete) + 4 (type) + 4 (verbs) = 19**. Sixteen names are named nowhere in the three dispositions.

## What this does NOT overturn

- **①′ landed and is sound.** The membership facet is measurable, and its 20-name delta is visible
  in the two instrument runs above.
- **③ stands** — four real verbs with live dispatch arms and no rows is still true.
- **The order still stands.** `(:wat::core::Option.Some {:value 7})` → `check=0` →
  `#wat.core/Option.None {}` on clean main. The blanket still dies before the dot flip.

## What must happen before another stone

The queue's terminal step — *"then `resolve` asks `registry().contains`, the blanket dies"* — is
**under-specified**, and ①②③ move the census from 309 names to roughly 290. The blocking question is
not which names to register. It is **which stores answer "does this name exist?"**, and the campaign
has already paid for that lesson three times.

**⛔ The disposition-by-count trap applies to this NOTE too:** 309 is not a worklist. It is the
measure of a predicate asking one store.
