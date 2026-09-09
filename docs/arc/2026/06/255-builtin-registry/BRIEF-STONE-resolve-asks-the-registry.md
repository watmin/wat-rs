# BRIEF — STONE: `resolve` asks the registry (arc 255's founding sentence)

## The arc's own opening text, still unshipped

`docs/arc/2026/06/255-builtin-registry/DESIGN.md`:

> *"The reserved-prefix blanket-accept hack is DELETED; builtins resolve through the same path as
> user forms. **One resolution path for everything.**"*
> *"The undefined-func class dies as a **side effect** of fixing the real defect."*

`src/resolve/walk.rs:272` today:

```rust
if is_reserved_prefix(head) {
    return true;
}
```

## What is now measured, and what it changes

`is_resolvable_call_head` asks exactly five things: the blanket, `sym.get` (user functions),
`sym.has_unit_variant`, `macros.contains`, and surface-method heads. **The 457-row intrinsic
registry arc 255 built so resolve could ask it is not among them.**

```
registry()                      src/intrinsic/mod.rs:586 — a FREE fn returning &'static
IntrinsicRegistry::lookup_entry  the membership question
```

**No plumbing, no signature change, no env.** The blanket could not be deleted before because
nothing replaced it; the replacement has existed all along and was never wired.

⚠ I removed the blanket *without* a registry lookup and measured **777 of 845 files refusing on 470
names** — `fn`, `def`, `match`, `let`, `println` among them. That is not the corpus being broken;
it is the measure of what the registry answers for and resolve cannot ask.

## ① THE CENSUS COMES FIRST

Swap the blanket for `registry().lookup_entry(head).is_some()`, build, and run `--check` over
`wat/`, `wat-scripts/`, `wat-tests/`. **Report before fixing anything:** how many files refuse, the
DEDUPED distinct names in frequency order, and one verbatim block per family.

⚠ **255's own census is dated 2026-09-07 (143/833, 17%, 35 names) and SEVEN STONES have landed
since.** Re-derive it; do not cite it. `HAERESIS EST ITERVM ROGARE` — the record's settled numbers
expire, and the ones that hurt are the right ones read past their date.

## ⛔ The one move that would defeat the stone

**Do NOT add names to the registry to make the corpus pass.** The census is the deliverable; a name
the corpus calls and the registry does not know is a FINDING, and which of them deserve rows is the
orchestrator's to shape. Registering thirty-five names to turn a number green is the tail wagging
the dog, and it would bury exactly the information this stone exists to produce.

## Acceptance

```
the census table on disk, with counts and one verbatim block per family
cargo nextest run --release -E 'test(p1_annotation)'  10 · 'test(a1_one_rule)' 4 · 'test(a2_a_variant)' 15
```

If the census is small enough that the blanket can come out in this stone, say so and do it. If it
is not, **the census alone is a complete deliverable** — that is the honest outcome and the one 255
has been missing for months.

## STOP triggers — each is a REJECTION

**STOP-1.** If the census exceeds ~40 distinct names — STOP and report. That is a campaign.

**STOP-2.** If you find yourself registering intrinsics — STOP. See above.

**STOP-3.** If `registry()` cannot be reached from `is_resolvable_call_head` — STOP and report the
borrow/lifetime reason. I measured it as a free `'static` accessor; if that is wrong, my premise is.

**STOP-4.** If any earlier stone's filter moves — STOP with the verbatim block.

## Tier

You edit and report. **Do NOT commit. Do NOT run `scripts/floor.sh` or clippy** — the orchestrator
runs those centrally, once.

⛔ **Do NOT call `pulsare_yield`, `pulsare_knock`, or contact any peer.** Report to the orchestrator
only. Previous riders knocked the peer unasked and it cost tokens the builder needed elsewhere.
