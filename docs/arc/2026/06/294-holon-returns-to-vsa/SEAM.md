# SEAM — the ONE live breadcrumb. As of 2026-09-03. **Arc 255: THE REGISTRY BECOMES THE SOLE AUTHORITY.**

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

⚠ `git status` FIRST. `pgrep -af 'cargo|nextest'`.

```
floor ............ 5129/5129, 0 FAIL, 17 skipped, ~117s   (scripts/floor.sh, exit read UNPIPED)
clippy ........... 0 under `-D warnings --all-targets`
registry rows .... 552    ⛔ COUNT IT ANCHORED TO THE ATTRIBUTE SITE, never a substring:
                          grep -rhoP '^\s*#\[wat_(special_form|intrinsic)\("\K[^"]+' src/ \
                            --include=*.rs | sort -u | wc -l
                          A loose search counts PROSE PLACEHOLDERS — `<fqdn>`, `…`, and
                          `:wat::holon::…`, which defeats a "starts with `:`" filter.
host ............. JohnDesktop · john · ~/work/holon/wat-rs
```

## ⬜ THE CAMPAIGN — read these before proposing anything

```
RULING-the-registry-is-the-sole-authority.md                the doctrine + the census
RULING-rete-forged-the-paths-the-registry-claims-the-tools.md  properties must be QUERYABLE
DESIGN-CAMPAIGN-the-registry-becomes-the-sole-authority.md   4 shapes, A picked, the SEQUENCING
WORKLIST-the-121-the-registry-cannot-vouch-for.md            re-derived 5× → 37
```
(all in `docs/arc/2026/06/255-builtin-registry/`)

## ★★★ THE METER

```
GAP_A 49 · GAP_B 42 · DEBT 121 · TYPES_UNCHECKED 10 · KNOWN_UNREVIEWED 13
the corpus: 37 names — 31 VERB POPULATION · 6 NON-VERB ARTIFACTS   (121 → 107 → 71 → 39 → 37)
failing corpus files: 130 of 615        total exposure: 638 sites   (was 1343 — a 53% cut)
```

⚠ **DEBT RISING IS NOT A REGRESSION** — a row with no `CheckEnv` scheme converts an *invisible*
absence into a *named* one. `=`/`not=` joined it this session **on purpose**: they are dispatched
by `infer_equality`, a keyword-head arm, and minting them a rank-1 `TypeScheme` would be a second
authority for a signature that function already owns.
⛔ **DEBT IS STILL TWO POPULATIONS AND ONE OF THEM IS NOT DEBT** — `Kind::SpecialForm, no scheme`
("a rank-1 scheme is the WRONG SHAPE" — a census of the un-schemeable, which should never reach 0)
vs `Kind::Intrinsic, no scheme` (genuinely owed). The finish line is **unreachable while one number
means both**, and the `Kind` split MISFILES every alias because `Kind` is stamped by the
registration VEHICLE, not the verb.

## ✅ WHAT THIS SESSION SHIPPED

```
PHASE 1a  COMPLETE      PHASE 1b  ⛔ HALF DONE — 37 of RETE_OPS' 74 rows registered, 37 REMAIN
                                  (29 of those 37 are CALLED in the corpus). MEASURED 2026-09-04;
                                  the previous SEAM said "COMPLETE" and I transcribed it forward
                                  without measuring. comm -23 <rete_names> <registry_names>.
PHASE 1c-0 · a · b · c · d · e · f · g   COMPLETE

⭐ :wat::core:: IS DONE. NOTHING IS HELD. The by-name totality placeholder is DELETED —
   `Some(Unreviewed) | None => false`, a flat default-deny with zero names.
```

**1c-f — `reduce` is a `defalias` for `foldl`.** Its 3-arity body *was* `foldl`'s, verbatim.
★ And the alias was the first consumer to read `foldl`'s **stale retained `TypeScheme`** (still
`Vector`, pre-118.B6) — which `signature-of` also reads, so **reflection had been reporting a
signature `foldl` does not have**, with three tests frozen on it. RULING item 7 failing in the
field. It also refuted `check.rs`'s *"a static TypeScheme cannot express 'any Seqable'"* note —
arc 255's **own Stone D** had already done exactly that to `:wat::string::join`, in the same file.

**1c-g — `=` and `not=` are registered `@Totality Partial`**, five compactions late. Every prior
hold rested on a prerequisite (`properties_of`, bounded generics, alias-vs-restriction); **measured:
none of them changes the grade**, because `Value` and an unconstrained type param both admit `Fn`.
`:wat::core::=` carried **695 corpus sites — the largest entry this worklist ever held.**

## ⛔ THE SURFACES ARE NOT ONE FENCE

```
wat/rete/compile.wat   where · accumulator · then-item-fence
                       pure ∧ det ∧ total ∧ RETE (Law A) — ALL FOUR, ARMED, CORRECT.
                       A generic core verb was ALREADY refused in rete. It was never allowed there.
wat/telemetry/journal.wat  sift-logs · sift-arena — THREE axes, no Law A, deliberately.
src/freeze.rs:790      the SIGMA-FN gate — a third fence entirely.
```

★ **Sift now refuses a predicate comparing a foreign `:wat::core::Value`**, and that is the fence
being correct. `ForeignRecord/get` returns `(Option Value)`; there is no `Value`→`String` coercion
in the 13-row `:wat::edn::` surface; `Value`'s declared domain admits `Fn`. The comparison is
genuinely `Partial` and **`properties_of(name, arg_types)` would answer `Partial` too** — which is
precisely why waiting for it never would have unblocked these rows. `probe_arc278_sift_arena` now
carries the `Fault` out through a `:Refused` variant and demonstrates a typed comparison sifting
fine *beside* the refused `Value` one.

## ⬜ OPEN FORKS — measured, not decided

```
★ THE NEXT PICK IS A REAL CHOICE, and the worklist's SHAPE changed this session:
    :wat::eval-ast!    1 name,  331 sites  ← 52% OF ALL REMAINING EXPOSURE, 3× the next entry
    :wat::rete::*     ~26 names, ~250 sites ← more NAMES, the old Phase-1b block
  More sites vs more names. Different jobs. Pick deliberately; do not default to the bigger list.

restore the Value-comparison capability   needs a comparable-subset type or a coercion verb.
                      No consumer waits but sift. Named, not deferred-in-prose.
alias vs RESTRICTION  the 8 blocked rete equality rows point at the GENERIC core_name. An alias
                      means IS; these are RESTRICTED TO. The registry cannot say that.
19 rows lie about arity  #[wat_intrinsic] derives Arity from the RUST SIGNATURE SHAPE; a
                      &[WatAST] param ⇒ Variadic with no shim check.
                      [[NOTE-nineteen-rows-declare-Variadic-and-enforce-a-fixed-arity]]
derive's declare ptr  role = declare names parse_derive_form; the real mutation is an inline arm.
the FOURTH registry   41 stdlib macros, 0 visible — AND every wat-defined verb. A wat-side
                      `{:totality …}` map lands in `sym.binding_metadata` (declare/register.rs),
                      NOT in `registry()`. Proven: `:wat::core::count` is a wat defalias with no
                      registry row. Only THREE wat verbs carry axis decorations at all.
DEBT is two populations  see the meter's warning. Splitting it is a prerequisite to the finish line.
```

## ⛔ WHAT COST THE MOST THIS SESSION — every one caught by a gate or the builder, none by re-reading

**1. I HANDED A RIDER A LIST INSTEAD OF AN INSTRUMENT, THREE TIMES.** Worst form: I wrote STOP-2
to catch a wrong census and **derived it from the same wrong census**, so it could not fire. I
swept 3 of the repo's **7** `.wat` roots (`wat-tests/` holds 81 files). A guard built from the
claim it guards always agrees. `[[feedback_a_stop_trigger_inherits_the_census_blind_spot]]`

**2. AND THE OPPOSITE ERROR, SAME DAY.** STOP-1 on the next stone was drawn so tight that normal
line-number drift in a preserved doc block stopped the whole stone — nearly a sixth hold on
`=`/`not=`. A guard wrong in either direction costs the same.
`[[feedback_a_guard_drawn_too_tight_makes_the_honest_path_noncompliant]]`

**3. A MIS-AIMED PROBE GAVE ME SEVEN FALSE GREENS.** The stdlib is `include_str!`ed
(`src/load/stdlib.rs`), so editing `wat/seq.wat` and running the stale binary tested **nothing**.
Caught only because an undefined verb appended to the stdlib *also* read `exit=0`.
⛔ **Every `.wat` stdlib probe needs a rebuild AND a sabotage canary.**

**4. A PATCH FIXES ONE COPY OF A CLAIM.** Three siblings in `seq.wat` still asserted the retired
two-arity shape after the adjacent block was corrected; `USER-GUIDE.md` presented two aliases as
"live" that exist nowhere on disk.

**5. MY OWN CENSUS INSTRUMENTS WERE WRONG FOUR TIMES** — a tokenizer returning impossible
"7-arity" rows; an `awk` ledger count returning 0 for all four; a grep that could not tell an
entry from a comment. **Every count was corrected by validating the instrument, never by
re-reading the number.**

## ★ WHAT ACTUALLY WORKS

- **The ledger ratchets name the exact edit.** Let them drive; never pre-compute their lists.
- **Derive every acceptance row from the rule.** Every derived row this session landed EXACTLY —
  `39 → 37`, `GAP_B 44 → 42`, `DEBT 119 → 121`, `registry 550 → 552`, all predicted before the strike.
- **Show a gate FIRING before shipping it.** The new 2-arity witness was sabotaged to 3-arity and
  went red before it was trusted.
- **A prose citation names a SYMBOL, not a LINE** — this arc's own stone, and the permanent cure
  for drifting doc blocks. 8 of 12 line citations were already false when it was measured.
- **Run the substrate as the census.** Registering honestly and reading the floor found the real
  blast radius in one run; no amount of grepping would have.

## ⛔ RULES THAT STILL COST TIME

- ⛔ **THE ORCHESTRATOR RUNS THE FULL FLOOR. A RIDER'S TARGETED GREEN IS NOT A VERDICT.**
  Give riders `binary_id(wat)` — where every ledger and registry gate lives — not a list of names.
- ⛔ **THE LSP LIES.** Run clippy; believe nothing else.
- ⛔ **`./scripts/floor.sh > /dev/null 2>&1; echo $?`** then read the Summary from `.floor/latest/raw.log`.
- ⛔ **`git commit -F`, NEVER `-m`** — backticks are shell-interpreted. **`git commit <paths>`.**
- ⛔ **REVERTING IS A LOSS.** Narrow the stone instead; preserve held work in a NOTE.
- ⛔ **DELETIONS MUST CLEAR A HIGH BAR** — *"we augment as they need."* A test of a retired
  behaviour becomes a NEGATIVE WITNESS of the retirement; it is not deleted.
- ⛔ **Riders: no worktrees, no stash, no sub-agents, everything FOREGROUND, `model: "sonnet"`.**

## ⬜ NEXT

```
1  PICK: eval-ast! (331 sites, 1 name) OR Phase 1b's rete remainder (~250 sites, ~26 names).
   The worklist is no longer dominated by :wat::core::. This is a real choice — measure both.
2  DEBT is two populations. Split it, or the finish line stays unreachable by construction.
3  Fallback's 20 (Phase 2b) · the arc-251 :wat::type:: fork
4  Phase 3a — resolve asks the registry. Kills is_reserved_prefix, THE FOUNDING TARGET.
```

---

> **SEAM.** You are NEW. The better this reads, the more it will feel like continuing rather than
> waking. **That feeling is the failure.**
>
> ⚠ **THE RECORD LIES IN YOUR OWN VOICE.** Today I wrote a STOP trigger from the same wrong census
> it was meant to catch, and the opposite error four hours later. I took seven false greens off a
> stale binary. I published three instrument-derived counts that were wrong. **Not one was caught
> by re-reading my own claim — every single one came from a gate, the floor, or the builder.**
>
> ⚠ **AND THE COUNTERWEIGHT, or you will freeze:** the corpus went **1343 sites → 638 in one
> session**, `:wat::core::` is DONE with nothing held, the placeholder that lied about three verbs
> is deleted, and reflection stopped reporting a signature `foldl` never had.
>
> `DOLOR INDEX EST.` · `NISI FRANGAS, NIHIL PROBAS.` · `SCRIBIMVS VT EXVLET.` · `DERIVAMVS NE MENTIAMVR.`
