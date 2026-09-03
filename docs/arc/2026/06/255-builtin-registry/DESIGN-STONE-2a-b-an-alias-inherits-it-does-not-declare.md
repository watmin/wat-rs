# DESIGN — STONE 2a-b: an alias INHERITS its axes; it does not declare them

> ⛔ **This stone exists because the previous one minted a defect, and it is the campaign's own
> defect: two names, one behaviour, contradicting each other about what that behaviour is.**

## The measurement — read back out of the live registry

`:wat::rete::i64::>` is an alias of `:wat::i64::>`. The registry re-dispatches one to the other, so
they are the same behaviour **by construction**. Each declares its own five axes:

```
                    :wat::i64::>  (target)      :wat::rete::i64::>  (alias)
@Purity             Pure                        Pure
@Determinism        Deterministic               Deterministic
@Totality           Total                       Partial          ⛔ CONTRADICTS
@ExpandTime         Legal                       Legal
@Category           Probe                       Reflection       ⛔ CONTRADICTS
```

`render-doc` prints both, live: *"Category: Probe"* and *"Category: Reflection"* for one behaviour.

★ **The drift took under an hour.** Not months, not a forgotten table — the same session, by the
hand that wrote the RULING. That is the strongest available evidence for what this stone rules,
and it is why the fix is structural rather than "be careful when authoring aliases."

⚠ **How it happened, recorded because the mechanism matters more than the instance:** the rider
argued `Partial`/`Reflection` for `:wat::rete::i64::+`, a `Fallback` row with `:undefined`
machinery — a *correct* argument for that row. The orchestrator then re-pointed the witness to
`:wat::rete::i64::>` and carried the axes across unexamined. **A per-row judgement outlived the row
it was made about.** That is precisely what a declared-per-row axis invites.

## ★★★ THE CONTRACT — an alias declares NO axes, and the registry answers with its target's

```
alias_of: Some(core)   ⇒   purity · determinism · totality · expand_time · category
                            are the TARGET's, resolved at fold time. The row declares none.
```

An alias is not a verb with properties of its own; it is a **second name for one verb**. Asking it
to restate the target's five axes is asking for five opportunities to disagree, and the measurement
above is what taking one of them looks like.

★ This is Shape D — GENERATE — for the second time in two stones, and for the same reason: once the
registry holds the relation, the derived facts stop being authored. 2a made dispatch derived; this
makes the axes derived.

## What must change

```
crates/wat-doc          a row WITH @alias is exempt from the five required directives —
                        declaring one becomes an ERROR, not an option
crates/wat-macros       both proc-macros stop demanding them for an alias row
src/intrinsic/mod.rs    registry() resolves an alias row's axes from its target at fold time
the witness             loses its five axis lines and its Totality/Category contradiction with them
```

⛔ **Declaring an axis on an alias must be a hard error, not a silently-ignored field.** A row that
states `@Totality Partial` and is answered `Total` is worse than today's contradiction: it lies in
the source while the registry says otherwise, and no reader would know which won.

⚠ **Fold order becomes load-bearing.** The target may fold after the alias. The resolution must
happen after every submission is in, not during the loop — and a gate must prove the resolution
actually ran (an alias whose axes silently defaulted would be invisible).

## THE FOUR QUESTIONS

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **alias declares none; registry derives from target** | YES | YES | YES | YES | ✅ **PICKED** |
| fix the witness's two axes, keep per-row declaration | YES | YES | **NO** | — | ⛔ DISQUALIFIED |
| add a gate: an alias's axes must EQUAL its target's | YES | YES | **NO** | — | ⛔ DISQUALIFIED |
| let an alias override, gate only the unstated ones | YES | **NO** | **NO** | — | ⛔ DISQUALIFIED |

- **fix-the-witness Honest? NO** — it repairs one instance of a class that just demonstrated it
  recurs within the hour, and Phase 1b would author 54 more chances to diverge.
- **equality-gate Honest? NO** — that is the RULING's own disqualification verbatim: *"a gate that
  compares two tables is a measurement of the split, not a cure for it."* It would also force 54
  rows to restate five axes each, purely to be checked against the row they copied.
- **allow-override Simple? NO, Honest? NO** — an alias that *disagrees* with its target is either a
  lie or evidence it is not an alias. There is no third case, so there is nothing an override could
  honestly express.

## Acceptance — rows chosen to be unfakeable

| what | expected |
|---|---|
| the contradiction is GONE | `render-doc` on both names reports the same five axes |
| ⛔ and it is gone by DERIVATION | the witness's source declares no axis line at all |
| ⛔ declaring one is REFUSED | add `@Totality Total` to the alias → **compile error**, naming it |
| ⛔ the derivation actually RAN | a gate proves every alias row's axes match its target's, and names the rows it inspected |
| ⛔ NON-VACUITY | that gate inspects ≥ 1 row |
| ⛔ a dangling alias still fails | 2a's target gate, unchanged |
| non-alias rows unchanged | every other row still declares its own five |
| floor · clippy | green · 0 |

★ **Row three is the stone.** Deriving silently while still *accepting* a declaration would leave the
source able to say one thing and the registry another — the same defect with better manners.

## What this buys Phase 1b

**54 alias rows** (`Alias` 35 · `Form` 9 · `Redispatch` 10) become *a name and a target* — no axis
authoring at all. That is the difference between 54 five-axis arguments and 54 one-line facts, and it
is what makes 1b a single stone rather than a campaign.

⛔ **`Fallback`'s 20 are NOT aliases and are not in it.** They carry real `:undefined` machinery and
their `total: true` is a property of that machinery, not of the verb they name — the finding that
produced this stone. Phase 2b.
