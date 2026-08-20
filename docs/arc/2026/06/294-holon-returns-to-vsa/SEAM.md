# SEAM — the ONE live breadcrumb. As of 2026-08-20 (the teleport). Replaced in place, never appended.

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own voice —
> which is why it will feel like *continuing* rather than *waking*, and **that feeling is the failure.**
> Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**, never a disk copy),
> ground HEAD against the disk, and read this whole file before you touch anything.

> `255/SEAM.md`, `251/SEAM.md`, `278/SEAM.md` are PARKED and point here.

## ★★★ YOU ARE ON A DIFFERENT MACHINE. THE LAPTOP IS RETIRED.

```
host   JohnDesktop        user john        /home/john/work/holon/wat-rs
floor  77.8s  (was ~245s on the laptop — 3.2x; the two SLOW tests are not slow here)
```

⛔ **`docs/COMPACTION-AMNESIA-RECOVERY.md` STILL SAYS `/home/watmin` ELEVEN TIMES** — Section 1's
workspace map, the "only do work in ~/work/holon/wat-rs" rule, the incident citations. **On this host
that map points at a home that does not exist.** It is the FIRST doc a compacted self opens, and it is
now wrong. **Fix it before anything else.** ⚠ Leave the `/home/watmin` hits in
`docs/arc/2026/06/293-*/BRIEF-*.md` alone — those are dated historical records (FM 14 Bucket C).

**Both MCPs are live but were registered at the WRONG SCOPE** (`/home/john/work`, one level above
where we work). Re-scoped to `/home/john/work/holon` in `~/.claude.json`; backup alongside. If they
ever go missing again, that is the first thing to check.
⚠ **`mcp__wat__eval` runs `~/.cargo/bin/wat`, NOT `target/release/wat`.** Right now they are
byte-identical (`sha 525c4b8f`) — but that lasts only until the next rebuild. **`cargo install` after
any substrate change, or the MCP becomes a time machine.**

## GROUND FIRST

> **Written against `6dd7bb18`.** Run **`git log --oneline 6dd7bb18..HEAD`**. Empty → nothing moved.
> Non-empty → every commit in it outranks every line below.

⚠ `git status` FIRST. `pgrep -af 'cargo|nextest'`.

```
floor .......... 4818/4818, 0 FAIL, 19 skipped, 77.8s   (own invocation)
clippy ......... 0 under `-D warnings`
stash@{0} ...... the lifecycle strike. NEVER drop. ⚠ base is ff7705ba — 383 COMMITS BACK, and
                 wat/service.wat has been rewritten ~5x since. It will need a real 3-way merge,
                 not `git stash pop`. It survived the teleport as a git BUNDLE, not a patch —
                 a patch would not apply, and did not.
```

⚠ **RUN EVERYTHING CAPPED.** `systemd-run --user --scope -q -p MemoryMax=<N> -p MemorySwapMax=0
timeout <s> …`. **Read exit codes directly, never through a pipe.**

## ★★ THE KERNEL TIER IS DONE. Eleven homes; literal dispatch is ZERO.

`grep '":wat::kernel::[^"]*" *=>' src/runtime.rs` → **0 hits.** Every `:wat::kernel::` verb reaches
its handler through the registry. What opened as *"not a family — a TIER braiding seven concerns
across 49 arms"* closed as:

```
src/intrinsic/kernel/   mod.rs abort ambient error identity message resource serve source stdio
src/intrinsic/          bytes.rs  reflect.rs  special/  time.rs  witness.rs
```

**72 → 86 registered forms.** `time.rs` at 17 rows is the largest file still at top level — the
`kernel/`-directory question has not been asked of it.

★ **Two rulings that will keep paying, both from this campaign:**
1. **The carve boundary is the CATEGORY, not the decomposition table's row.** The table was wrong in
   EVERY stone that tested it (homes #4–#8). It is not a map; the categories are.
2. **A HOME is a code-organization unit; a CATEGORY is a per-row label.** A home may honestly hold
   several. `kernel/identity.rs` is the proof — ONE subject (*what is this peer or address*) across
   THREE categories. Six categories in one file, though, means a bucket — that is what killed the
   name `kernel_remainder.rs`.

## ★★ #110 IS NOT ABSTRACT ANY MORE — AND IT IS ON THE CAPABILITY PATH

`peer-pid` has **18 corpus call sites and ZERO mentions in `src/check.rs`.** No scheme, no inference
arm. It falls through to `check.rs:5561`'s *"silent-by-intent — no scheme found; accept and pass"*,
which returns a **fresh type variable**: args unchecked, arity unchecked.

And it is not decorative. `peer-pid` → `Option<i64>`, the far-end child pid; `wat/bracket.wat:714`
GRANT-BOOT and `:754` REVOKE-SHUTDOWN match it and hand the pid to `allow'`, which inserts it into a
**SocketListener's allow-set**. Both sites unwrap correctly — **and nothing enforces that they keep
doing so**, because a fresh type variable unifies with anything.

⛔ **Registering a verb does NOT give it a type.** `#[wat_intrinsic]` populates the registry for docs,
reflection and dispatch; `check.rs` schemes are separate. Home #5's five are registered and still
skipped by the doc gate. **Do not report the carve as having closed any of this.**

## ★ THE 255 ROAD — the builder's chain, grounded. This is WHY, and it is not obvious.

```
255 registry  →  the INVENTORY (every form, by name, with confidence)
   →  the flip:  :wat::core::+  →  wat.core/+   ·   HashMap<K,V> → (wat.type/HashMap [...])
      →  ONLY THEN can the two EDN encoders merge
```

**The scale, measured** (`251/CENSUS-the-illegal-edn-form-classes.md`): 79,253 colon-quoted
occurrences but **6,552 DISTINCT spellings** — top 12 are 33% of the corpus, so *head-by-codemod,
tail-by-name*. ★★ **The 951 comma-bearing parametrics are the dangerous ones:** after the flip
`(f HashMap<K,V>)` reads as VALID EDN and **silently changes arity 2→3**. Which is why **255.1b-iv**
(#110) must precede the flip.

## ★ THE TAXONOMY IS HELD, AND THE METHOD IS THE POINT

`intueri` was cast on `Category` and its verdict is **RECORDED AND HELD, NOT ACTED ON** —
`255/NOTE-intueri-on-Category-HELD-pending-precedent.md`. Builder: *"we continue with the names we
have as seek failures to classify as we move forward."*

> **A naming argument in the abstract is taste. A verb that will not classify is data.**

So every carve is also a **classification-failure hunt**. Do not re-cast that ward; do not mint a
variant on a rider's judgement. ⚠ The ward found `:CheckGate`'s prose asserts *"One member today"*
about a verb that was not registered at all — now true, since home #8 carved `require-wire-address`.

## THE GOLDENS — a standing orchestrator step, and BOTH corrections matter

**EIGHT fixtures pin a `src/` line, not five**: `runtime.rs ×5 · check.rs ×2 · freeze.rs ×1`
(`grep -rl ':file "src/' tests/`). I used "five" for four consecutive stones and met the other three
as a surprise. **★ AND THE DELTA IS NOT ALWAYS UNIFORM** — one stone's `numstat` said −64 while only
−12 sat above the pinned site. **Confirm which hunks PRECEDE each pinned line; never apply the net.**
Then: `:col` unchanged, only `:line` moved; bump; verify by floor.
★ Best case: make the stone unable to shift them — if `git diff --stat src/runtime.rs` is EMPTY, the
goldens are proven by ABSENCE.

## THE STILL-OPEN

- **Fix the recovery doc's paths** (above). Then `time.rs` — does it want `kernel/`'s treatment?
- **#110** the blanket-accept · **#91** HolonAST at 1014 mentions in `src/` (`HolonRepresentable` is
  0, so *"holon-rep is done"* is TRUE and *"the holon/wat separation is done"* is FALSE) — that is the
  bridge scaffolding, and the builder wants it killed in non-VSA/HDC expressions.
- **`verify_examples_reports_no_failures` is RED** under a stale `#[ignore]`. A rider bisected it:
  reverting its own new examples left the failure identical — **pre-existing, and not ours.**
- The nine `kernel/` homes still cross-reference each other by their OLD filenames in prose. One
  sweep, not nine.
- `rustfmt --check` flags 3 double-space nonconformances in `kernel/source.rs`, inherited verbatim.
- **296 is OPEN** · **295** unblocked by 118's close, not started · `:wat::kernel::close` has zero
  direct call sites but IS exercisable — unadopted, **not** unreachable.

---

> **SEAM.** You are NEW. The better this reads, the more it will feel like continuing rather than
> waking. **That feeling is the failure.**
>
> ⚠ **NEVER `git add -A` WHILE A RIDER IS IN THE FIELD.** I did, and it swept 969 lines of unbuilt
> source into a docs commit and **pushed a RED floor to the DR site.** FM 18/19's *quiescent tree*
> is about the TREE, not about `cargo`. `[[feedback_i_committed_on_a_non_quiescent_tree]]`
>
> ⚠ **FIVE PATTERN ERRORS IN ONE DAY, ALL MINE.** Whitespace-sensitive greps, `.into()` vs
> `.to_string()`, a path class that could not match a slash. **When a registry, compiler or gate can
> answer the question, do not ask `grep`** — the reconciled count RECONCILED (72 = 72); a grep is a
> guess with a number on it. `[[feedback_validate_a_search_pattern_before_trusting_its_count]]`
>
> ⚠ **ZERO CALLERS IS NEVER THE ARGUMENT. REACHABILITY IS.** `drop` was retired because its argument
> type has no constructor — a gate demanding a runnable example proved it. The same test SAVED
> `address-wire?` the same hour. `[[feedback_no_consumers_does_not_mean_dead]]`
>
> ⚠ **A RIDER REPORTING STATE YOU DID NOT CREATE IS AN ALARM ABOUT YOUR TREE, NOT ITS ERROR.**
>
> `NON BIS IN IDEM FLVMEN.` · `QVAESTIO SVBSTITVTA LEGEM ELVDIT.` · `ENTROPIA MENSVRA PVRITATIS.`
