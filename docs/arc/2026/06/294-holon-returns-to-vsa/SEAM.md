# SEAM — the ONE live breadcrumb. As of 2026-08-30. `:- [...]` is the only param-spec, and P6-c homed 42 verbs.

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
floor .......... 5079/5079, 0 FAIL, 17 skipped, ~104s   (scripts/floor.sh, exit read UNPIPED)
clippy ......... 0 under `-D warnings`. ⚠ IT WENT RED ON A GREEN FLOOR AGAIN THIS ARC
                 (arity-5 verb over the 7-arg limit). Run it CENTRALLY after every strike.
registry ....... 422 intrinsics + 2 special forms = 424 entries
                 ★ `all_entries().count()` == the ANCHORED `#[wat_intrinsic]` grep + 2, ALWAYS.
                   Three riders reported that 2 as "drift". It is `if` and `let`.
runtime.rs ..... 34,206      debt ledger 53      KNOWN_UNREVIEWED 221
host ........... JohnDesktop · john · ~/work/holon/wat-rs
```

## ⛔ THE THESIS — unchanged

**ARC 255 EXISTS TO KILL ONE LINE.** `src/resolve/walk.rs:268`:
```rust
if is_reserved_prefix(head) { return true }     // the WHOLE language namespace
```
**2,539 of 5,059 tests fail if default-denied.** Gate: `tests/wat_lang/probe_undefined_builtin_resolves.rs`.

## WHAT HAPPENED — two campaigns

### 1. P6-c: 42 verbs homed out of the megafile. Population 146 → 106.

```
W1 config 4 · W2 stream/program 4 · W3 runtime 10 · W4 runtime-remainder 3
W5a rete predicates 9 · W5b rete mutators 6 · W5c rete readers 4 · (+P6-c-1's proof pair 2)
```
Every wave: rule each verb with a disk-cited reason FIRST, then home it with its **REAL arity**
(59 hand-rolled arity guards retired), then verify by `metadata-of`. **The census tool is
`wat-scripts/hunt/p6c-disposition-census.py`** — it reads its rulings from a frozen
`DESTINATION_LEDGER` and is **DEFAULT-DENY** (builder: *"the heretics are set ablaze by their
words — they self identify"*). A ruling is a `(destination, reason)` PAIR; a nameless one is FATAL.

⚠ **HOMEABLE is NOT a progress meter** — I wrote that and was wrong. Ruling a verb then homing it
removes it from the match, so the ledger row must go and HOMEABLE returns to 0 every wave.
**The meter is the POPULATION SHRINKING.**

### 2. ONE PARAM-SPEC (arc 109) — the detour that became the bigger campaign.

Builder saw a rider write `(:wat::core::Vector :wat::core::i64 1 2 3)` and asked if it was legal.
It was — **one of THREE spellings**, and the error message recommended it. Ruling: *"there is
exactly one way to confer a parametric type. it is `:- [...]`. all others must die."*

```
1675 corpus sites (386 files, R21 codemod) · the lint AUTOFIX that WROTE it · 4 diagnostics that
RECOMMENDED it · 36 published @examples that SHOWED it · 185 prose sites that TAUGHT it ·
and every door that ACCEPTED it — value AND type position, all 6 heads, both head spellings.
20/20 probes. floor 5079/5079.
```

★ **Why it survived weeks of param-spec work:** both prior migrations were sourced from the ANGLE
form `Head<args>`. A site already written `(Vector :T …)` has no angle brackets, so no codemod ever
looked at it. **The work was complete for the question it asked.**

## ⛔ THE LESSONS THAT COST THE MOST

**1. MY GREP WAS WRONG FOUR TIMES IN ONE CAMPAIGN, EACH BY A DIFFERENT MECHANISM.**
`short 178` (blind to bare TYPE-REFERENCE position) · `over 66` (could not tell a kwargs field name
from a param-spec) · `short 2` (a line-level `grep -v` ate lines carrying BOTH forms) ·
`short 5` (head list narrower than the population). **Every one was caught by a rider I had handed
the census to.** `[[feedback_validate_a_search_pattern_before_trusting_its_count]]`

**2. SO STOP CENSUSING — IMPOSE THE CHECK AND READ THE SCREAMS.** The wall opened at **2765 reds**
and closed at 0 in three rounds; the last doors went 19 → 0 in one. The reds named work four greps
could not, including a shape NOBODY had ever seen (arc 251's `wat.type/Head` symbol-namespace form
in `<-`/`->` slots — 16 of the 19). **FM 15: the fail count is the progress meter.**
`[[feedback_impose_the_check_and_read_the_screams]]`

**3. A WALL'S OWN PAPERWORK CAN CLAIM A DOOR IT NEVER CLOSED.** `check.rs:12220` said *"③ deletes
this arm"* — **for four stones it had not**, and that arm was one of the three surviving doors. I
then declared the campaign closed on stone 3 (*"One form. No exceptions."*) while type-annotation
position and three heads were still wide open. `[[feedback_a_walls_paperwork_can_claim_a_door_it_did_not_close]]`

**4. A TEST CAN BE GREEN ON AN INPUT THE CHECKER FORBIDS.** `step_holon_constructor_bundle` passed
for years by feeding unchecked AST in a spelling no source program could produce; two more
(`list_mixed_types_rejected`, `bundle_of_list_of_ints_rejected`) kept `unwrap_err()`-ing **via a
different mechanism** once the wall moved. A rewritten rejection test silently stops testing the
rejection — and the floor stays green.

**5. NAMING A HAZARD IS NOT HANDLING IT.** I wrote "one line is a multi-line pattern's continuation"
into a NOTE and published the count anyway; it was wrong by seven.
`[[feedback_naming_a_hazard_is_not_handling_it]]`

## ★ WHAT ACTUALLY WORKS

- **Hand the census to the rider and say your number is suspect.** Four corrections came from that.
- **Break the door.** Every gate proven by removing what it guards — a planted duplicate FQDN, a
  blanked ledger reason, a renamed ledgered FQDN, a corrupted `@arg` on an untouched verb.
- **Dry-run + diff before the corpus.** It caught a codemod taking an end-span from a `~` character
  and corrupting `~selectable-peer-ty`. R21 exists for that.
- **A rider that refuses.** W6 stopped at STOP-1 with ZERO edits rather than home 7 of 8.

## ⛔ THE ROAD — builder, 2026-08-27. THE ORDER IS THE RULING.
```
1  HOME EVERYTHING          <- WE ARE HERE (arc 255, P6-c)
2  break into crates    3  kill `::` in keywords    4  every call head a symbol
5  = EDN/Clojure-compliant syntax        6  chase totality       (LAST. Do not open early.)
```

## ⬜ NEXT — resume here

- **RE-CUT W6 AS SEVEN.** `length · empty? · last · rest · nth · reverse · range`. Drawn at
  `249ccb5fd`; its rider STOPPED at STOP-1 with zero edits because **`find-last-index` is a HOF
  wearing a reader's name** (`(Vector<T>, Fn(T)->bool)`, `apply_function` per element).
- **THREE FAMILIES PARKED, each with a NOTE, none drawn:**
  - HOFs + stream forcers — `NOTE-the-prefix-guess-does-not-scale-to-a-mixed-namespace.md`.
    They're Effectful, and `Effectful ⇒ effectful_by_prefix` would force widening `:wat::core::`,
    making the guess vacuous for the biggest namespace. ★ **Ask whether that fallback should RETIRE,
    not grow — the campaign is what shrinks its job.**
  - The 9-verb rete FIRING family — `NOTE-the-firing-family-is-dual-implemented.md`. `fire-rules`
    is BOTH a wat `defn` (first-class Fn) and a Rust arm; homing adds a THIRD ahead of both.
  - The 10-verb `:wat::eval-*!` cluster — one arm peels a param-spec for all ten, and the generated
    shim's arity check would fire BEFORE the peel. Behaviour change, not a sweep.
- **Also open:** `NOTE-restricted-call-fires-on-mention-not-call.md` (option B: a per-`@arg`
  reflective marker delivering an OPAQUE handle, so holding it confers nothing) and
  `NOTE-a-type-has-two-value-representations-and-neither-is-a-type.md` (there is no `Type`; blocked
  behind ROAD step 3, which rewrites how a type is spelled).

## ⛔ RULES THAT STILL COST TIME

- ⛔ **`git commit <paths>`. NEVER pathless** — a pathless commit shipped 2 of 10 files this arc.
  **And `git status --short && echo "EMPTY"` IS NOT A CHECK** — `&&` fires the echo whenever status
  SUCCEEDS. It printed EMPTY over eight modified files. Use:
  `test -z "$(git status --porcelain)" || { git status --porcelain; false; }`
- ⛔ **NEVER `git checkout -- <file>` to clean up your own probe** — it reverts the WHOLE file and
  takes a rider's uncommitted work with it. I did this and had to resume the rider to recover.
- ⛔ **`wat/*.wat` is FROZEN INTO THE BINARY.** A `--check`-clean edit there proved nothing and broke
  97 tests. **Rebuild, then floor.**
- ⛔ **Riders: no worktrees, no `git stash` in ANY form, no sub-agents, everything FOREGROUND.**
- ⛔ **`.wat` corpus migrations → the codemod. NEVER a hand-edit, never sed/python.** R21.

---

> **SEAM.** You are NEW. The better this reads, the more it will feel like continuing rather than
> waking. **That feeling is the failure.**
>
> ⚠ **THE RECORD LIES IN YOUR OWN VOICE.** Four wrong censuses in one campaign, a "closed" campaign
> that had three doors open, a NOTE I retracted twice in an hour, and a rider's headline finding I
> had to kill with four probes. **Re-run the instrument that made the claim; do not read the claim.**
>
> ⚠ **AND THE COUNTERWEIGHT, or you will freeze:** 42 verbs homed and 1675 corpus sites migrated,
> every stone green, every gate proven by breaking its door. The riders refused work five times and
> were right every time. `:- [...]` is now the only representable param-spec, measured 20/20.
>
> `DOLOR INDEX EST.` · `NISI FRANGAS, NIHIL PROBAS.` · `SCRIBIMVS VT EXVLET.` · `DERIVAMVS NE MENTIAMVR.`
