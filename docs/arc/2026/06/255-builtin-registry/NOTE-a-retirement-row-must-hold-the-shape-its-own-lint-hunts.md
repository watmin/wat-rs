# NOTE — a `RETIREMENT_TABLE` row must hold the exact shape its own lint exists to hunt

> Found by the rider on STONE `sort$native` (2026-08-30), **not predicted by that stone's DESIGN.**
> Verified by the orchestrator against the lint's source. **No row, nothing drawn** — this records a
> structural tension so the next primed retirement inherits the measurement.

## The shape

Retiring a primed verb requires a `RETIREMENT_TABLE` row (`src/remedy/retirement.rs`) whose
`retired:` field holds the **exact old spelling**, because `retirement_lookup` matches the literal
string a stale caller still types:

```rust
RetirementEntry { retired: ":wat::core::sort'", replacement: ":wat::core::sort$native", note: None },
```

That string is, by construction, *"a primed identifier inside a Rust string literal"* — precisely
the shape `tests/lint/retired_name_justified.rs` exists to catch. **So the act of retiring a primed
name creates a new hit for the lint that made retiring it worthwhile.**

★ The retirement row is not a leftover. It is the mechanism that turns a vanished name into a
teaching error — measured working this stone:

```
':wat::core::sort'' is retired; use ':wat::core::sort$native' instead
:remedies [#wat.kernel/Remedy {:form ":wat::core::sort$native" :kind :retirement …}]
```

## What is NOT a problem — measured, correcting the rider's worry

The rider flagged that the lint's taxonomy names only two honest reasons (`readln'`'s macro pair;
`Frame'`'s positional ctor) and that a retirement row is neither. **The gate does not care.**
`retired_names_are_justified` tests exactly one thing:

```rust
.any(|l| l.contains("// rune:lint(retired-name)"))
```

The taxonomy lives in the **assertion's failure message**, as guidance for a human — it is never
matched against. A rune with any reason keeps the floor green, so the floor was never at risk and no
"fourth taxonomy reason" is needed to stay green.

## What IS the problem — and it is the ladder, not the gate

Every future primed retirement will hand-write a rune for a site whose exemption is **structural,
permanent, and knowable without judgement**. That is the CONVENTION rung
(`extirpare`): a rule each future hand must remember, for a case the code can decide by itself.

Two rungs above it, in order of preference:

1. **A CHECK the lint makes itself** — skip hits inside `RETIREMENT_TABLE`'s `retired:` field.
   The field's whole contract is "hold a name that no longer resolves," so a hit there is not
   evidence of anything. This is where the class dies.
2. **A named reason in the taxonomy** — cheaper, still a convention, and it keeps the exemption
   visible per-row. Honest, but it does not stop the next hand from forgetting.

⚠ **The tension is real in one direction only.** The lint's job is *"a wat name in a Rust string must
be a name a user can type."* The retirement table is the one place in the substrate where the
opposite is required. That is not an exception to the rule — it is the rule's own escape hatch, and
it deserves to be recognised structurally rather than re-argued at each row.

## The acceptance row this refutes — the orchestrator's, not the rider's

`DESIGN-STONE-sort-prime-becomes-sort-native.md` shipped this bar:

> | the five runes are gone | `grep -rn "rune:lint(retired-name)" src/ \| grep sort` | no output |

**It cannot pass, and never could** — the retirement row legitimately contains both `sort` and the
rune marker. The bar was written from what the orchestrator EXPECTED the end state to look like
rather than derived from what the rule requires, which is the exact failure
`[[feedback_an_acceptance_row_is_a_pin_unless_it_derives_its_bar]]` names. The correct bar is
*"no rune survives at a site whose name no longer contains a prime"* — five retired, one born, and
the one born is structural.

## What retires this NOTE

A ruling on rung 1 or rung 2. Until then, a primed retirement carries a hand-rune on its
`RETIREMENT_TABLE` row, and whoever writes the next one should read this first rather than
rediscover it.
