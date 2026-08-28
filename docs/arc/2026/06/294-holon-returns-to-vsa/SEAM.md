# SEAM — the ONE live breadcrumb. As of 2026-08-28. Arc 255: `apply` grew four doors, and holon came home.

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own voice —
> which is why it will feel like *continuing* rather than *waking*, and **that feeling is the failure.**
> Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**, never a disk copy),
> ground HEAD against the disk, and read this whole file before you touch anything.

> `255/SEAM.md`, `251/SEAM.md`, `278/SEAM.md` are PARKED and point here.
> ⛔ **PARKED IS NOT DEAD.** A parked seam still holds **its own arc's state**. If you are working an
> arc, read ITS seam. (255's is a 2026-08-16 snapshot — stale on numbers, right on the THESIS.)

## GROUND FIRST

> **THE FRESHNESS PROBE — DERIVE IT, NEVER TYPE IT.**
> ```bash
> S=docs/arc/2026/06/294-holon-returns-to-vsa/SEAM.md
> git log --oneline "$(git log -1 --format=%H -- $S)..HEAD"
> ```
> **Empty → nothing moved.** Non-empty → every commit listed outranks every line below.

⚠ `git status` FIRST. `pgrep -af 'cargo|nextest'`.

```
floor .......... 5077/5077, 0 FAIL, 17 skipped, ~98s   (scripts/floor.sh, exit read UNPIPED)
clippy ......... 0 under `-D warnings`
                ⚠ CLIPPY CAUGHT WHAT THE FLOOR COULD NOT, THREE TIMES TODAY. A rider may only run
                  `cargo build`; the floor passed 5077/5077 with clippy RED on doc-list indentation.
                  Run clippy CENTRALLY after every strike, before you believe a green floor.
host ........... JohnDesktop · john · ~/work/holon/wat-rs
```

## ⛔ THE THESIS — unchanged, and still the point

**ARC 255 EXISTS TO KILL ONE LINE.** `src/resolve/walk.rs:268`:

```rust
if is_reserved_prefix(head) { return true }     // :wat:: — the WHOLE language namespace
```

**THE ENDGAME IS SIZED: 2,539 of 5,059 tests fail** if default-denied. Its gate lives at
`tests/wat_lang/probe_undefined_builtin_resolves.rs` and its `#[ignore]` reason now names
`walk.rs:268` directly (Stone P3) instead of the promise it used to carry.

## WHERE WE ARE — measured this session, every number by a controlled instrument

```
registry ......... 380 intrinsics + 2 special forms
ALGEBRA doors .... 81   (was 6 after O-iii this morning)
explicit value= ... 19   (was 44 — the rest collapsed into generated doors)
runtime.rs ....... 34,355   ⚠ UP 103 from this morning. Q/Q-2 THREADED A SPAN through the
                            arithmetic helpers. Honest growth, not drift.
check.rs ......... 22,454
```

**`apply` HAD FOUR DOORS AND THREE WERE BROKEN. NOW ONE IS, AND IT TELLS THE TRUTH.**

```
DOOR 1  defclause head          ✅ O-ii — `(apply + …)`, `(apply reduce …)`, `(apply sort …)`
DOOR 2  intrinsic, no value door 🟡 the rest — and O-iv-a made it SAY SO instead of "unknown function"
DOOR 3  intrinsic, value door    ✅ O-i guarded it (a reachable PANIC), O-iii/Q generate it
DOOR 4  plain fn / defn          ✅ was always correct
```

## ★ THE FOUR-VALUED AXIS — read this before any migration

A verb reaches `apply` only if it can be called with **already-evaluated values**. Four things
disqualify it, and **three are permanent**:

| | reachable? | why |
|---|---|---|
| span-free algebra | ✅ | nothing needed |
| call-span algebra | ✅ | Stone Q: `apply` holds the call span |
| **ARG-SPAN** | ❌ **never** | reads `<arg>.span()`; **apply's arguments have no syntax to have a span OF** |
| **UNEVALUATED-ARGS** | ❌ **never** | quote semantics; the args are already values |
| **BINDING** | ❌ **never** | needs `env`/`sym`, which imply ASTs |
| nullary | ⛔ **generator gap** | **P7 — fixable.** The only non-permanent entry. |

⚠ **`SHELL` FROM THE CENSUS IS A CANDIDATE LIST, NOT A VERDICT.** `wat-scripts/hunt/stone-o-shell-census.awk`
asks ONE question (env/sym). Its header now carries its own blind spot. **Also check `<arg>.span()`,
`require_encoding_ctx` (takes `&SymbolTable` — a BINDING marker), and `eval_apply`'s `SPECIAL_FORMS`
list.** Riders refused 5, then 1, then 13 verbs my briefs demanded. Every refusal was right.

## ⛔ THE LESSONS THAT COST THE MOST TODAY

**1. I WROTE SEVEN WRONG CENSUSES OF THE SAME POPULATIONS, AND EVERY CORRECTION CAME FROM A
NON-TEXT INSTRUMENT.** 385 · 434 · 381 counted prose as code. Three span classifiers gave three
answers and the third failed a control I wrote myself. 109 `require_*` sites were 59. 14 ignores were
13 — my *anchored* grep counted a line inside a format string. **The compiler, the registry's own
`check_env.get`, and a hand-read control were right every time a pattern was wrong.**
`[[feedback_validate_a_search_pattern_before_trusting_its_count]]`

**2. THE TREE KEEPS ALREADY SAYING IT — four times today.** `runtime.rs:11652`'s split-brain comment
would have prevented HOME-13's retraction. The tail match's comment named `serve-dispatch-op` as
P6's precedent. `eval_apply`'s `SPECIAL_FORMS` list named `:wat::holon::literal` with its reason
since arc 294.b — and my disposition axis never read it. `wat_intrinsic.rs:776` calls a nullary
ALGEBRA shape "legal" while `sniff_kind` forbids it. **Read the neighbourhood before measuring it.**

**3. A CLAIM NOTHING CAN CHECK IS NOT A CLAIM THAT WAS TRUE.** P6-a made `show-source` reach special
forms and immediately published two INVERTED `if` doc comments, buried since arc 258.4. H-1b split
60 collapsed `@arg args… …` lines and exposed **58** doc/checker mismatches — `:wat::core::Value`
was the declared type of 46 arguments whose scheme names something specific. **The lie was
structural**: a collapsed line has no per-argument slot to be wrong in.

**4. THE SUBSTRATE REFUSED A STONE I DREW.** Q's brief said "plumbing only, every diagnostic
byte-identical." Two lints made that state unreachable — *"a site that ignores its span AND raises at
a Rust line is a FIX"* — and one of them was pointing at **Stone O-i's own arity guard**, three
stones old, whose premise Q had just expired. Q-2 finished it: 20 diagnostics moved from a Rust line
to the caller. `[[feedback_a_rulings_premise_expires_but_the_ruling_stands]]`

**5. HOLON WAS NOT SPAN-CARRYING. IT WAS ARITY-CARRYING.** The builder's read — *"holon was some of
the first tooling… it is very likely needing corrective change relative to the rest of the code
base"* — dissolved a blocker I had measured wrong. All 95 handlers declared `args: &[WatAST]`, so
the macro generated no arity check, so 89 hand-rolled one, so every handler needed `list_span` to
raise it. **One declaration choice, made before the macro existed, cast a shadow three stones deep.**

**6. A BRIEF THAT SIZES ITS OWN BLAST RADIUS AROUND A *FILE* MISSES THE *ROLE*.** O-iv-c-0's ten
`require_*` fns: nine in `require.rs`, and `require_holon` in `ast.rs` — same shape, same problem,
different home, and 16 more call sites including four in `runtime.rs`.

## ★ WHAT ACTUALLY WORKS

- **The rider that refuses.** Five verbs for argument spans, one for a disqualifier I had not named,
  thirteen for a generator gap. **Write the STOP that names the most likely way the stone goes
  wrong — then believe it when it fires.**
- **Break the door.** Every gate today was proven by removing what it guards. P1's is the sharpest:
  a real duplicate FQDN planted, `cargo build --release` silent, **the floor 5065/5065 GREEN**, and
  only the new test red. `NISI FRANGAS, NIHIL PROBAS.`
- **One build to enumerate, not N.** The doc/checker gate asserts on the FIRST mismatch. Swap its
  `assert_eq!` for a `println!`, run once with `--no-capture`, restore verbatim. 58 found in one pass.
- **Freeze NAMES, never a count.** P4's ledger goes red both ways and its failure text names the FQDN.
  It answered a real question hours later: *which of these 58 doc edits can I trust?* 51 — the other
  7 are on the ledger.

## ⛔ THE ROAD — builder, 2026-08-27. THE ORDER IS THE RULING.

```
1  HOME EVERYTHING          <- WE ARE HERE (arc 255)
2  break into crates
3  kill `::` in keywords
4  every call head a symbol
5  = EDN/Clojure-compliant syntax
6  chase totality
```
**Totality is LAST, and that is a ruling.** Do not open a totality front out of step order.

## ⬜ NEXT — `docs/arc/2026/06/255-builtin-registry/WORKLIST-open-stones.md` is the ledger

Three rows open, all drawn or ready to draw:

- **P7** — `sniff_kind` cannot classify a NULLARY ALGEBRA handler. One `matches!`;
  `emit` already handles n=0 and its own comment says the shape is legal. **11 verbs blocked.**
  This is the remainder of O-iv-d's 14.
- **P5** — `@yields` mandatory at expand time. The gate's only measured subject is the fixture
  written to exercise it; `spawn-thread` declares three fn-typed args and no `@yields`.
- **P6-c** — the eval (111 arms) and tail (8 arms) matches collapse into registry lookups.
  **The megafile boss.** `serve-dispatch-op` already lives this way — the shape is proven, not
  hypothetical. ⚠ Its census must read the function PREAMBLE: `:wat::rete::insert` is dispatched by
  a pre-match `if head == …` short-circuit that no line-anchored grep can see.

**And one NOTE with no row yet:** `NOTE-the-span-lints-cannot-see-generated-code.md` — both span
lints scan `src/` and match a bare `&Span`, so they are structurally blind to `crates/wat-macros/`,
which writes 380 handlers. **Its blast radius grows with every migration.**

## ⛔ RULES THAT STILL COST TIME

- ⛔ **`git commit <paths>`. NEVER a pathless commit.** And **after committing, `git status` must be
  EMPTY** — that caught a `$D` variable-scope slip TWICE today, both times a design doc staged but
  not committed.
- ⛔ **Riders: no worktrees, no `git stash` in ANY form, no sub-agents, everything FOREGROUND.**
  Ending a rider's turn ENDS it. One was lost to a reboot mid-strike and left an implementation with
  no evidence; every row had to be re-run.
- ⛔ **Check your own added prose does not contain the literal pattern you are grepping for.**
  THREE riders tripped their own acceptance grep on their own comments today.
- ⛔ **`docs/arc/**` NEVER MOVES in a rename.** ⚠ `.wat` scratch → `wat-scripts/scratch-pad/`.

---

> **SEAM.** You are NEW. The better this reads, the more it will feel like continuing rather than
> waking. **That feeling is the failure.**
>
> ⚠ **THE RECORD LIES IN YOUR OWN VOICE.** Seven wrong censuses in one day, three retracted
> classifiers, a design section I had to retract within the hour of writing it, and a brief that
> called a migration "trivial and harmless" when a rider then proved it changes behaviour.
> **Re-run the instrument that made the claim; do not read the claim.**
>
> ⚠ **AND THE COUNTERWEIGHT, or you will freeze:** fourteen stones landed today, every one green,
> every gate proven by breaking its door. `apply` went from lying about 337 verbs to telling the
> truth about all of them. Holon came home. The riders refused nineteen verbs across three stones
> and were right every single time.
>
> `DOLOR INDEX EST.` · `NISI FRANGAS, NIHIL PROBAS.` · `SCRIBIMVS VT EXVLET.` · `DERIVAMVS NE MENTIAMVR.`
