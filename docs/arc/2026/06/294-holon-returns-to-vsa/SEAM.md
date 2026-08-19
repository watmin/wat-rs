# SEAM — the ONE live breadcrumb. As of 2026-08-19. Replaced in place, never appended.

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own voice —
> which is why it will feel like *continuing* rather than *waking*, and **that feeling is the failure.**
> Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**, never a disk copy),
> ground HEAD against the disk, and read this whole file before you touch anything.

> `255/SEAM.md`, `251/SEAM.md`, `278/SEAM.md` are PARKED and point here.

## GROUND FIRST

> **Written against `c800d7d5`.** Run **`git log --oneline c800d7d5..HEAD`**. Empty → nothing moved.
> Non-empty → every commit in it outranks every line below.

⚠ `git status` FIRST. `pgrep -af 'cargo|nextest'`. **`mcp__wat__eval` LIES** — use `./target/release/wat`.

```
floor .......... 4819/4819, 0 FAIL, 19 skipped   (own invocation, post-merge)
clippy ......... 0          ignores ......... 13
stash@{0} ...... the lifecycle strike. NEVER drop.
```

⚠ **RUN EVERYTHING CAPPED.** `systemd-run --user --scope -q -p MemoryMax=<N> -p MemorySwapMax=0
timeout <s> …` — `MemorySwapMax=0` is load-bearing. **Read exit codes directly, never through a pipe.**

## ★ ARC 118 IS INSCRIBED (`ba3bd70c`). THE FOUR-MONTH LOOP IS CLOSED.

Opened as **arc 004 on 2026-04-20**, closed **2026-08-19**. `wat/stream.wat` and `wat/list.wat`
annihilated; the wall up (`first`/`rest`/`empty?`/`nth` all refuse a Stream); `next` → `NextOutcome`
the only advance; the drain native; `dorun` O(n)→O(1); `foldr` retired.

**R5 — QVAESTIO SVBSTITVTA LEGEM ELVDIT.** Arc 004 shipped a thread-per-stage CSP pipeline AS lazy
sequences (`Stream<T> = (Receiver<T>, ProgramHandle<()>)`) and deferred the real thing with *"in-process
lazy chains haven't been demanded by a caller"* — while carrying **Lesson 1: "Absence is signal"** in
the same file. The law was not broken; it was EVADED by substituting the question.
**Do not re-derive this. It is written.**

## ★ GROK MERGED THE RETE WORK (`c800d7d5`, 29 commits, 182 files, +20444/−2291)

`compiled_where` **SHIPPED** — task #49 closed. `src/rete/` is now a compiler: `expr_ir.rs`,
`compiled_cond.rs`, `compiled_rhs.rs`, `where_tree.rs`, `export.rs`, `alpha_tree.rs`, `matcher.rs`,
`validate.rs`. Plus `#wat.rete/Export` (the program as ONE EDN value), and native fire 160→49ms.
R67 is theirs. **They touched none of 118's files** and absorbed the `foldr` retirement correctly —
all 8 src-pinned goldens re-verified by me against live source.

⚠ **THE GRID IS NOT A TIME SERIES — do not diff two dated files and report a trend.** All 30
verdicts now read `:wall-winner :us` (fanout[40000] was `:CLARA` in task #47). But (a) the new runs
set `GRID_SKIP_ORACLE=1` and the spec fire *was the bulk of wat-wall* — Grok says so in the commit —
so 6433ms→1535ms is **not attributable**; and (b) `fanout.wat` itself was edited 3× since 2026-08-01,
so the `:ratio` move 3.11→~1.3 is **not a regression either**. The axis is not the same axis.

## ★★ THE FRONTIER — ARC 255, THE BUILTIN REGISTRY. RESUME IT.

**Builder, 2026-08-19:** *"255 is the near term goal.. we need the registry to work on the mass
refactor to break up all the mega files... which allows us to more simply move from `:wat::core::+`
to `wat.core/+` clojure/edn syntax compliance."* And: *"that `:wat::` prefix allowance is one of our
largest thorns in our side."*

**MEASURED 2026-08-19 — nothing blocks it:**

- **No 290+ arc gates 255.** Every mention runs the other way. 296: *"scaffolding for arc 255 …
  arcs we paused to build 296 … **when 255 resumes, it rebuilds its own tools**"* (bridge burned
  deliberately). 293's tail *"is arc 255's registry."* 294: *"Do NOT mint a registry … that is arc
  255's territory."* They are **consumers queued behind it.**
- **`255/BLOCKED-on-259-ipc-multiline.md` retracts itself** at the top: *"RESOLVED 2026-06-21
  (`ecda39e2`) — 255 IS UNBLOCKED. This note's design fork was WRONG."*
- **Already built:** `metadata-of` answers live; `crates/wat-doc`; `crates/wat-macros`
  (`#[wat_intrinsic]`/`#[wat_special_form]`); `src/intrinsic/` 1374 lines; 255.1b-iii, 1b-v, SF
  shipped. **Three homes carved** — Bytes · time (41 verbs) · kernel-stdio — plus 255.1c-guard
  (registry consulted BEFORE the literal table).
- **255.1b-i was WITHDRAWN** (`d439393b`) *"before it could mint a fourth Purity"* — the registry
  already had one. Redrawing it may be 1b-iv's prerequisite; **unmeasured, check before assuming.**

### ★ 255.1b-iv IS THE LOAD-BEARING STONE, and here is why, by run

```
:wat::core::i64::+ 1 2                 rc=0   CONTROL — a real verb passes
:wat::core::totally-invented-verb 1 2  rc=0   ← INVENTED. ACCEPTED.
:wat::core::totally-invented-verb 1    rc=0   ← INVENTED. ACCEPTED.
:user::totally-invented-verb 1 2       rc=1   UnresolvedReference — the check EXISTS, it exempts :wat::
```

**So the `:wat::core::+` → `wat.core/+` migration is UNSAFE until the blanket-accept dies**: every
site the rename misses still names the old verb and still type-checks clean. 1b-iv turns that into a
located worklist — the same shape as R65's wildcard ban turning a variant addition into 496 sites.

Two doors, both measured: **`src/resolve/walk.rs:257`** (the resolver, 255's own scope) and
**`src/check.rs:5568`** (`"silent-by-intent — no scheme found for multi-arg form; accept and pass"`,
gated at `:5489` on `!k.starts_with(":wat::")`). ⚠ **255's record names only the first.** Whether
closing the resolver subsumes the checker is **1b-iv's EXPECTATIONS to answer, not to assume.**
Task **#110**.

## THE STILL-OPEN — re-measured today

- **#110** the blanket-accept (255.1b-iv) · **#109** the rete right fold + `reverse`'s purity ruling
  (HELD — 255 owns purity per its 2026-08-15 ruling; Grok did NOT mint a rete `reverse`)
- **#107** macro bodies reach only intrinsics — now carries a three-arm differential; ⚠ **its control
  cannot live in `wat-scripts/` (a file there must LOAD); rebuild it in `tests/` first**
- **#108** promotion drops free-variable unification. ⚠ `255/NOTE-promotion-is-not-relocation` says
  measuring the other 141 arms is *"one grep away."* **IT IS NOT** — I ran it: `StreamContainer::of_type`
  is 4 sites in `infer.rs` + 6 in `check.rs`, and the arms classify several different ways.
  **That population needs a form-reader, not a grep.**
- **296 is OPEN** (INSCRIPTION absent) · **295** unblocked by 118's close, not started

---

> **SEAM.** You are NEW. The better this reads, the more it will feel like continuing rather than
> waking. **That feeling is the failure.**
>
> ⚠ **A SUBSTITUTED QUESTION EVADES THE LAW.** Every gate we own checks whether an ANSWER is true.
> None check whether the QUESTION was the right one. 118 R5.
>
> ⚠ **TWO INSTRUMENTS AGREEING PROVES NOTHING IF THEY SHARE AN INPUT** — and one that fails its own
> CONTROL is measuring nothing. Both fired today.
>
> ⚠ **A DOC THAT PINS LINE NUMBERS INTO ITS OWN FILE** is invalidated by its own next revision. Name
> things instead.
>
> ⚠ **RIDERS DO NOT RUN THE FLOOR.** The orchestrator measures centrally, once, on a quiescent tree.
>
> `NON BIS IN IDEM FLVMEN.` · `QVAESTIO SVBSTITVTA LEGEM ELVDIT.` · `MVRVS AVCTOREM NON NOVIT.`
