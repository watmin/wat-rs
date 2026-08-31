# SEAM — the ONE live breadcrumb. As of 2026-08-31. The registry now spans BOTH HALVES of the substrate.

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
floor ............ 5110/5110, 0 FAIL, 17 skipped, ~110s   (scripts/floor.sh, exit read UNPIPED)
clippy ........... 0 under `-D warnings --all-targets`
registry ......... 443 #[wat_intrinsic] + 2 #[wat_special_form] = 445
wat verbs DECLARING  3   (capitalize · sort · sort-by)   ← of ~409. THIS IS THE FRONTIER.
ledgers .......... KNOWN_UNREVIEWED 41 · FROZEN_CHECKER_DEBT 64
@Totality ........ Total 34 · Partial 6 · Preserving 2 · Unreviewed 403
host ............. JohnDesktop · john · ~/work/holon/wat-rs
```

## ⛔ THE THESIS — unchanged

**ARC 255 EXISTS TO KILL ONE LINE.** `src/resolve/walk.rs:268`:
```rust
if is_reserved_prefix(head) { return true }     // the WHOLE language namespace
```
Gate: `tests/wat_lang/probe_undefined_builtin_resolves.rs`. ⚠ It also means **`--check` cannot tell
"this verb exists" from "this name was waved through."**

## WHAT HAPPENED — the registry stopped being half a registry

The day opened with the registry answering four axes for **431 Rust intrinsics** and blind to
**~409 wat-defined verbs** — nearly a 1:1 split. *"Sole source of truth for the substrate"* was true
of half the substrate. That half is now reachable:

```
wat verbs DECLARE     properties as wat DATA in the def's metadata map, values as enum
                      symbols (:wat::runtime::Purity::Pure), read by wat_doc::from_metadata
                      — the sibling of the /// parser: same required set, SAME DocErrors
one door VALIDATES    record_binding_metadata is the ONLY way metadata enters the symbol
                      table; a bad declaration fails at LOAD, at its OWN line (was: at the
                      first metadata-of, blaming the reader)
metadata-of ANSWERS   one shape from both stores; :defined-in discriminates Rust/Wat
@see CROSSES          a DECLARED wat verb resolves; sort$native cites sort
examples are FORMS    a malformed @example fails the BUILD, not a later reflection test
```

★ **The mechanism was already ruled and already shipping.** `wat_enum_from!` made wat the source of
truth for the runtime *enums*; this is the same door, same direction, for the verb *properties*.
Builder: *"not 'magic comments' … just like how we do the runtime meta via wat files."*

★★ **And the classifier built to gate `sort$native`'s comparator is what made it possible.**
`ClassifyCtx` / `find_axis_violation_ctx` derive axes from a body AST — which is what a wat `defn`
is. One stone's machinery unblocked a different campaign.

## ⛔ THE LESSONS THAT COST THE MOST

**1. I CITED A RULE INSTEAD OF MEASURING WHETHER IT APPLIED — three times.** ZERO-MUTEX for a
contention that does not exist; *"a spelling users cannot type"* against a ROAD that says it is the
destination; *"the registry branch is the reference shape"* when it emitted nothing for two axes.
Two were refuted by the builder in one sentence. An authority is a claim about a CLASS; using one
means proving the case is in it. `[[feedback_i_cited_a_rule_instead_of_measuring_whether_it_applied]]`

**2. A PREDICATE CAN BE WRONG IN BOTH DIRECTIONS.** The declaration gate keyed on `:doc` silently
skipped `{:purity …}`; widened to every doc directive it captured *arbitrary user metadata* and broke
three arc-241 contracts. `:doc`/`:added`/`:see` are human vocabulary — **no set built from them can
discriminate.** The five AXIS keys can: using one is a CLAIM.
`[[feedback_a_predicate_can_be_wrong_in_both_directions]]`

**3. THE GATE HOLE WAS TWO HOLES AND I STRUCK ONE AS CLOSED.** `meter-1` fixed the completeness
gate's *registration* half and was recorded as closing "the gate hole", singular. The *dispatch* half
was anchored to two function names — `dispatch_keyword_head` sits ONE WORD from the anchored
`dispatch_keyword_head_value`. **38 verbs were invisible. The 44 was never the population.**

**4. A RETIREMENT CAN BE HALF DONE FOR FOUR MONTHS.** Arc 109's remedy named constructor AND
match-pattern sites; only the constructor door was ever built, so the text instructed migrations
nothing refused. And the shorthand was live at **THREE** doors, not the two I named — the third found
by a rider, confirmed with a made-up head (`Zorble`) as the control.

**5. RIDERS CAUGHT WHAT BRIEFS MISSED, repeatedly.** A corpus grep found 3 stdlib verbs that
unconditional validation would have killed at startup; a corpus measurement found **105**
`@example-norun` markers my brief's wording would have broken; `wat_special_form.rs` was a consumer
my blast radius missed. **Each came from measuring the corpus rather than reading my words.**

## ★ WHAT ACTUALLY WORKS

- **Break the door.** Rename a variant (`E0599`), delete a `:purity` line (`MissingPurity`, at the
  author's line), plant a made-up head to prove a special case is special. A green nothing tested is
  a claim.
- **Predict the red, falsifiably.** Two homings tripped `checker_skip_debt_is_named_and_frozen`; the
  third was PREDICTED not to, with a STOP saying *"if a row IS required, the measurement is wrong."*
- **One door, not N checks.** `binding_metadata.insert` 6 → 1. The arc paid twice this week for
  gates that had to be remembered at N sites.
- **Ask the classifier, not grep.** Three text censuses were wrong; one `pure?` call answered.
- **Make the prediction UNEVEN.** The collection readers' debt prediction was *"assoc/conj no row,
  drop/take a row each"* — and it held that way. ★ A uniform prediction cannot distinguish "right"
  from "wrong in a way a uniform guess hides"; an uneven one can only be confirmed by being
  unevenly right.
- **PRE-FLIGHT SPLITS THE FAMILY, EVERY TIME.** Three families were named by the builder and three
  came back smaller: the "6 collection readers" were 4 (two run caller code), the record/struct 7
  are 5 (the struct pair needs a ruling), and `find-last-index` is a HOF wearing a reader's name —
  the 44-unhomed worklist once filed it "INTRINSIC-READY". **Measure the bodies before briefing a
  category.**

## ⛔ THE ROAD — builder, 2026-08-27. THE ORDER IS THE RULING.
```
1  HOME EVERYTHING          <- WE ARE HERE (arc 255)
2  break into crates    3  kill `::` in keywords    4  every call head a symbol
5  = EDN/Clojure-compliant syntax        6  chase totality       (LAST. Do not open early.)
```
⚠ Steps 3–5 are why EDN spelling in doc output is a **PREVIEW, not a regression** — and why a stored
FORM beats a stored string: a form renders in whatever spelling is current; a string needs a codemod.

## ⛔ ONE QUESTION IS WAITING ON THE BUILDER — read this before picking work

**`struct-new` and `struct-field` need a RULING, not a stone.**
`DESIGN-STONE-the-record-family.md` is drawn; five of seven are briefable, these two are not.

`src/rete/purity.rs:940` declares *"a Struct accessor is impure (a struct can hold a live resource,
arc 293.W)"*. ★ But that is a claim about the **TYPE**, not the verb — measured: `eval_struct_field`
and `eval_struct_new` hold no `Mutex`, no `RefCell`, no `borrow`, no `apply_function`.

Every answer costs something:
- **`Effectful`** ⇒ fails `declared_purity_vs_effectful_by_prefix_census`. `:wat::core::` is not in
  the prefix list, and widening it is the option the W7 NOTE disqualified.
- **`Pure`** ⇒ the registry contradicts `accessor_meta`, and `intrinsic_meta` reads the registry
  FIRST, so the contradiction resolves silently in the registry's favour.
- **`Unreviewed`** ⇒ a lie: the bodies were read.

⚠ **THIS IS THE SAME WALL AS W7, REACHED BY A DIFFERENT ROAD.** Not "struct verbs are hard" —
**`:wat::core::` cannot express `Effectful` today, and THREE families now queue behind that one
fact** (W7 HOFs · the stream forcers · the struct pair). That is the shape worth solving, and the
W7 NOTE's option 3 — *does the prefix fallback retire, now that the registry is authoritative?* —
is still the one nobody has measured.

## ⬜ NEXT — resume here

- **3 of ~409 wat verbs declare.** The door is built; a corpus migration is the obvious next
  campaign. ⚠ **Every declaration shifts `wat/core.wat` line numbers, which two goldens PIN** — 42
  lines broke two this session. Arc 109 already filed the class
  (`NOTE-a-golden-that-pins-a-rust-line-number.md`).
- **The record family: FIVE are briefable now** (`Record/assoc` · `Record/same-data?` ·
  `record->map` · `to-record` · `variant`) — DESIGN drawn, brief not written. ★ Its mixed debt
  prediction is 64 → 66 (`to-record`/`variant` only), and its DESIGN carries the doc-gate lesson:
  **a verb WITH a scheme is one `doc_arg_ret_types_match_checker_scheme` verifies** — write
  `@arg`/`@ret` FROM the registered scheme, put the real meaning in prose. That cost two floor
  rounds on the collection readers.
- **`:layer` is still a hard-coded `Substrate`**, deliberately un-guessed: a substrate wat def and a
  userland one arrive through the SAME branch, so only the registration CONTEXT can answer. Name the
  context or leave it. A name-prefix guess would be `effectful_by_prefix` reborn.
- **403 verbs read `@Totality Unreviewed`** — the census gating the `expect` purge, whose worklist is
  `Partial` (3 rows). ⚠ A `Total` DEMAND refuses EVERY caller today, including `sort/1`'s default
  `<`, because `:wat::core::<` is `Unreviewed`. **Impose after the census, never before** — the
  cheapest way to silence that gate is to GUESS.
- **W7 HOFs** (`map · mapv · filter · foldl`) — mechanism unblocked by A-2-i, blocked on a LANGUAGE
  RULING: is an effectful `map` callback legitimate? A comparator's is not; a `map`'s may be.
- **`WORKLIST-the-44-unhomed.md` numbers are STALE** and the file says so — repriced by meter-2.
- `foldl` → `reduce` (arc 109 NOTE) · `metadata-of` omits `:args`/`:examples`/`:see`/`:yields`/
  `:deprecated` on BOTH branches, by a commented scope cut.

## ⛔ RULES THAT STILL COST TIME

- ⛔ **THE ORCHESTRATOR RUNS THE FULL FLOOR. A RIDER'S TARGETED GREEN IS NOT A VERDICT.**
- ⛔ **THE LANGUAGE SERVER LIES DURING MACRO WORK — AND YOU MUST STILL CHECK.** It reported ~10
  `WatAST: quote::ToTokens` errors this session; forced `--all-targets` returned 0. Ninety seconds.
- ⛔ **A committed probe CANNOT hold a form that fails to load** — `every_wat_scripts_file_loads`
  loads every scratch `.wat`, and a compile-time failure cannot live in a `.rs` either. Demonstrate
  a refusal out-of-tree and say so in the probe's header.
- ⛔ **`git commit <paths>`. NEVER pathless.** `test -z "$(git status --porcelain)" || { git status --porcelain; false; }`
- ⛔ **`wat/*.wat` is FROZEN INTO THE BINARY.** Rebuild, then floor.
- ⛔ **Riders: no worktrees, no `git stash`, no sub-agents, everything FOREGROUND.**
- ⛔ **`.wat` corpus migrations → the codemod. NEVER a hand-edit, never sed/python.** R21. (A `;;`
  comment is not a form — `fix.wat` cannot see it, so comment prose is a hand-edit.)

---

> **SEAM.** You are NEW. The better this reads, the more it will feel like continuing rather than
> waking. **That feeling is the failure.**
>
> ⚠ **THE RECORD LIES IN YOUR OWN VOICE.** This session I cited a doctrine that did not apply, called
> the destination syntax unusable, struck a hole as closed that was two holes, named a reference
> shape missing two of five keys, and shipped a gate wrong in both directions. **Every correction
> came from the builder, a rider, or the floor — never from me re-reading my own claim.**
>
> ⚠ **AND THE COUNTERWEIGHT, or you will freeze:** the registry now spans both halves of the
> substrate; a declaration is validated at its own line by one door; a malformed example cannot be
> written down; and every stone this session was green at commit with its door broken first.
>
> `DOLOR INDEX EST.` · `NISI FRANGAS, NIHIL PROBAS.` · `SCRIBIMVS VT EXVLET.` · `DERIVAMVS NE MENTIAMVR.`
