# NOTE (arc 109) — the Persistent collections were admitted WITHOUT the parametric discipline. 238 sites.

**Filed 2026-08-20. MEASURED.** Surfaced by the builder while ruling that the type vector is mandatory:

> *"the Persistent collections were built wrong and i didn't catch it… we need to force them into
> compliance… 'built wrong' meaning we don't impose parametrics explicitly…. rete brought them in for
> perf… and we never added the strong parametric requirements.. rete basically just does
> `HashMap<Value,Value>` to handle substrate speed."*

## The measurement — the std family holds the discipline; the persistent family never had it

Type position only (immediately after `<-` or `->`), across `wat/` + `tests/**/*.wat`:

| type | parametric | BARE | |
|---|---|---|---|
| `Vector` | 560 | 1 | disciplined |
| `HashMap` | 26 | 3 | disciplined |
| `HashSet` | 1 | 0 | disciplined |
| **`PersistentVector`** | 210 | **102** | ⚠ a third bare |
| **`PersistentMap`** | 4 | **136** | ⛔ **34:1 INVERSION** |

**238 bare persistent-collection annotations against 4 bare std ones.**

⚠ A first pass at this counted `HashMap` "BARE=323" — it was conflating CONSTRUCTOR CALLS with type
annotations. The table above counts type position only. `[[feedback_a_file_count_is_not_an_item_count]]`

## The mechanism — nothing enforced it, and only one family noticed

A bare head is legal for **all five** — measured, `check exit=0` for bare `Vector`, `HashMap`,
`PersistentVector`, `PersistentMap`. The checker has never required type args on any of them. The
std family is parametric by CONVENTION (the strongest rung anything here stands on), and the
persistent family, admitted later and for a different reason, simply never picked the convention up.

**That is the whole defect: a convention that was never a check.** It held for the family that was
present when it formed and lapsed silently for the family that arrived afterwards
(`[[feedback_a_gate_freezes_names_never_a_count]]` — nothing froze these names either).

## rete is the origin and still the worst site

```
wat/rete.wat:30, :37, :1830, :2383    bindings <- :wat::core::PersistentMap
```

Four bare annotations on the engine's own Token bindings — the hottest structure in the substrate.
The honest annotation IS writable today; measured:

```
(:wat::core::PersistentMap [:wat::core::Value :wat::core::Value])   check exit=0
```

So compliance is not blocked on anything. It was simply never asked for.

## Consequence for the mandatory-type-vector ruling

These 238 are the largest concentration of sites that will change meaning-of-annotation (from
"unspecified" to "explicitly `Value,Value`" or to whatever the real K/V turn out to be) rather than
merely changing shape. **They are NOT the same as the 977 bracket-less CONSTRUCTOR calls** — those
are value position; these are type position. Both are forced by the ruling; they are different
populations and must be counted separately.

⚠ **The honest K/V for rete's four is UNMEASURED.** `Value,Value` is what the builder describes
(*"rete basically just does HashMap<Value,Value>"*) and it type-checks, but whether a tighter, truer
type exists at each site has not been looked at. Writing `Value,Value` everywhere would make the
annotation honest about what the code does while possibly cementing a weaker type than the data
supports. **Look before writing.**

## ⚠ OPEN AND UNMEASURED — the tag question

Builder: *"we should have support for their tags… that is odd."* The substrate PRINTS
`#wat.core/PersistentVector [1 2 3]`. Whether it can READ that back is **not established here**: a
probe found the tagged form unreadable, **but its control also failed** (`#user/A {:x 1}`, a plain
record, likewise did not read), so the probe measured the reader's tag support in general rather than
these types. If the print/read round trip is genuinely broken, that is a real defect and adjacent to
arc 300's thesis that wat source IS edn — but it needs a probe whose control fires before anyone acts
on it.

⚠ Also noticed, also unverified: bare `:wat::core::Any` was accepted as a parametric type-arg in a
throwaway probe, while `types.rs`'s `reject_any` is documented as enforcing an `:Any` ban in
parametric heads/args. The probe's fn never USED the parameter, so this may be a shallow measurement
rather than a gap. Do not act on this sentence — measure it.
