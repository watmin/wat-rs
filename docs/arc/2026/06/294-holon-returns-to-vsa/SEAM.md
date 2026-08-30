# SEAM — the ONE live breadcrumb. As of 2026-08-30. The registry answers FOUR axes; the hand-lists are backlogs.

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own voice —
> which is why it will feel like *continuing* rather than *waking*, and **that feeling is the failure.**
> Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**, never a disk copy),
> ground HEAD against the disk, and read this whole file before you touch anything.

> `255/SEAM.md`, `251/SEAM.md`, `278/SEAM.md` are PARKED and point here.
> ⛔ **PARKED IS NOT DEAD.** A parked seam still holds **its own arc's state**.

## GROUND FIRST

> **THE FRESHNESS PROBE — DERIVE IT, NEVER TYPE IT.**
> ```bash
> S=docs/arc/2026/06/294-holon-returns-to-vsa/SEAM.md
> git log --oneline "$(git log -1 --format=%H -- $S)..HEAD"
> ```
> **Empty → nothing moved.** Non-empty → every commit listed outranks every line below.

⚠ `git status` FIRST. `pgrep -af 'cargo|nextest'`.

```
floor ........ 5109/5109, 0 FAIL, 17 skipped, ~106s   (scripts/floor.sh, exit read UNPIPED)
clippy ....... 0 under `-D warnings`
registry ..... 429 #[wat_intrinsic] + 4 #[wat_special_form]
runtime.rs ... 33,917      KNOWN_UNREVIEWED 50      debt ledger 55
@Total ....... Total 25 · Partial 1 · Preserving 2 · Unreviewed 403
@ExpandTime .. Legal 143 · RuntimeOnly 0 · Preserving 0 · Unreviewed 288
host ......... JohnDesktop · john · ~/work/holon/wat-rs
```

## ⛔ THE THESIS — unchanged

**ARC 255 EXISTS TO KILL ONE LINE.** `src/resolve/walk.rs:268`:
```rust
if is_reserved_prefix(head) { return true }     // the WHOLE language namespace
```
**2,539 of 5,059 tests failed if default-denied — measured 2026-08-26, STALE by this whole day.**
Gate: `tests/wat_lang/probe_undefined_builtin_resolves.rs`.
⚠ It also means **`--check` cannot tell "this verb exists" from "this name was waved through."** I
used it as an existence probe today and it proved nothing.

## WHAT HAPPENED — the registry became the source of truth for FOUR axes

```
purity          ✅ derives   T5     hand-list gone
determinism     ✅ derives   T5     hand-list gone
totality        ✅ derives   T1→T4b residue: 11 unhomed verbs
expand-time     ✅ derives   T1→T4b residue: 59 unhomed verbs
```

Two axes were **minted from nothing** this day — `Totality` and `ExpandTime`, both as
`defenum`s in `wat/runtime-meta.wat` with the Rust generated from them, both proven by **renaming a
variant in the `.wat` and watching Rust fail with `E0599`**. Each then went declarable → required →
derived. `intrinsic_meta` fell 2565 → 2369 lines; `is_expand_time_legal`'s 202 names became 59.

★ **Every axis converges on the same residue: verbs with no home.** The homing campaign and the
property campaign are one campaign. `WORKLIST-the-44-unhomed.md` and
`WORKLIST-the-registry-properties.md` are both on disk and are the agenda.

## ⛔ THE LESSONS THAT COST THE MOST

**1. `Unreviewed` AS A FOURTH VARIANT IS WHAT MADE EVERYTHING ELSE SAFE.** Both new axes got
`Total|Partial|Preserving|Unreviewed` rather than a bare pole pair. That choice — an unmeasured verb
is **default-deny, not a guess** — is why T5 could move **275 verdicts with `ALSO_TOTAL=0`** and admit
nothing new. A design decision paying out three stones later.

**2. A CONTAINMENT ARGUMENT MUST NAME WHICH CONSUMERS IT COVERS.** I proved `ALSO_TOTAL=0` and called
T5 contained. True — for the four-axis `where` fence. `:wat::rete::deterministic?` is a **standalone
single-axis predicate**, and two floor tests call it. Mine named no consumers, so it read as general
when it was specific. `[[feedback_a_pass_answers_only_the_question_the_instrument_asks]]`

**3. A PATCH THAT FIXES ONE COPY OF A CLAIM HAS FIXED ONE COPY.** expand-1 removed `keys`/`values`,
retracted it, and I corrected the header's bullet while **missing a second comment** at the removal
site still asserting the removal. Found by a rider that refused to transcribe a blessing the code
gave and a comment denied. That is the defect class expand-1 EXISTED to audit.
`[[feedback_a_walls_paperwork_can_claim_a_door_it_did_not_close]]`

**4. A CO-LOCATED RUNE IS ATTACHED TO A LINE — MOVING THE LINE DROPS THE EXEMPTION.** `sort'`'s
earned `rune:lint(retired-name)` was left behind when its arm relocated. Not a new offender: a lost
exemption, and the lint was the only thing that would notice.

**5. MY ACCEPTANCE FILTERS WERE TOO NARROW THREE TIMES.** `-p wat-doc -p wat-macros` cannot see
`tests/` (it belongs to the `wat` package); a `macro+stdlib+intrinsic+reflection` filter cannot see
`test(lint)`. **Riders came back green and the floor came back red.** Only the orchestrator's full
floor is a verdict.

## ★ WHAT ACTUALLY WORKS

- **Break the door, every time.** Rename a variant in the `.wat` (`E0599`), hard-code a token
  (`left: Unreviewed, right: Legal`), delete one ledger row, point a delegate at the wrong impl
  (~230 reds naming `defrecord`). Nine stones, nine proofs that the green meant something.
- **Derive the set; hand the rider a PREDICTION, never a list.** "Delete iff `lookup_entry` is
  `Some`." My counts were wrong repeatedly; the registry's never was.
- **The compiler as census.** Layer 1's no-`Default` rule turns a mandate into compiler output.
  Delete one directive, rebuild, and the error NAMES the verb.
- **Riders that refuse.** Six correct refusals today, four caused by my briefs.

## ⛔ THE ROAD — builder, 2026-08-27. THE ORDER IS THE RULING.
```
1  HOME EVERYTHING          <- WE ARE HERE (arc 255)
2  break into crates    3  kill `::` in keywords    4  every call head a symbol
5  = EDN/Clojure-compliant syntax        6  chase totality       (LAST. Do not open early.)
```

## ⬜ NEXT — resume here

- **THE NAMING NOTE IS UNWRITTEN AND THE BUILDER ASKED FOR IT.** Convention: **`name$native`** for a
  native impl, **`name$oracle`** for the wat spec — replacing the `'` suffix. Measured: 5 live
  `$native`/`$oracle` pairs (the whole `:wat::rete::` firing family, e.g. `runtime.rs:5617` dispatches
  `"fire-rules$native" | "fire-rules"` on one arm), against **25 surviving primes** and 6 `-spec`.
  ★ **This DISSOLVES the W8 blocker** — I filed the firing family as "dual-implemented, homing adds a
  THIRD"; the convention already separates native from oracle. And `sort'` is a straggler that should
  be `sort$native`, which retires the rune that broke the floor today.
- **`foldl` should be `reduce`** — `NOTE-foldl-should-have-been-reduce.md` (arc 109). 572 corpus calls
  to the primitive vs 38 to the Clojure surface. ★ `sort'`/`sort` is the SAME shape done RIGHT: its
  primitive wears a suffix meaning "primitive". `foldl` wears a name you would call.
- **The 174-verb expand-time gap.** 288 verbs read `@ExpandTime Unreviewed`; ~174 are pure ∧
  deterministic and probably legal. No longer invisible — it is in the source, at each verb.
- **`WORKLIST-the-44-unhomed.md`** — 34 homing · 5 mechanism-unknown (`+ - * / reduce`, live but
  reached by an unidentified path) · 3 namespace rules. ⛔ The 5 are NOT homing work until the
  mechanism is named.
- **`WORKLIST-the-registry-properties.md`** — `expand_time_legal` ✅ done; `defined_in`/`layer`
  ⛔ **DO NOT BUILD** (they would be constant across all 433 entries); `primitive?` is an open
  question, possibly rete's to own.

## ⛔ RULES THAT STILL COST TIME

- ⛔ **THE ORCHESTRATOR RUNS THE FULL FLOOR. A RIDER'S TARGETED GREEN IS NOT A VERDICT.**
- ⛔ **THE LANGUAGE SERVER LIES DURING MACRO WORK — AND YOU MUST STILL CHECK.** Twice today it
  reported ~200 `E0560`s and 5 `MissingExpandTime`s against stale expansions; both times a forced
  `cargo check --release --all-targets` returned 0. *"It's just the analyzer"* is a dismissal; the
  forced rebuild is a measurement. Ninety seconds, every time.
- ⛔ **`git commit <paths>`. NEVER pathless.** And `git status --short && echo EMPTY` is NOT a check.
  Use `test -z "$(git status --porcelain)" || { git status --porcelain; false; }`
- ⛔ **NEVER `git checkout -- <file>` to clean up your own probe** — it takes a rider's work with it.
- ⛔ **`wat/*.wat` is FROZEN INTO THE BINARY.** Rebuild, then floor.
- ⛔ **Riders: no worktrees, no `git stash`, no sub-agents, everything FOREGROUND.**
- ⛔ **`.wat` corpus migrations → the codemod. NEVER a hand-edit, never sed/python.** R21.

---

> **SEAM.** You are NEW. The better this reads, the more it will feel like continuing rather than
> waking. **That feeling is the failure.**
>
> ⚠ **THE RECORD LIES IN YOUR OWN VOICE.** Today I published a megafile ceiling that was too
> generous by 2×, a containment argument that covered one consumer of two, a comment claiming a
> removal I had retracted, and roughly a dozen wrong counts. **Every correction came from a rider,
> a wall, or the compiler — never from me re-reading my own command.**
>
> ⚠ **AND THE COUNTERWEIGHT, or you will freeze:** two axes minted, made mandatory, and derived in
> one day; four properties now answered by the registry; two hand-lists reduced to named backlogs;
> every stone green at commit and every gate proven by breaking its door.
>
> `DOLOR INDEX EST.` · `NISI FRANGAS, NIHIL PROBAS.` · `SCRIBIMVS VT EXVLET.` · `DERIVAMVS NE MENTIAMVR.`
