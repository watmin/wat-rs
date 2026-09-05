# SEAM — the ONE live breadcrumb. As of 2026-09-04. **Two campaigns: holon is WALLED; docs are NOT.**

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own
> voice — which is why it will feel like *continuing* rather than *waking*, and **that feeling is
> the failure.** Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**,
> never a disk copy), ground HEAD against the disk, and read this whole file before you touch
> anything.

> `251/SEAM.md`, `278/SEAM.md` are PARKED. ⛔ **PARKED IS NOT DEAD.**

## GROUND FIRST

> **THE FRESHNESS PROBE — DERIVE IT, NEVER TYPE IT.**
> ```bash
> S=docs/arc/2026/06/294-holon-returns-to-vsa/SEAM.md
> git log --oneline "$(git log -1 --format=%H -- $S)..HEAD"
> ```
> **Empty → nothing moved.** Non-empty → every commit listed outranks every line below.
> ⚠ A PASSING PROBE PROVES NOTHING ABOUT TRUTH. On 2026-09-04 it came back empty while the
> GROUND block below carried a registry count its own printed command refuted. **Re-run the
> commands, do not read the numbers.**

⚠ `git status` FIRST. `pgrep -af 'cargo|nextest'`.

```
floor ............ 5139/5139, 0 FAIL, 17 skipped, ~120s   (scripts/floor.sh, exit read UNPIPED)
                   ⭐ AND doctests — armed 2026-09-04, runs FIRST, was NEVER running before
clippy ........... 0 under `-D warnings --all-targets`
registry rows .... 571 · 85 SpecialForm · 52 alias    ⛔ ASK, never grep:
                   ./target/release/wat wat-scripts/scratch-pad/255-registry-census.wat
host ............. JohnDesktop · john · ~/work/holon/wat-rs
```

## ⬜ TWO LIVE EFFORTS — read the right entry point, not this summary

```
⛔ BLOCKED, and blocked on DECISIONS not labour
   [[RESUME-the-registry-is-blocked-on-three-named-decisions]]   (255/)
   ★ THIS IS THE REGISTRY'S ENTRY POINT. Three decisions, each measured, each with its
     evidence and its re-derivation command on disk. Nothing needs re-deriving.

⬜ ACTIVE — expected to span several compactions
   255-builtin-registry/the-walls-must-not-be-muted/
     RULING-a-wall-that-cannot-run-is-not-a-wall.md     the doctrine + the 5-step order
     DESIGN-the-tagged-edn-doc-row.md                   #wat.doc/Row · #wat.doc/Alias
```

## ★★★ THE TWO SENTENCES THIS SESSION TURNED ON — the builder's own

> *"the registry is forcing the discovery of bad practices."*
>
> *"our mitigations and walls must not be muted."*

Both name the same mechanism from two sides. **A crutch survives indefinitely while nothing asks
it a question.** `HolonAST` carried syntax for six months; a public doc example taught an
impossible construction through two API changes; `@alias`+axis was a rule no shape enforced. Each
one was found the moment an instrument was pointed at it, and not before.

## ✅ WHAT SHIPPED — 2026-09-04

```
HOLON — the assault is FINISHED and the ground is WALLED
  the CEK stepper stops speaking holon. eval-step! returned values it never received:
    (quote 1/2) -> "1/2" StringLit · (+ 1/2 1/3) -> StepNext FOREVER · (fn [x] x) -> (Atom (fn (x) x))
  the special-form signature sketch is a WatAST::List — syntax stopped being a hypervector
  value_to_holon DELETED — a second HolonAST-from-Value builder its own file forbade
  ⭐ tests/lint/holon_is_vsa_only.rs — ARMED AT ZERO, sabotage-proven TWICE, and it STATES
     its own three blind spots in its module doc

DOCS — the gate that never ran
  ⭐ scripts/floor.sh runs `cargo test --doc` FIRST and UNCONDITIONALLY. It had NEVER run.
     First run, on a 5139/5139 green tree: 3 RED. One was a public example constructing
     RuntimeError by struct literal — a shape no external caller can use, both fields private,
     stale through TWO API changes.

REGISTRY — 15 rows in, and three STOPs that were all defects in MY design
  571 rows · 52 alias. The alias-vs-RESTRICTION fork CLOSED: rete's totality demand is
  CONTEXTUAL (compile-condition, for a `where`), so an alias inherits and the fence stays a
  separate, correct authority.
```

## ⛔ WHAT COST THE MOST — and NOT ONE was caught by re-reading my own claim

**1. I REPEATED STONE 2a's EXACT ERROR, against a ★★★ warning naming it.**
`rete_alias.rs`'s first 36 lines say `OpClass::Fallback` rows may **never** be aliased, and record
that it *"broke eight live rete tests when Stone 2a's DESIGN named `:wat::rete::i64::+`"*. My design
named `:wat::rete::i64::+`. **My census asked "is the core_name registered?" and never asked "is
this row ALIASABLE?"** — 35 clear was 15. `[[feedback_a_census_predicate_can_name_the_wrong_act]]`

**2. I ASSERTED AN ABSENCE I NEVER PROBED.** I told the builder that *"`@alias` + an axis does not
compile"* was rung 3 and did not exist. It exists — `DocError::AliasDeclaresAxis`, a real
`compile_error!`. The rider settled it by writing the illegal row and watching cargo refuse.

**3. MY OWN CENSUS PATTERN INVENTED 33 PHANTOM OFFENDERS.** `: *HolonAST` matched the `::HolonAST`
inside the *string* `":wat::holon::HolonAST"`. Validated line-by-line, the real count was 26 lines
of which 10 were misuse. `[[feedback_validate_a_search_pattern_before_trusting_its_count]]`

**4. I READ A COMMENT AS A RATIONALE.** The stepper's `HolonAST::Atom` wrap was justified as *"so
cosine / hash / cache keys see it as a single coordinate"* and I filed it as a VSA question for the
builder. `Atom` is `HolonAST -> HolonAST` — the algebra's quote. One grep away.

**5. I BRIEFED A GATE BY THE WRONG NAME** (`alias_axes_follow_their_target`; it is
`..._are_resolved_from_their_target`) **and prescribed axes that a real gate makes impossible**
(`cond` with `@Purity Preserving` demands `role=check`/`role=eval` impls a `defmacro` has not).

## ★ WHAT ACTUALLY WORKS

- **ASK THE SUBSTRATE, DO NOT GREP IT.** Every census this session that was wrong was a grep;
  every correction came from the compiler, a gate, a probe, or `(:wat::intrinsic::rows)`.
- **HAND A RIDER THE INSTRUMENT, NEVER THE RESULT.** STOP-2 ("the wall's census is the wall's own —
  if it finds an offender outside my four, STOP") held under real pressure: the lint found SIX, and
  the rider fixed its own detector rather than widening the allowlist.
- **SHOW A GATE FIRING BEFORE TRUSTING IT.** Both walls armed this session were sabotaged first —
  the holon wall twice, independently. `NISI FRANGAS, NIHIL PROBAS`.
- **A WALL MUST STATE WHAT IT CANNOT SEE.** `holon_is_vsa_only` names three blind spots in its own
  doc, before anything made it fail.
- **DERIVE EVERY ACCEPTANCE ROW FROM THE RULE.** Floor 5129 → 5139 landed exactly: the wall plus
  its 9 detector tests, predicted before the run.

## ⛔ RULES THAT STILL COST TIME

- ⛔ **THE ORCHESTRATOR RUNS THE FULL FLOOR. A RIDER'S TARGETED GREEN IS NOT A VERDICT.**
- ⛔ **THE LSP LIES.** It reported two E0308s on a tree `cargo build --release` compiled clean.
- ⛔ **`./scripts/floor.sh > /dev/null 2>&1; echo $?`** then read the Summary from `.floor/latest/`.
- ⛔ **`git commit -F`, NEVER `-m`** — backticks are shell-interpreted. **`git commit <paths>`.**
- ⛔ **REVERTING IS A LOSS.** Narrow the stone; preserve held work in a NOTE.
- ⛔ **DELETIONS MUST CLEAR A HIGH BAR** — *"we augment as they need."* A test of a retired
  behaviour is a NEGATIVE WITNESS of the retirement; it is not deleted.
- ⛔ **Riders: no worktrees, no stash, no sub-agents, everything FOREGROUND, `model: "sonnet"`,
  and they do NOT run the floor.**

## ⬜ NEXT

```
THE DOC-COMMENT CAMPAIGN — active, expect several compactions
  4  the #[ignore] census: 12 attributes, most naming a follow-up stone. WHICH HAVE LANDED?
     An exemption must earn its standing AS IT AGES. Plus: only 8 doctests are COLLECTED of 64
     bare fences — the rest sit on private items and can NEVER run. Both are muted walls.
  5  the migration to #wat.doc/Row · #wat.doc/Alias.
     ⛔ TWO PROBES GATE EVERYTHING, and neither is measured:
        (a) can `wat-edn` be a proc-macro dependency? (parse EDN at expand time)
        (b) does an ```edn fence survive the now-armed doctest gate?
     Then: the ratchet freezing @-form names DAY ONE (not a drop at the end), and a migration
     tool that reuses the proc-macro's OWN @-parser so the transform is faithful by construction.
     Design cold first: @example x459 + @example-norun x139 hold wat source with quotes and #=>.

THE REGISTRY — read [[RESUME-the-registry-is-blocked-on-three-named-decisions]]. Do not re-derive.

UNBLOCKED, pick by value: Phase 3b (432/432) · the DEBT split · the 270 grading batch ·
  the :None codemod (94 sites, dry-run PROVEN, no design left) · the SIX non-verb artifacts.
```

---

> **SEAM.** You are NEW. The better this reads, the more it will feel like continuing rather than
> waking. **That feeling is the failure.**
>
> ⚠ **THE RECORD LIES IN YOUR OWN VOICE.** Today the freshness probe came back EMPTY and the file
> was still wrong — the GROUND block stated a registry count that the command printed directly
> beneath it refuted. A probe asks *"did a commit land after this file?"*, never *"is this file
> true?"* **Re-run the commands. Do not read the numbers.**
>
> ⚠ **AND THE SHARPER ONE: I WAS WRONG FIVE TIMES TODAY AND EVERY CORRECTION CAME FROM OUTSIDE.**
> A ★★★ warning in the first 36 lines of a file I had just cited. An absence I asserted without
> probing. A regex that invented 33 offenders. A comment I read as a rationale. A gate I briefed by
> the wrong name. The builder caught two, riders caught two, the compiler caught one. **Zero were
> caught by re-reading my own claim** — which is the whole argument for asking an instrument.
>
> ⚠ **AND THE COUNTERWEIGHT, or you will freeze:** holon is walled at zero and the wall has been
> broken on purpose twice. The doctest gate is armed and caught a lying public doc on its first
> run. The registry can be asked about itself. Corpus 1343 sites → 638. `:wat::core::` is DONE.
>
> `DOLOR INDEX EST.` · `NISI FRANGAS, NIHIL PROBAS.` · `SCRIBIMVS VT EXVLET.` · `DERIVAMVS NE MENTIAMVR.`
