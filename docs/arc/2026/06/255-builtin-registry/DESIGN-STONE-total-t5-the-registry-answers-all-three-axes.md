# DESIGN — STONE total-T5: `intrinsic_meta` derives ALL THREE axes; the ledger evaporates

> Builder: *"i care less about a number and more about forward progress to the registry being the
> single source of truth."*

T4b made the `total` axis derive. This stone makes `pure` and `deterministic` derive too — and
unlike `total`, **no transcription stone is needed first**, because `@Purity` and `@Determinism`
have been mandatory on every registration since long before this campaign.

```
total          ✅ derives (T4b) — needed T4a first; 431 verbs answered `Unreviewed`
pure           ⬜ can derive NOW — the answers already exist
deterministic  ⬜ can derive NOW — likewise
```

## The change

```rust
fn intrinsic_meta(head: &str) -> Option<OpMeta> {
    …rete_op_for and the existing early-return special cases, unchanged…

    if let Some(e) = registry().lookup_entry(head) {
        return Some(OpMeta {
            pure:          matches!(e.purity,      Purity::Pure          | Purity::Preserving),
            deterministic: matches!(e.determinism, Determinism::Deterministic | Determinism::Preserving),
            total:         matches!(e.totality,    Totality::Total       | Totality::Preserving),
        });
    }
    // Unregistered: the residue keeps its hand-ruling until the verb has a home.
    …
}
```

★ `Pure | Preserving` and `Deterministic | Preserving` are the convention already on disk
(`intrinsic/mod.rs:1038`, `intrinsic/reflect.rs:83`), and T4b established `Preserving => true` for
totality on the same argument.

## ⛔ THIS ONE MOVES VERDICTS — and the containment is MEASURED, not argued

Unlike T4b, this is not verdict-neutral. **275 registered verbs are not named in `intrinsic_meta`
today** and would go from `None` (no ruling ⇒ the fence denies) to a declared ruling.

Measured by probing the registry itself, not by grepping text:

```
NEWLY_RULED = 275      PURE_AND_DET = 163      ALSO_TOTAL = 0
```

**Zero.** Every one of the 275 carries `@Total Unreviewed`, so `total` comes out `false`, and arc
278's fence requires `pure ∧ deterministic ∧ total ∧ primitive`. **The fence admits exactly zero
additional verbs.**

★ That is `Totality::Unreviewed` earning its keep three stones after it was minted. T1 chose a
fourth variant specifically so an unmeasured verb would be *default-deny* rather than a guessed
pole — and that choice is what makes this stone safe to land without re-auditing 275 verbs.

## What actually changes, then

**The completeness gate's arithmetic.** 275 verbs stop being "unreviewed" — because they never were.
Their rulings were in the registry the whole time and the gate was asking a hand-list instead.
`KNOWN_UNREVIEWED`'s 228 rows will mostly go **stale**, and the gate's own `stale` assert will
demand they be deleted.

**That is the deliverable, not a side effect.** A 228-row ledger of "unreviewed" verbs, most of
which carry a declared purity ruling made at registration time, is the clearest possible statement
that the registry was not being consulted.

## The one contract decision, pinned

**The residue is defined by NON-REGISTRATION, never by a name-list.** After this stone, a verb falls
through to hand-ruling if and only if `registry().lookup_entry(head)` returns `None`. There is no
"these 133 derive, those 275 do not" list — that would be the hand-list wearing a new coat.

## Out of scope = REJECTED

- **The early-return special cases** (`uuid::v4`, `hashmap/map::keys/values`, `stream::next`,
  `aggregate-new`, …). Verified: each already AGREES with its registration's declaration, so they
  are redundant rather than wrong. Retiring them is a follow-up; doing it here would conflate a
  derivation with a cleanup.
- **`is_pure_total` and `RETE_OPS`.** Two more totality consumers, neither this stone.
- **`effectful_by_prefix`.** Dies when its last 17 customers are homed.

## Calibration

Predicted 40–60 min. The code change is small; the ledger pruning and proving 275 verdict-changes
are safe is the work.
