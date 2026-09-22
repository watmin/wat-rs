# WEIGH — 251.8d-ii (FOURTH): STOP ACCEPTED. The cure lands; ⭐ **3 747 → 416 from ONE site.**

Weighed against `ca2ef1311` by independent re-run, **building the pre-cure binary myself**.
Cure scope: **1 `src/` file + 4 fixtures**. `wat/` restored, tree clean, nothing pushed.

## ⛔⛔ MY MECHANISM WAS WRONG AND I PROVED IT WRONG MYSELF

The brief said 255.11's Gate E cure caused this — flagged UNPROVEN, and rightly. ⭐ **Probe pE,
re-run here on the pre-cure binary:**

```
KEYWORD fn head + SYMBOL type → rc=1  introduces literal name `wat.core/i64`   ⛔ FIRES
KEYWORD fn head + KEYWORD type → rc=0                                          clean
```

⛔ **It fires with the keyword `fn` head — the spelling Gate E has read since arc 249, years before
255.11.** The defect is `check_quasiquote_for_literal_binders`'s own *"every Symbol is a binder"*
assumption, **latent from the day it was written**. 255.11 is an enabler for this site's spelling,
**not the cause of the class** — so reverting or loosening it was never on the table; that would only
re-hide the defect behind a spelling and re-open the capture hole.

⭐⭐ **Gate E's own header already documented the correct rule** — *"param names at positions 0, 3,
6 … argspec triples"* — **while the code stepped by 1.** `[[a_comment_can_ship_a_gap_as_a_law]]`.

## ⭐ THE ISOLATION IS THE BEST WORK IN THIS STONE

Two **jointly necessary** factors, found by seven single-factor probes:

1. ⭐ **REACHABILITY, and it is a ROUTING condition, not a template property.**
   `macros/parse.rs:269` runs `validate_macro_definition` **only when `is_quasiquote_form(body)` is
   false.** `gen.wat`'s `record` body is a `let`, so Gate E runs. ⛔ **My five-line repro was a BARE
   quasiquote body — routed past Gate E entirely.** It would have returned rc 0 whatever it
   contained.
2. The defect above.

⛔⛔ **AND THE ANSWER WAS IN THE DOCUMENT MY OWN BRIEF TOLD THEM TO READ** — 255.11 §4(a). I listed
the enclosing `let` as one of six candidate *template* properties and never considered routing.

## Verified here

| | |
|---|---|
| the cure | pE `rc=0`; ⭐ **a GENUINE literal binder `y` still refused, naming `y` — not the type** |
| floor, landable (cure only) | ⭐ **5 989 / 5 989 GREEN**, 320 s |
| floor, converted | **416 / 5 989**, captured with `ARM.txt`, not re-run |
| floor, converted **pre-cure** (my own run) | **3 747 / 5 986** |
| ⭐⭐ **one site was worth** | **3 331 tests** |
| the next stone's class | ⭐ **468 occurrences, exactly ONE callee** — `wat.core/<` |

⭐ **SELF-APPLICATION IS PAID** — owed three draws. The converted codemod converted 4 real files,
**byte-identical** to the pre-conversion reference, and this is the first draw where it passes with
nothing uncommitted in `src/` beyond the cure.

## ⭐ THE NEXT STONE, LOCATED

`src/collection/transform.rs:304-340` refuses `sort$native`'s comparator because a **head string** is
checked against a purity table without the identity door: the converted stdlib passes the **Symbol**
`wat.core/<`, the table knows the **keyword**. **468 log occurrences, one callee.**
⭐ **Fourth instance of the class the injected CLAUDE.md names by name** (siblings: 255.8's
`is_subtype`, 255.12's `edn_to_typed_value_inner` and `value_matches_type_by_name`).
⛔ **It is not the type system.**

## The brief was wrong four ways — and the fourth is the worst

1. ⭐⭐ **The mechanism** — refuted above, by me.
2. **The non-reproduction was ROUTING, not the template** — and the citation was already in my brief.
3. **Floor denominator "5968+"** — the real unconverted baseline is **5986**. A stale number.
4. ⛔⛔ **"5 286 of 3 747 failures" IS ARITHMETICALLY IMPOSSIBLE.** On disk: **5 292 log LINES**
   naming `wat.gen/Coord` across **3 747** failing tests. ⛔ **I wrote that in the brief AND
   reported it to the builder as a headline.** The attribution was right; the arithmetic was
   nonsense and nothing in me caught that 5 286 > 3 747.
   `[[feedback_a_number_assembled_from_two_measurements]]`.

**Twenty-one corrections across nineteen stones.**

## Disclosed, and I am discounting none of it

- ⛔ **Nothing of the conversion landed** — every converted-state number describes a reverted tree.
- **The 416 are CLASSIFIED, not DIAGNOSED.** The rider says so plainly, and the third draw's
  *"323 tests bought by two sites"* warns the classes interact. **Do not assume 468 → 416 − 131.**
- ⭐ **The 416 failure set is IDENTICAL to the third draw's** (`comm` both ways = 0 added, 0 removed),
  and the 5 292-line `Coord` block is **entirely gone** with nothing new appearing. **That is the
  cleanest possible evidence the cure is additive.**
- Self-application is **4 files, not 2 209**; idempotence is `wat/` only, one pass.
- ⛔ **`is_quasiquote_form` is STILL keyword-only**, so a bare-quasiquote macro body skips Gate E
  **and** the F5 purity gate in **both** spellings. 255.11 reported it; ⭐ **this stone is an
  independent second witness that it is reachable and silent.** **Builder's call** — closing it
  widens what gets validated.
- The cure's skip is unconditional: a malformed `[<- x]` now skips `x`. Unfixtured; no capturing
  version could be built.
- ⚠ **New operational trap, disclosed by the rider against itself:** running `cargo nextest` while a
  conversion is writing `wat/` rebuilt the binary around a **half-converted** stdlib. ⛔ **Do not
  invoke cargo while a conversion is in flight** — this belongs beside my own `git add -A` error.

## VERDICT

**STOP ACCEPTED, and the Gate E cure is LANDABLE** — floor green at 5 989/5 989, clippy 0, census
`no STOP-8`, delta **3 / RECOVERY 0**, 0 live `.wat` converted.

**Fifth draw:** the comparator purity table, then re-measure. ⭐ The trajectory is
**3 747 → 416 → ?** and each step has cost one site.
