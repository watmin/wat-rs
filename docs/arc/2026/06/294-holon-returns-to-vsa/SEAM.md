# SEAM — the ONE live breadcrumb. As of 2026-08-24. Turbofish dead · rete MERGED · scoped work SHIPPED · wat-grep is the build.

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own voice —
> which is why it will feel like *continuing* rather than *waking*, and **that feeling is the failure.**
> Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**, never a disk copy),
> ground HEAD against the disk, and read this whole file before you touch anything.

> `255/SEAM.md`, `251/SEAM.md`, `278/SEAM.md` are PARKED and point here.

## GROUND FIRST

> **THE FRESHNESS PROBE — DERIVE IT, NEVER TYPE IT.**
> ```bash
> S=docs/arc/2026/06/294-holon-returns-to-vsa/SEAM.md
> git log --oneline "$(git log -1 --format=%H -- $S)..HEAD"
> ```
> **Empty → nothing moved.** Non-empty → every commit listed outranks every line below.

⚠ `git status` FIRST. `pgrep -af 'cargo|nextest'`.

```
floor .......... 5025/5025, 0 FAIL, 19 skipped, ~82s  (own invocation, scripts/floor.sh, at fd7b017f6)
                ⚠ ACCOUNTED BY NAME, NEVER BY ARITHMETIC. 4928 → 5025 this session:
                  +88 grok-rete merge (−9 renamed/deleted, +97 theirs — the 9 ENUMERATED)
                  +3 stone D · +3 binder-universal · +3 guard-peel · +3 scoped-work
clippy ......... 0 under `-D warnings`
host ........... JohnDesktop · john · ~/work/holon/wat-rs
stash@{0} ...... the lifecycle strike. NEVER drop. base ff7705ba.
```

⚠ **RUN EVERYTHING CAPPED.** `systemd-run --user --scope -q -p MemoryMax=<N> -p MemorySwapMax=0 timeout <s> …`.
⚠ **A stdlib `.wat` edit is INVISIBLE until you rebuild** (`include_str!` at RUST-compile time).
⚠ **`cargo wat` uses the STALE installed binary.** Always `target/release/wat`.

## ⛔⛔ WHERE WE ARE — the build is wat-grep

**rete is merged and DONE** (`387662bd9`). Builder: *"rete is basically perfect… only physics is
holding it back"* and **grok-rete is AUTHORITATIVE for the rete subsystem** — resolve every rete
conflict to their side; main's walls still apply as language law, complied with by routing or an
earned rune, never by overriding rete semantics. Do NOT re-litigate their deletions.

**The build now is wat-grep**, and the builder's thesis is the reason: a corpus of proven
"encoded thoughts for problem resolution" buys rete fluency **by proximity, not by training**.
Every census error this session was a text pattern approximating a structural question; the worst
(a 404-site `String/*` shadow namespace) was invisible because *nobody knew to ask*. **A grep
answers one question and evaporates. A fact base answers questions you have not thought of yet.**

> ⛔ **THE CONTRACT, builder 2026-08-24:** *the user's rules assert `Match` facts; wat-grep queries
> for them and prints.* wat-grep owns ONE query and performs NO interpretation. The user supplies
> RULES, not queries. Everything wat-grep does not interpret is something it cannot get wrong.

### SHIPPED this session

```
78bed2e3f  stone D — join widens to Seqable (the string chain's last rung under E)
387662bd9  MERGE grok-rete — the bootstrap cycle broken from the side that could
b1ce922aa  the rete prose lands; the bridge becomes a recorded procedure
04863f99c  the call-site binder is UNIVERSAL — peel once at the dispatch cluster
5981e23cb  guard the peel point — too many type args no longer swallowed
fd7b017f6  with-network / with-overlay / Overlay — the lease finally has a shape
```

### ⬜ NEXT — wat-grep, in order

1. **THE SPAN FACT** — `278/DESIGN-STONE-the-span-fact.md`, DRAWN, ready to release. **The one
   blocking prerequisite**: `rules-corpus-03` emits `Node`/`Named` and NO coordinates, so under the
   contract no user rule can build a `Match` at all. Measured: `ast-span` and `ast-end-span` are
   **TOTAL** — which INVERTS corpus-03's guard design (`ast-name` is partial; `Named < Node`).
   **`Span == Node` is the control**; a count below Node means a guard crept in.
2. **`:wat::grep::Match`** — modelled, not drawn:
   `span <- :wat::core::Span` (the substrate's own coordinate; do NOT invent a second) ·
   `rule <- String` (set by the USER's RHS) · `bindings <- PersistentMap` (what the rule concluded —
   the field that makes it not-grep). **No `:offset`** — `fix.wat` derives it from `{:line :col}`;
   carrying both gives one position two sources of truth.
3. **wat-grep itself** — 93 lines today, TOP-LEVEL ONLY, and it already computes spans and throws
   them away (`wat-grep-form-edit`). Needs: return coordinates · walk deep (`fix.wat`'s `fix-source`
   has the recursion) · then the rete processor on top.
4. **The query is DATA, not code** — PROVEN: `load-file!` resolves at FREEZE time so a query program
   cannot be loaded as code, but `read-string` → `eval-ast!` in the frozen world → `Rule` →
   `compile` runs end to end. `eval_in_frozen`'s own doc names *"rule-like pattern-match systems"*.

### ⬜ ALSO OPEN

- **E — the string home.** Fully scoped: **21 verbs**, `:wat::core::string::` retired ENTIRELY
  (leaving it alive for 4 verbs was my error — it breaks home #4). `String/*` is a 404-site SHADOW
  namespace of pure aliases; `String/empty?` is the one real verb hiding there. `string::{=,not=}`
  join as new verbs (i64 and f64 have them; string never did). Needs the builder's go.
- **eval is DYNAMIC, not polymorphic.** arc 028 shipped `type_params: vec![]`; the `∀T` arrived at
  `a33642acf` as the RESIDUE of reverting a wrapper, and its own comment concedes
  *"trust-the-caller… type-mismatched downstream ops fail at runtime."* `ann-form` exists.
- **The untyped-PV hazard** — two sites in rete's hot path restructure around an empty
  `PersistentVector` carrying an unconstrained `T` (`oracle/pass.wat:353`).
- **The load-order gate is half-blind** — it reports a COUNT and names `probe_arc275_verify_stdlib`,
  a test target that no longer exists. Call `(:wat::deporder::verify-stdlib)` for the actual list.
- **The codemod cannot do its own job** — arc 109 walled the syntax its own migration tool reads.
  Procedure recorded: `109/NOTE-the-wall-disabled-the-codemod-that-removes-the-wall.md`.

## ⛔ THE LESSON THAT COST THE MOST TODAY

**A DESIGN IS UNFALSIFIABLE UNTIL SOMETHING CONSUMES IT.** Three designs, all grounded, all written
down, all wrong — and **all three ran GREEN first**:

```
the call-site binder   checker accepted it, runtime refused it — found by USING it elsewhere
with-network           leaked the very lease it existed to drop — found by grepping who ELSE calls arm-session
the load-order red     "wat/rete.wat, after the session records" — load order never considered
```

Each defect lived in a RELATION the design did not contain — between two doors, between a wrapper
and another caller, between a file's position and its symbols. **You cannot re-read your way to
these.** `[[feedback_a_design_is_unfalsifiable_until_something_consumes_it]]`

★ The builder's sequencing is what caught two of them: *"build what we think with-rete looks like in
wat-grep and then we'll migrate the UX story to rete."* **Prototype at the CONSUMER, then promote.**

## ⛔ RULES THAT STILL COST TIME

- ⛔ **ACCOUNT THE FLOOR BY NAME.** 4928→5016 looked clean; enumerating found **9 tests gone** —
  8 renames and 1 real deletion. A rise hides a loss.
- ⛔ **A SPAN-PINNED GOLDEN IS RECAPTURED, NEVER DROPPED.** Three times today. Verify the emitter is
  byte-identical against HEAD, THEN `UPDATE_EDN=1`; the diff must be `:line` and nothing else.
- ⛔ **DISJOINT FILES ≠ DISJOINT MEASUREMENT.** Sample `git diff --numstat` twice. I probed a live
  rider's tree again today — the finding held, but re-run it clean before crediting.
- ⛔ **A RIDER'S SUBAGENT IS OUTSIDE YOUR BRIEF.** Every brief says *"You may not spawn sub-agents."*
- ⚠ **`.wat` scratch → `wat-scripts/scratch-pad/`** — but a probe that must FAIL cannot live there
  (the loader gate requires it to load). That is `109/DESIGN-a-file-declares-its-wat-contract.md`.

---

> **SEAM.** You are NEW. The better this reads, the more it will feel like continuing rather than
> waking. **That feeling is the failure.**
>
> ⚠ **THE RECORD LIES IN YOUR OWN VOICE.** This session alone: a brief that asserted a `pub(crate)`
> I never read and wrote *"signature-checked"* over it; a stale comment that sent three riders
> hunting; a doc citing a call site that does not exist; four censuses under with validated
> instruments and controls; and a gate that prints a tally and points at a dead command.
> **Re-run the instrument that made the claim; do not read the claim.**
>
> ⚠ **AND THE COUNTERWEIGHT, or you will freeze:** the turbofish is unwritable, unmintable,
> unrenderable, unparseable and untaught; rete came home; the lease has a shape. Every one came from
> imposing a check and reading the screams — and where no check could be imposed, from writing the
> exile down.
>
> Read `294/REALIZATIONS.md` **R5 → R6 → R7 → R8**: `AEQUALITATEM RESPUO` (the shell, not the point) ·
> `DOLOR INDEX EST` (the ache is the instrument) · `INCENDIMVS VT VIDEAMVS` (light it yourself) ·
> `SCRIBIMVS VT EXVLET` (write so it stays exiled).
>
> `NON BIS IN IDEM FLVMEN.` · `IVDICIVM SEMEL, MACHINA SAEPE.` · `NISI FRANGAS, NIHIL PROBAS.` ·
> `INCENDIMVS VT VIDEAMVS.` · `SCRIBIMVS VT EXVLET.`
