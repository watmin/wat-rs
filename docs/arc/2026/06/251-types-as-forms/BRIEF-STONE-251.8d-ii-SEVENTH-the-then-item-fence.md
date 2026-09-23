# BRIEF — STONE 251.8d-ii (SEVENTH DRAW): the `:then` item fence

**Drawn 2026-09-23 against `main` @ `1448c98d0`.** Landable floor 6008/6008, clippy 0, census
`no STOP-8`, delta **3 / RECOVERY 0**, ⭐ **heresy ledger 223** (shape **B = 1**).
Converted floor **161**. Trajectory **3 747 → 416 → 283 → 161**.

## ⭐⭐ THE ADDRESS WAS ISOLATED BY EXPERIMENT, NOT INFERRED

⛔⛔ **The last THREE briefs each convicted the wrong address** — a range with no instance of the
pattern; dead code behind a guard; a fire-time function blamed for freeze-time errors. ⭐ **The
orchestrator has now adopted the sixth draw's own method — SINGLE-FILE CONVERSION — and ran it before
writing this line.**

**Measured, ~90 s:**

| step | result |
|---|---|
| `wat::rete probe_arc278_then_is_an_expansion_boundary::both_fact_form_shapes_expand_their_values`, unconverted | ⭐ **PASS** |
| convert **`wat/rete/compile.wat` ALONE**, rebuild, re-run | ⛔ **FAIL** |

⭐ **ONE FILE REPRODUCES IT.** `wat/` restored, tree clean. **That is the address; everything below
it is a lead.**

## The class

| | |
|---|---|
| failures carrying `… is not a rete primitive` | **117 of 161** |
| occurrences naming `:wat::core::kwargs-construct` | **141** |
| occurrences naming anything else | **1** (`:wf::core-fold`) |
| ⭐ the 5 NEW the sixth draw opened | **the same fence** |

The fence (`wat/rete/compile.wat:795`, and twins at **461** and **607**):

```wat
is-rete   (:wat::rete::primitive? item)     →  converted:  (wat.rete/primitive? item)
```

## ⚠⚠ WHAT IS **NOT** ESTABLISHED — DO NOT INHERIT THESE AS FACTS

1. ⛔ **Whether `primitive?` is itself spelling-sensitive is UNKNOWN.** The orchestrator probed it
   directly and got `false` for **all four** inputs — including `(:wat::core::+ 1 2)`, which should
   be a primitive. **The probe's argument shape is therefore wrong and it proves nothing in either
   direction.** ⭐ **Re-probe it properly; that is question one.**
2. ⛔ **WHICH construct inside `compile.wat` is responsible is UNKNOWN.** The file has three
   `primitive?` sites and much else. ⭐ **Narrow it further** — the same method works on a hunk, a
   `def`, or a single form.
3. The ledger's candidate is `rete/purity.rs::is_declaration_derived_construction` (2 sites,
   shape A). ⚠ **A CANDIDATE. The last three briefs died of exactly this kind of confidence.**

## ⛔⛔ FOURTH CONSECUTIVE PERMISSIVE CURE — AND THIS ONE IS A WALL

`then-item-fence` is **Law A**: it refuses core-spelled COMPUTATION inside a `:then` item. ⭐ **Making
it admit more is the red → false-green direction, and here the thing admitted runs in a rule head.**

⭐ **NON-VACUITY, MANDATORY, IN ONE TEST:**
1. A legitimate constructor in `:then` compiles — **both spellings**.
2. ⛔ **A genuinely non-primitive item is STILL REFUSED — both spellings**, same located reason.
   ⭐ **Law A must still refuse core-spelled computation**; the sixth draw proved `wat.core/<` in a
   `where` is still refused, and the equivalent must hold here.
3. ⛔ **The sixth draw's 5 NEW must be re-examined**: it kept a cure that **re-armed a `:then`
   expansion the conversion had silently switched off**, and those five now fail *correctly*.
   ⭐ **If your cure makes them pass, say WHICH mechanism did it** — a wall re-disarmed looks
   identical to a bug fixed.

## The work

1. ⭐ **Narrow inside `compile.wat`** by the same single-file/single-hunk method. **Report the
   discriminating change.**
2. Answer question 1 above — is `primitive?` spelling-sensitive? **Probe it correctly.**
3. Cure through the identity door, one door, Keyword arm byte-identical (255.13's DUAL-ARM RULE).
4. The three non-vacuity rows.
5. ⭐ **Re-measure:** convert live `wat/`, build, floor.

## The gate

- ⭐ **Ledger drops and is RE-FROZEN in the same commit**; ⛔ **report the SHAPE MIX, not just the
  count.** ⚠ **Shape B is down to ONE site** (`purity.rs::walk_rete_defn_callees`), which currently
  carries the calibration anchor — ⛔ **if you cure it, re-anchor per the note already written into
  the test; do NOT delete the assertion.**
- ⭐ **Converted floor: 161 → ?** ⛔ **Report it even if unchanged**, and ⭐ **give FIXED/NEW against
  the sixth draw's 161** — more informative than the total. ⚠ **Use `clean.log`, and strip the ENTIRE
  `FAIL […] (n/m)` prefix; `ARM.txt` REPEATS each failure and a padded index `( 167/6008)` broke the
  orchestrator's regex once already.** ⛔ **DO NOT PROMISE 161 − 117.**
- The three non-vacuity rows, in one test.
- `scripts/floor.sh` green on the **landable** state; clippy 0 (⚠ **not cached** — touch first or
  check elapsed); census `no STOP-8`; `scripts/replay/delta.sh` baseline **3**, ⛔ **RECOVERY
  non-zero is a STOP**.
- ⭐ **LAND THE CONVERSION ONLY IF THE CONVERTED FLOOR IS GREEN.** Otherwise `git checkout -- wat/`,
  rebuild, report. **Five draws have stopped correctly; all five were accepted.**

## ⛔ Operational traps

- ⛔⛔ **NEVER `git add -A`, NEVER invoke cargo, while a conversion is writing `wat/`.**
- ⭐ **The single-file loop is ~90 s** (convert ~2 s · build ~22 s · test ~1 s · restore+build ~22 s).
  **Use it before the 22-minute whole-stdlib cycle. It is what found this address.**
- ⛔ `git ls-files 'wat/**/*.wat'` → 31 of 64. Use `git ls-files | grep -E '^wat/.*\.wat$'`.
- ⛔ The stdlib is `include_str!`'d — rebuild between steps and say you did.
- ⚠ A fast floor is a symptom (24 s = stdlib did not load; ~330 s healthy; ~450 s converted-but-red).

## Doctrine

- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** ⚠ One bookkeeping exception: `harvest_wrap_split`
  (`278-rules-engine/FINDING-harvest-cost-is-not-a-gate.md`). **Cite and report; never a pass, never
  re-run to clear. Name it either way.**
- ⛔ **REPORT WHAT A DOOR CANNOT EXPRESS.** Eighteen stones running.
- ⛔ **If this brief contradicts the code, THE CODE WINS.** ⭐ **Twenty-four corrections across
  twenty-two stones.** ⭐⭐ **This brief's ADDRESS is measured; its MECHANISM is not. Treat §"what is
  not established" as the honest boundary and assume a twenty-fifth.**
- Work only in `/home/john/work/holon/wat-rs`. **Do not push.**

## Out of scope

Shape **E** — ⛔ **125 sites, 78 in `check.rs`, UNTOUCHED; the terminal cut needs A+E at zero, not
just heads** · `is_where_form`'s keyword-only guards in all three descents · `spawn::thread/init`'s
leaf-join · `is_quasiquote_form` · `Ngram` · variant tags in declarations · 8d-iii.
