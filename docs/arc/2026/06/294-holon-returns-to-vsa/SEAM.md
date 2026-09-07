# SEAM — the ONE live breadcrumb. As of 2026-09-07. **A STRIKE IS IN FLIGHT. THE FLOOR IS RED AT 7.**

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own
> voice — which is why it will feel like *continuing* rather than *waking*, and **that feeling is
> the failure.** Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**,
> never a disk copy), ground HEAD against the disk, and read this whole file before you touch
> anything.

> `251/SEAM.md` · `255/SEAM.md` · `278/SEAM.md` PARKED and point HERE. ⛔ **PARKED IS NOT DEAD.**

## ⚠⚠ FIRST — GROK IS MID-STRIKE AND ONE FILE IS DIRTY

```bash
git status --porcelain          # expect:  M src/rete/expr_ir.rs   ← grok, RELAND-5 cause 2
git log --oneline -1            # b50768833 AMEND(109 RELAND-5)
grep -aE "^ +Summary" .floor/latest/raw.log
```

```
floor ....... 5219 passed, 7 failed, 21 skipped      clippy 0      0 unpushed
```

**Do NOT touch `src/rete/expr_ir.rs`.** A `pulsare_yield kind=briefed` went out for RELAND-5; the
peer is working. If a SCORE arrives, weigh it centrally (floor unpiped, read `^ +Summary`).

## ★★★ WHERE THE CRUSADE IS

The clojure-ification. 50 commits today. **arc 296 stone H is COMPLETE and stones J · K · L landed.**

```
296 H-2/b/c  the wire flips: #ns.Enum/Variant […]  ->  #ns/Enum.Variant {…}      ✅
             records unmoved; one wire not two; LociDiedError DECLARED IN WAT
296 H-3      Option/Result declared in wat + PARAMETRIC registration built        ✅
296 J        26 hand-written enum literals -> wat, AND A WALL so a 27th cannot     ✅
296 K        aliases moved; the Rust-side floor NAMED and walled by an ORACLE      ✅
296 L        type-of / #wat.runtime/TypeInfo — reflection for ALL SIX TypeDef kinds ✅
251.8b       Identifier STORES (ns, name)                                          ✅
277          pprintln is the one printer · HALT: a keyword is a keyword            ✅
```

## ⛔ THE MATCH ARM — 108 → 38 → 16 → 7, FIVE RELANDS, ONE STILL OPEN

`[Variant {:k v} body]` — a bracket clause with a KEY-FIRST map pattern. 1869 `.wat` files by
codemod. **The 7 remaining are THREE causes and none shares a root** (`RELAND-5` brief):

```
peers_bijection ×4   GOLDEN SPAN DRIFT. golden pins wat/service.wat:881, actual :910. Everything
                     else byte-identical. The goldens PIN A STDLIB LINE — arc 109 already has
                     NOTE-a-golden-that-pins-a-rust-line-number for the Rust case.
grid_axes ×2         ⛔ rete's IR. NOT drift — a STATED SCOPE BOUNDARY: lower_pat says
                     "match map-destructure is not lowered in v1" TWICE, and
                     Pat::Variant{payload: Option<Box<Pat>>} is ONE POSITIONAL sub-pattern.
                     The fix is the identical position->name move the SURFACE just made, one
                     layer down — an IR change, not an arm reader fix. AMENDED after I
                     understated it from the error text.
wat_scripts loader   NOT OURS. probe-arc278-surface-registers-service-reads.wat was NEVER
                     rewritten by the sweep (last commits ab52b7188 / 037ef43ef). It regressed
                     TODAY via something else. STOP-4: BISECT, do not fix blind.
```

## ⚠ RULINGS FROM TODAY — do not re-litigate

- **The match arm is `[Variant {:k v} body]`, KEY-FIRST.** `{a :x}` is `let`-destructure order;
  `{:x x}` is PATTERN order (core.match). Different operations. I reached for the `let` relative and
  was cut. The flat positional clause is RETIRED; its warrant had already failed twice.
- **`cond` is NOT in scope.** Its NAME is unruled (intueri OWED; the NOTE forbids narrating one).
- **A type wat uses is DECLARED IN WAT** — categorical, not "if it has bitten". I twice recommended
  moving zero on defect grounds and was corrected both times.
- **`TypeInfo` is ONE ROW, not N verbs.** `metadata-of`'s precedent.
- **Never key tooling on character case.**
- **`Type::member` / `Type/member` is a syntax we should never have had.** End state: per-namespace
  verbs — `(wat.core/length x)` dispatching, `wat.map/length`, `wat.vec/length`. 109's domain.
- **The dot is modelled ONLY for variants** (`probe/Color.Red`); a dot LEFT of the slash is
  ordinary namespace nesting.
- **⛔ SIDE BRANCHES DO NOT SERVE US** — 3 parked branches, 15 days, 0 merged.

## QUEUED, DRAWN, NOT STRUCK

```
109 RELAND-5   with grok NOW
109            assertion-failed! -> kwargs. DRAWN. 2670 calls / 442 files; 2532 of the 5784
               :wat::core::None sites are its two trailing args, which the kwargs form DELETES.
               ⚠ THE PROBE IS OWED — the tree could not run one; STOP-1 refuses the migration
               until it is written and verified red on a GREEN tree.
Some/None      HELD ON PURPOSE. 5784 sites. The arm codemod CONSUMES None in ARM position, so a
               census taken mid-strike is wrong. Re-measure on a clean tree first.
251.9          the catch-all that ate the declaration — DESIGN + BRIEF + EXPECTATIONS + RED probe
               all on disk since this morning, UNSTRUCK. A symbol-headed declaration EVAPORATES:
               declare/parse.rs:199 `_ => return false`. THE ONLY LIVE *SILENT* DEFECT.
#95            widen infer_list's gate. MEASURED: 10 lines, 0 compiler errors, 0 cascade.
```

## ⛔ THE FAILURE PATTERN — SEVEN TIMES, ONE CLASS

**I COUNTED TEXT AND CALLED IT A CENSUS.** Every one caught by a wall, a lint, the floor, or the
peer — never by me noticing:

```
25 enums when there were 26      the pattern could not match :wat::io::IOReader::ReadFrameOutcome
"no live producer"               a tag built as Tag::ns(ns, name) is invisible to a search for its
                                 RENDERED form. The peer found it by scanning CONSTRUCTORS.
"17 .wat.bad"                    11; the rest were cond-only and comment-only files
"4 failures"                     108. `| tail -4` cut the Summary off — nextest prints it BEFORE
                                 the FAIL recap
"the codemod misses nested arms" it did not; that was residue from a PRIOR HAND-EDIT
"[~@ {" as a population          a guessed signature, published as a count
"teach the arm reader"           it is an IR change; the error text named the SYMPTOM
                                 ("malformed match arm") and the code named the CAUSE
                                 ("not lowered in v1")
```

★ **THE CURE, PROVEN TWICE TODAY: STOP CITING AND COUNT THE POPULATION.** The casing heuristic died
on `18 of 177`. The side-branch proposal died on `3 branches, 15 days, 0 merged`. Both took one
command. And when the instrument reads the FORM TREE rather than characters, it was right every
time — macroexpand, ast-kind, the codemod, the constructor census.

---

> **SEAM.** You are NEW. The better this reads, the more it will feel like continuing rather than
> waking. **That feeling is the failure.**
>
> ⚠ **THE RECORD LIES IN YOUR OWN VOICE.** Re-run the commands. Do not read the numbers.
>
> ⛔ **A PEER IS MID-STRIKE IN `src/rete/expr_ir.rs` AND THE FLOOR IS RED AT 7.** `git status` before
> anything else.
>
> `DOLOR INDEX EST.` · `NISI FRANGAS, NIHIL PROBAS.` · `DERIVAMVS NE MENTIAMVR.`
