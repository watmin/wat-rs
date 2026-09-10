# SCORE — Phase 1 (`tests/`, 32 files): applied for real, floor went RED, reverted — a check-contract gap

Executing `DESIGN.md`'s Phase 1 only. Written as-I-go per house rule. Branch `grok-rete`, floor
baseline `5490/5490`. **Current state: reverted to that baseline.** The corpus is untouched; only
the codemod (`wat-scripts/fixes/hoist-where-into-condition.wat`) and this SCORE.md are modified.

**Three passes recorded below.** (1) The original dry-run-only submission (everything through the
first "What I did NOT do"): diff read, nothing applied. (2) The coordinator approved it on
substance, asked for a whitespace refinement, then asked for Phase 1 to be applied for real — see
"Coordinator's refinement" onward: the fix, a re-run dry-run diff, the real application, re-
verification, idempotency on the real corpus. (3) **The floor after applying went RED — 24
failures** — see "⛔ Floor — RED" at the end: a genuine gap in the ORIGINAL check's own "could
legally move" contract (not a codemod bug), captured in full, corpus reverted per the exact
precedent this strike already set once, nothing committed.

## Re-derivation, before any edit

Read in order: `DESIGN.md` (this strike), `wat/fix.wat` in full including the BOOTSTRAP /
STASH-DANCE note at `:23-53`, `strike-where-fence-hoistable/SCORE.md` (the census + the check's own
semantics), and two existing fixes as shape models
(`rename-record-def-to-defrecord.wat`, `positional-to-kwargs.wat`).

`git log --oneline` confirmed the check (`f47a9fccc`) is **not** on this branch's history — it was
authored, then the very next commit (`23625037b`, "NOT SHIPPED") rebuilt on the check's OWN parent
(`85ca63ea1`) without it, so `f47a9fccc` is a dangling-but-intact commit, exactly as DESIGN's table
says ("recoverable, intact"). `git cherry-pick -n f47a9fccc` applies cleanly onto current HEAD
(`7341d677a`) with zero conflicts — the two follow-on commits (`23625037b` SCORE.md-only,
`7341d677a` DESIGN.md-only) never touched `src/rete/validate/{mod,error}.rs`.

## The stash dance, run for real, twice (enumeration, then verification)

Per `wat/fix.wat:23-53` and this strike's own instruction ("restore it in a scratch build to
enumerate the sites and read its `rewrite` field"):

1. `git cherry-pick -n f47a9fccc` — check applied, uncommitted (9 files: `error.rs`, `mod.rs`, 6
   `.wat` fixtures + 1 `.rs` + 2 `.edn` goldens the check's own authoring strike added).
2. `cargo build --release` — NEW checker, used ONLY to enumerate real sites (below).
3. `git stash push -m "check" src/rete/validate/mod.rs src/rete/validate/error.rs <the 9 files>` —
   check stashed away.
4. `cargo build --release` — OLD checker restored, matching HEAD. Codemod authored and dry-run
   against THIS binary (the codemod's own execution — `read-string`/`write-file` — never invokes
   `validate_rete_rules`, so which checker is baked in cannot affect the dry-run's correctness; the
   dance matters for what a REBUILD bakes into `wat/*.wat`'s frozen stdlib, and for keeping the real
   corpus loadable while iterating).
5. Verification (see "Check-enabled re-verification" below) needed the NEW checker again: popped
   the stash, `cargo build --release`, verified, then **stashed the check away again and dropped the
   stash** (not popped) — Phase 1 does not ship the check; that is Phase 3. Final `cargo build
   --release` restored the OLD checker, confirmed by `grep -n "Where(_)" src/rete/validate/mod.rs`
   showing all four original `{}` arms, matching HEAD byte-for-byte.

## Population, re-derived independently — 32 files matches, 63 sites does NOT match DESIGN's 75

Ran the NEW-checker binary against every `tests/**/*.wat` file containing a `:wat::rete::where`
hit (`grep -rl`, 40 files — 36 real corpus + 4 negative-control fixtures the check's own authoring
strike added, `probe_where_fence_hoistable_{or,exists,accumulate,join}.wat`, correctly excluded from
the census since they didn't exist when `DESIGN.md`'s table was written and are the check's OWN
fixtures, not corpus). Classified each by the compiled validator's actual verdict, not by reading
ASTs — `WhereHoistable` in stderr → refusable:

| | this measurement | DESIGN.md |
|---|---|---|
| files w/ `:wat::rete::where` hit (36 real + 4 check-fixtures) | 40 | — |
| **REFUSABLE files** | **32** | **32** (matches exactly) |
| clean files (hit, not hoistable) | 4 | (implied 4) |
| **REFUSABLE sites** | **63** | **75** |

File count matches DESIGN's exactly. Site count does not. Counted sites via the same anchor the
prior strike's SCORE used (`WhereHoistable {:rule`, which appears once per REAL structured error —
the error's own `:message` field embeds a second, differently-shaped copy of itself,
`WhereHoistable {:message`, which this anchor does not match), cross-checked two ways before
trusting it:
- One known file (`tests/services/probe_arc278_sift_rules_arena.wat`) was checked by hand: 11 raw
  `:wat::rete::where` occurrences in the source, 11 `WhereHoistable {:rule` anchors in the compiled
  output, 0 excluded — anchor and raw count agree exactly where every `where` in a file IS
  hoistable, which is the calibration case.
- For every one of the 32 refusable files, compared raw `:wat::rete::where` hit-count against the
  anchor count; exactly 4 files diverge, and each divergence has a NAMED, checked reason, not an
  unexplained gap: `tests/rete/datamancer.src.wat` (11 raw / 10 flagged — the excluded one is a
  genuine cross-condition join, `(:wat::rete::core::i64::< ?g ?t)` reading `?g` from `:dm::Gap` and
  `?t` from `:dm::Beat`, two different conditions, exactly the shape `:where` exists for);
  `tests/rete/probe_arc278_derived_exists_acc.wat` (2/1 — the excluded one reads an accumulate's
  result var, `?n` from `(?n <- (:wat::rete::acc::count) :from …)`); `probe_arc278_where_is_positionally_free.wat`
  (5/4 — the file's own header COMMENT contains the literal text `` (:wat::rete::where …) `` as
  prose, a `grep` hit that is not a code site at all);
  `probe_arc278_vsa_where_native_differential.wat` (5/2 — the excluded 3 each read TWO vars,
  `?obs`/`?cobs`, split across the file's TWO conditions: `?cobs` bound by `:vsa::Catalog`, `?obs`
  bound by `:vsa::Observation` — three more genuine cross-condition joins, the same shape as
  `datamancer.src.wat`'s). Every one of the 4 divergences resolves to one of the same two causes
  already named in the check-authoring SCORE (a genuine join split across conditions, or an
  accumulate-var/comment false hit) — no new exclusion shape turned up. Spot-checked the 3 largest
  files by hand (11, 10, 5 sites) against their raw source; all matched the anchor count.

I did **not** find any evidence for 75 in the current tree, the same conclusion the prior SCORE.md
reached about DESIGN's `wat-scripts/` figure (71 claimed vs 38 measured, traced to a bad `rg` glob).
Reporting my own re-measurement, not DESIGN's, per that file's own instruction not to reuse a figure
without checking it.

## The codemod — `wat-scripts/fixes/hoist-where-into-condition.wat`

A from-scratch reimplementation of `check_where_hoistable`/`hoist_scope`
(`src/rete/validate/mod.rs`, recoverable at `f47a9fccc`) in wat, since a codemod cannot call into
the Rust checker — it must independently answer the same question the check answers, from the same
AST, and DELETE+APPEND instead of merely refusing.

**Shape classification** (`top-shape-tag`) mirrors `classify_rete_clause`'s top-level dispatch at
exactly the subset needed: keyword head `:wat::rete::where/not/exists/or/and` → that tag; any other
keyword head → `"plain"`; symbol head `?v <- <keyword with "::">` → `"factbind"`; symbol head
`?v <- <anything> :from <anything>` (5 items) → `"accumulate"`. A condition's own bound vars
(`bound-vars-of-plain`/`-factbind`) are its fact-bind var (factbind only) plus every inline
`(?v <- :field)` clause — never "every symbol that looks like a var".

**Collection is transparent through `:and` ONLY** (`collect-hoist-targets`,
`collect-where-sites`) — `:or`/`:not`/`:exists`/an accumulate are never descended into for either
hoist TARGETS or hoistable `where` SITES, mirroring the Rust `hoist_scope` thread exactly: a var
"bound" inside any of the three is never even offered as a candidate. Under-collecting is the safe
direction, same as the Rust check.

**Match rule**: collect every `?var` the predicate reads (`var-occurrences` — a plain recursive
walk for `?`-prefixed symbols, no rete-specific knowledge needed once inside an expression), find
every candidate whose bound-var set is a SUPERSET; hoist iff exactly one matches.

**Text-span splice, not re-serialization**: the deleted `where` and the appended predicate are both
copied/removed as raw source spans (`node-start-offset`/`node-end-offset`/`string::subs`), so
everything else survives byte-identical. Per `wat/fix.wat`'s own doctrine, a deletion covers exactly
the `where` form's own span; the whitespace it leaves behind survives untouched (wat-fmt's job, not
this codemod's).

**A defect I found and fixed myself, before anyone else read the diff**: my first draft emitted one
independent insert edit per hoisted `where`. When two `where`s hoist into the SAME target condition
(`:dm::four`'s four `(:dm::Primer …)` conditions each paired 1:1 is fine, but
`probe_arc278_sift_rules_arena.wat`'s `:arena::suspect-rule` has THREE `where`s all targeting the
ONE `:arena::Event` condition), `fix-text-apply`'s two same-offset inserts land in **reverse** of
whatever order they were queued in (each later-processed insert lands to the LEFT of an
earlier-processed one, since both are spliced into the same absolute offset of the still-original
prefix). Two of my own attempts at re-ordering the COLLECTION (`collect-where-sites`) had no effect
on this — the final render order was governed by `(reverse (sort edits))`'s tie-break, which for two
edits sharing an offset falls through to lexicographic comparison of the tuple's THIRD field (the
insert text itself), not encounter order. Fixed at the root: `merge-insert-into` accumulates ALL
predicates targeting the same offset into ONE insert, concatenated in encounter order, so there is
only ever one edit per target position and no tie for `sort` to break. Verified against
`:arena::suspect-rule` (source order total-ns/reputation/country → output order
total-ns/reputation/country, previously reputation/total-ns/country) and `:dm::four` (4 conditions,
1:1, unaffected either way — confirms the bug was specifically the same-target case). This is
exactly the kind of thing DESIGN warned nobody would eyeball at 32 files; it was caught by actually
reading the `sift_rules_arena` diff, not by inspection of the code.

## The dry-run — `/tmp` copies of all 32 files, diffed against the real corpus

`refusable_files.txt` → copied into `/tmp/.../scratchpad/dry-run-copy/` preserving relative paths →
`printf '[...]\n' | ./target/release/wat ./wat-scripts/fixes/hoist-where-into-condition.wat` (OLD
checker binary) → `diff -u` each copy against its real-corpus original. **The real corpus was never
written to** — every `wat::io::write-file` call the codemod itself makes targets only the `/tmp`
copy paths passed on stdin.

**Result: all 32 files changed, zero errors.** Aggregate shape check across the whole diff set:

```
$ grep -c '^-.*:wat::rete::where' all_diffs.txt   →  63   (every `where` deletion, matches the census)
$ grep -c '^-' all_diffs.txt                      →  158
$ grep -c '^+' all_diffs.txt                      →  153
```

Every one of the 32 per-file diffs was read in full (not sampled) — the phase's whole point, per
DESIGN's "at 32 files nobody will eyeball it afterwards — this is the moment it gets read." Every
hunk is exactly: a `where` line disappears, its predicate's exact source text (including multi-line
predicates — `reduce`/`fn` bodies, a nested `cond`, an `or`, `:undefined` fallback kwargs, a 2-level
field accessor) reappears verbatim as a new trailing clause on the ONE condition that binds every
var it reads, and the vacated line becomes blank/whitespace (per `wat/fix.wat`'s own "surviving
whitespace is wat-fmt's job" doctrine). Representative (full diffs for all 32 are reproducible via
the codemod against the file list in `refusable_files.txt`, not reproduced in full here — 645 lines):

```diff
 (:wat::rete::defrule :dm::gap
-  :when [(:dm::Beat (?t <- :t) (?k <- :kind))
-         (:wat::rete::where (:wat::rete::core::string::= ?k "gap"))]
+  :when [(:dm::Beat (?t <- :t) (?k <- :kind) (:wat::rete::core::string::= ?k "gap"))
+         ]
   :then [(:dm::Gap :t ?t)])
```

```diff
           (:wat::rete::defrule :arena::suspect-rule
-            :when [(:arena::Event (?client <- :client) (?route <- :route) (?timing <- :timing) (?bytes <- :bytes))
-                   (:wat::rete::where (:wat::rete::core::i64::> (:arena::Timing/total-ns ?timing) 500000))
-                   (:wat::rete::where (:wat::rete::core::i64::< (:arena::Client/reputation ?client) 0))
-                   (:wat::rete::where (:wat::rete::core::string::= (:arena::Geo/country (:arena::Client/geo ?client)) "XX"))]
+            :when [(:arena::Event (?client <- :client) (?route <- :route) (?timing <- :timing) (?bytes <- :bytes) (:wat::rete::core::i64::> (:arena::Timing/total-ns ?timing) 500000) (:wat::rete::core::i64::< (:arena::Client/reputation ?client) 0) (:wat::rete::core::string::= (:arena::Geo/country (:arena::Client/geo ?client)) "XX"))
+                   
+                   
+                   ]
             :then [(:arena::Suspect :client ?client :route ?route :timing ?timing :bytes ?bytes)])
```

**Correctly excluded, confirmed by reading the source, not just trusting a zero-diff**:
`probe_arc278_derived_exists_acc.wat`'s accumulate-bound `?n` `where`; `datamancer.src.wat`'s
genuine `?g`/`?t` cross-condition join in `:dm::read-after`; every `where` inside a `:not`/`:exists`
(the codemod's synthetic regression tests below).

## Ambiguous sites — NONE found

Per the phase's own STOP condition ("more than one condition could host the predicate ... STOP and
list those sites"): zero. Every one of the 63 real sites resolved to exactly one candidate; no site
was skipped for ambiguity (only the 12 that are genuinely non-hoistable — cross-joins and an
accumulate result var — were skipped, and those are BY DESIGN, not a check hole).

## Idempotency — confirmed, 0 changes on re-run

Ran the codemod a SECOND time against its own already-migrated output (`idem-copy/`, a copy of
`dry-run-copy/` after the first pass): `diff -q` against every one of the 32 files reports zero
differences. Re-derived directly, not assumed — once a `where` is hoisted there is no more
`(:wat::rete::where …)` node at that site for a second pass to find.

## Regression probes — synthetic, run through the SAME codemod, matching the check's own negative
controls

Before trusting the codemod against real files, drove eight small synthetic cases through
`:user::migrate` (mirroring the check's own four negative controls plus edge cases the merge-bug
fix needed): plain-pattern hoist, factbind-pattern hoist, `where` positionally BEFORE its binder,
two `where`s in one `:when` with two different targets, two `where`s targeting the SAME target
(the merge-bug case), a `where` nested inside `:not`-of-`:and` (must NOT hoist), two conditions each
binding the same var name (ambiguous — must NOT hoist), a genuine cross-condition join (must NOT
hoist), an accumulate result var (must NOT hoist). All eight matched the predicted output exactly.

## Check-enabled re-verification — every migrated file re-compiles clean

Popped the check back in (see stash dance above), rebuilt, ran the check-enabled binary against all
32 migrated `/tmp` copies **from the copy tree's own root as CWD** (see the near-miss below for why
that CWD choice matters): zero files still produce a `WhereHoistable` error. This confirms the
codemod's own target form is accepted by the compiled validator it was built to satisfy, not merely
"looks right" — the same instrument that found the 63 sites confirms none remain.

## ⚠ A near-miss worth recording: a hardcoded relative `write-file` path, and a CWD mistake

`tests/rete/datamancer.src.wat`'s `:user::main` ends in
`(:wat::io::write-file "tests/rete/datamancer.rete.edn" …)` — a STRING LITERAL relative path baked
into the corpus file itself. Running `./target/release/wat <path-to-the-/tmp-copy>` from the repo
root (as I did once, to check-enabled-verify all 32 files at once) resolves that literal path
against the CURRENT WORKING DIRECTORY, not the copy's location — so it silently overwrote the REAL,
tracked `tests/rete/datamancer.rete.edn` with the compiled-from-migrated-source export (a genuinely
different EDN: fewer `:j`/`:t` beta-join/test nodes, since the hoisted predicate is now an inline
alpha test). Caught via `git status` showing an unexpected `M` on a file I had not intended to
touch; the diff's own shape (fewer nodes, matching exactly what hoisting predicts) confirmed the
mechanism before I reverted it. **Fixed**: `git restore tests/rete/datamancer.rete.edn` immediately,
then re-ran the same check-enabled verification a second time with the check-enabled binary invoked
from the `/tmp` copy tree's OWN root as CWD (so the same relative path resolves harmlessly inside
`/tmp`), confirmed zero `WhereHoistable` findings, confirmed `git status` shows the real file
untouched. This is disclosed because it is exactly the kind of real-corpus side effect Phase 1
exists to prevent, even though the corpus file that nearly got touched (`datamancer.rete.edn`) is
not itself one of the 63 `where` sites — it is a *generated* artifact one of the 32 SOURCE files
would regenerate as an unrelated side effect of being executed at all.

## Floor — before and after, byte-identical working tree

`pgrep -af 'cargo|nextest'` checked clear before the run (only unrelated long-lived `wat --mcp`
processes present). `./scripts/floor.sh`, foreground:

```
     Summary [ 453.261s] 5490 tests run: 5490 passed (2 slow), 19 skipped
```

Captured at `.floor/2026-09-09T23-34-40Z/` (`.floor/latest`). **5490/5490, matching the stated
baseline exactly** — expected, since nothing in the real corpus or `src/` changed: `git status`
immediately before AND after the floor run shows only `?? wat-scripts/fixes/hoist-where-into-condition.wat`.

## What I did NOT do (first pass — dry-run only, before the coordinator's go-ahead)

- **Did not apply the codemod to the real `tests/` corpus.** All 32 files remain exactly as they
  were; only `/tmp` copies were rewritten, per this phase's explicit "STOP after the dry-run diff"
  instruction. (Superseded below — the coordinator read this diff, approved it on substance,
  asked for one refinement, then asked for Phase 1 to be applied for real. See "Applied for real"
  further down.)
- **Did not touch `wat-scripts/`** — Phase 2's tree, untouched, per the ⛔ in the brief. Still true
  after applying for real.
- **Did not restore the check permanently** — it was cherry-picked, used twice (enumeration, then
  verification), and stashed away + dropped both times. `src/rete/validate/{mod,error}.rs` are
  byte-identical to HEAD. Restoring it for real is Phase 3.
- **Did not re-derive DESIGN's `wat-scripts/` figures** — out of Phase 1's scope entirely (that tree
  is untouched).
- **Did not independently re-run each fixture's fire/query assertions** against the migrated form.
  Most of the 32 files carry **no `:user::main`** at all (`grep` confirms — e.g. every
  `*_native_differential.wat`, `*_two_where_native_spec.wat`, `export.wat`,
  `sift_rules_arena.wat`): they are driven only by a paired `.rs` test via `startup_from_file`, which
  only runs against the REAL corpus path under `cargo nextest`. Proving fire/query-level
  match-set-and-three-way-agreement preservation on the migrated form is exactly what Phase 3's
  "restore the check and floor green" step re-proves (the paired tests will run against the NEW
  form once it lands for real); re-deriving it against a `/tmp` copy here would either need to fake
  the test harness's own file-discovery or duplicate Phase 3's job early. What Phase 1 DID confirm:
  the check-enabled compiled validator (the same instrument that PROVED the two forms equivalent in
  the original where-fence-hoistable strike, `f47a9fccc`'s SCORE) accepts every migrated file
  cleanly, and the codemod's own eight synthetic regression probes (mirroring the check's four
  negative controls) all resolve exactly as predicted.
- **Did not commit the near-miss `datamancer.rete.edn` change** — reverted via `git restore` before
  it ever reached the index; `git status` throughout the write-up shows it untouched.
- **Did not re-run a red floor** — there was none; both the immediate-post-dance and final floor
  runs were green on the first try.

## Coordinator's refinement — a deleted `where` takes its LINE with it, not just its own span

Approved on substance; one issue: the dry-run diff left a whitespace-only line where each hoisted
`where` used to sit (`:when [(:dm::Beat …)\n         ]` — a dangling indented `]`), and at
`probe_arc278_sift_rules_arena.wat`'s `:arena::suspect-rule`, three in a row. Nothing gates this —
no trailing-whitespace/line-length lint exists, and the corpus already has an 885-char line — but
this branch is the reference for an imminent merge into `main`, and 63 sites of it is noise in
exactly the files a merge will be resolving conflicts in. Fix: **the deletion must remove the
`where`'s own LINE, not just its span**, so the `:when` vector closes naturally.

**The fix — `backward-trim`, asymmetric on purpose.** Added `ws-char?`/`backward-trim` to the
codemod: before building the deletion edit, walk backward from the `where` node's own start offset
through every contiguous whitespace character (space/tab/CR/LF) until hitting a non-whitespace
character (or offset 0); the deletion then runs from THAT position to the node's own end, folding
in its leading indentation and the newline before it. Deliberately **backward-only, never
forward** — reasoned through before touching the file, then confirmed empirically:

- If it also trimmed forward, two adjacent hoisted `where`s' deletion spans could overlap (the
  newline between them would be claimed by BOTH sites), corrupting `fix-text-apply`'s offset
  arithmetic (which depends on non-overlapping, correctly-ordered edits, exactly the invariant
  `positional-to-kwargs.wat`'s own doc calls out).
- Backward-only means site K's trim always stops at site (K-1)'s own closing paren — a
  non-whitespace character — because sites are exactly the `where` forms' own token spans and
  nothing else. Traced by hand for `:arena::suspect-rule` (3 wheres, all hoisting into the SAME
  `:arena::Event` condition): site1's deletion runs `[end of :arena::Event's paren, end of
  where1)`; site2's runs `[end of where1, end of where2)`; site3's runs `[end of where2, end of
  where3)` — three back-to-back, non-overlapping ranges, confirmed by re-running the dry-run (below)
  rather than trusted from the trace alone.
- The one case backward-trim does NOT collapse to a single line — a `where` sharing its `[` on the
  SAME line as `:when` (no line existed for it to remove) — was checked separately and is not a
  regression: `:when [\n         (:t::A …)]` is `:when [` ending its own line normally, not a
  dangling bracket on its own separate line, which is the shape being asked for. No REAL file in
  the 32-file corpus has this shape (confirmed below, "positionally-free" and single-line-rule
  cases are the two patterns that actually occur).

**Re-verified against the eight synthetic regression probes first** (plain-pattern hoist,
factbind-pattern hoist, `where` before its binder, two targets, two `where`s → ONE target — the
merge-bug case, `:not`-of-`:and` exclusion, ambiguous exclusion, cross-join exclusion, accumulate
exclusion): all eight still resolve exactly as before — `backward-trim` only touches the deletion
span's START; it cannot change WHICH sites hoist, only how much whitespace a hoisted site's
deletion also removes. Re-ran double-migrate on three representative cases
(`:dm::gap`, `:arena::suspect-rule`, `:wpf::r-trail`) — idempotent, byte-identical to single-migrate.

## Re-run dry-run, `/tmp` copies, diffed against the recorded diff

Fresh `/tmp` copies of all 32 real files, same codemod (now with `backward-trim`), same OLD-checker
binary (no rebuild needed — `wat-scripts/*.wat` is not frozen into the binary, only `wat/*.wat`'s
stdlib is). All 32 processed cleanly, zero errors.

```
$ grep -c '^-.*:wat::rete::where' all_diffs2.txt   →  63   (unchanged — same 63 sites)
$ grep -c '^+[[:space:]]*$' all_diffs2.txt         →  0    (was N>0 before the fix; now none)
```

Read every one of the 32 diffs again in full (not sampled — same discipline as the first pass).
Confirmed: the ONLY difference from the recorded first-pass diff is the absence of the
whitespace-only lines; every hoisted predicate lands in the exact same place, verbatim, as before.
Representative:

```diff
 (:wat::rete::defrule :dm::gap
-  :when [(:dm::Beat (?t <- :t) (?k <- :kind))
-         (:wat::rete::where (:wat::rete::core::string::= ?k "gap"))]
+  :when [(:dm::Beat (?t <- :t) (?k <- :kind) (:wat::rete::core::string::= ?k "gap"))]
   :then [(:dm::Gap :t ?t)])
```

```diff
           (:wat::rete::defrule :arena::suspect-rule
-            :when [(:arena::Event (?client <- :client) (?route <- :route) (?timing <- :timing) (?bytes <- :bytes))
-                   (:wat::rete::where (:wat::rete::core::i64::> (:arena::Timing/total-ns ?timing) 500000))
-                   (:wat::rete::where (:wat::rete::core::i64::< (:arena::Client/reputation ?client) 0))
-                   (:wat::rete::where (:wat::rete::core::string::= (:arena::Geo/country (:arena::Client/geo ?client)) "XX"))]
+            :when [(:arena::Event (?client <- :client) (?route <- :route) (?timing <- :timing) (?bytes <- :bytes) (:wat::rete::core::i64::> (:arena::Timing/total-ns ?timing) 500000) (:wat::rete::core::i64::< (:arena::Client/reputation ?client) 0) (:wat::rete::core::string::= (:arena::Geo/country (:arena::Client/geo ?client)) "XX"))]
             :then [(:arena::Suspect :client ?client :route ?route :timing ?timing :bytes ?bytes)])
```

```diff
 (:wat::rete::defrule :wpf::r-trail
-  :when [(:wpf::A (?id <- :id) (?k <- :k))
-         (:wpf::B (?id <- :id)) (:wpf::C (?id <- :id))
-         (:wat::rete::where (:wat::rete::core::string::= ?k "yes"))]
+  :when [(:wpf::A (?id <- :id) (?k <- :k) (:wat::rete::core::string::= ?k "yes"))
+         (:wpf::B (?id <- :id)) (:wpf::C (?id <- :id))]
   :then [(:wpf::Trail :x ?id)])
```

Idempotency re-confirmed on this fixed version too: a second pass over the migrated copies makes
zero further changes.

## Applied for real — all 32 `tests/` files

`git status` checked clean (only the modified codemod file) immediately before. Applied the
codemod directly to the 32 real paths (listed explicitly, one `printf`/EDN vector, no hand-edits,
no sed):

```
$ cat apply_real_paths.edn | ./target/release/wat ./wat-scripts/fixes/hoist-where-into-condition.wat
```

All 32 printed `[hoisted]`, zero errors. `git status --short` immediately after: exactly the 32
target files `M`, plus the already-modified codemod file — nothing else, `tests/rete/datamancer.rete.edn`
untouched. `git diff -- tests/` shows 63 `where`-deletion lines removed, zero whitespace-only added
lines. Byte-compared every one of the 32 real (now-modified) files against the just-recorded
`/tmp` dry-run copies: **identical**, confirming the applied corpus is exactly what was reviewed.

## Check-enabled re-verification, again — zero remaining `WhereHoistable`, real corpus content

Re-ran the stash dance (cherry-pick `f47a9fccc`, `cargo build --release`) to get the check-enabled
binary back. Learned from the earlier near-miss: verified against the `/tmp` copy tree (byte-
identical to the real, now-applied files) **as CWD**, not the real repository root, so
`datamancer.src.wat`'s hardcoded relative `write-file` path cannot resolve against the real
`tests/rete/datamancer.rete.edn` no matter what its `:user::main` does. Zero of the 32 files produce
a `WhereHoistable` finding. `git status --short tests/rete/datamancer.rete.edn` confirmed empty
(untouched) immediately after. Stashed the check away and dropped it, rebuilt the OLD checker
(`grep -n "Where(_)" src/rete/validate/mod.rs` shows all four original `{}` arms, matching HEAD).

## Idempotency, on the REAL corpus this time

Snapshotted the 32 real (already-migrated) files, re-ran the codemod against their real paths a
second time: all 32 still print `[hoisted]` (the driver always attempts every path; "hoisted" means
"processed", not "changed"), but byte-comparison against the pre-re-run snapshot shows **zero
files changed**. `git status --short` line count unchanged (33: the 32 corpus files + the codemod).

## ⛔ Floor — RED. 24 failures. NOT re-run. NOT committed. Corpus reverted; finding surfaced instead.

`pgrep -af 'cargo|nextest'` checked clear immediately before (only unrelated `wat --mcp` processes).
`./scripts/floor.sh`, foreground:

```
     Summary [ 452.222s] 5490 tests run: 5466 passed (2 slow), 24 failed, 19 skipped
```

Captured whole at `.floor/2026-09-10T00-02-20Z/` (`ARM.txt`, `clean.log`, `raw.log`; also preserved
outside `.floor/` at the scratchpad in case a later `floor.sh` run recycles the directory). Per
`wat-rs/CLAUDE.md`: **DO NOT RE-RUN, capture whole, name the exact arm, surface as a finding.**
Confirmed BEFORE reading a single failure that this was not check-dance contamination: `git status`
showed exactly the 32 migrated files + the codemod, nothing from the check-enabled verification
phase; `grep -n "Where(_)" src/rete/validate/mod.rs` showed all four original `{}` arms, matching
HEAD. **These 24 failures are a real, reproducible consequence of the migration itself, against the
unmodified current compiler — not contamination, not a check artifact.**

### The finding: `check_where_hoistable`'s "could legally move" contract only verifies VARIABLE
BINDING. It never verifies the target form still compiles or behaves identically as an inline
clause. At least four independent, unrelated compiler-level facts falsify that assumption — read
every one before forming a theory, per `[[feedback_a_finding_names_one_site_enumerate_the_rest]]`.

**(A) — 9 failures — an inline clause is NOT the same grammar as a `where`'s interior.** A `where`
fence's predicate can call an arbitrary PURE user-defined/foreign function
(`probe_arc278_foreign_pred_purity.wat`'s whole existence proves this is a supported, blessed shape).
An INLINE clause cannot: `classify_rete_clause`'s `Predicate` arm gates on
`expr_is_provably_boolean` (`src/rete/clause.rs:242`), which recognizes `and`/`or`/`not`/`if`/
`let`/`match`/a bool literal/a known `RETE_OPS` row **by shape alone, with no registry** — an
arbitrary function call is never in that set, foreign or not. Two distinct files, same mechanism,
verbatim:

```
thread 'probe_arc278_6b_ii_a_where_oracle::where_with_user_fn_predicate_blocks' panicked at
tests/rete/probe_arc278_6b_ii_a_where_oracle.rs:63:5:
where (big? 50) false → 0 Gates; got Err("startup: #wat.rete/ReteCheckErrors {:message
\"#wat.rete/ReteCheckErrors {:message \\\"1 rete rule validation error\\\" ... #wat.rete/MalformedClause
{:message \\\"defrule `wb::big-gate` (`:weather::Temperature`): malformed rete clause
`(:test/big? ?c)` — not a recognized :when shape\\\" :location ... {:file \\\"tests/rete/probe_arc278_6b_ii_a_where_oracle_userfn.wat\\\"
:line 11 :col 44 ...} ... :rule \\\"wb::big-gate\\\" :fact-type \\\"weather::Temperature\\\"
:clause \\\"(:test/big? ?c)\\\"}]}" ...}")
```
(and its `_passes` sibling, identical mechanism, `probe_arc278_6b_ii_a_where_oracle.rs:56:5`).

`probe_arc300_2_fix_defrule.rs`'s 7 tests share ONE `startup_beside` fixture, so a single hoisted
`(:fix/head-keyword-str? ?name)` — same mechanism exactly — takes down all 7 at once:
```
thread 'probe_arc300_2_fix_defrule::head_keyword_deduces_headconv' panicked at src/freeze.rs:1165:9:
call_beside_value: fixture beside ".../tests/rete/probe_arc300_2_fix_defrule.rs" failed to freeze:
#wat.rete/ReteCheckErrors {... #wat.rete/MalformedClause {:message \"defrule
`fix::head-keyword->conv` (`:fix::Node`): malformed rete clause `(:fix/head-keyword-str? ?name)`
— not a recognized :when shape\" :location ... {:file \"tests/rete/probe_arc300_2_fix_defrule.wat\"
:line 67 :col 52 ...} :rule \"fix::head-keyword->conv\" :fact-type \"fix::Node\" :clause
\"(:fix/head-keyword-str? ?name)\"}]}
```
(identical panic, same file/line, for `left_arrow_deduces_arrowconv`, `non_arrow_symbol_deduces_nothing`,
`post_arrow_keyword_deduces_typeconv`, `post_arrow_keyword_is_not_headconv`,
`right_arrow_deduces_arrowconv`, `type_shaped_keyword_deduces_typeconv_even_when_not_post_arrow`).

**(B) — 2 failures — hoisting a DELIBERATELY malformed `where` moves it out of `where`'s OWN
Law-A/totality diagnostic path.** `probe_fence_names_the_head_core_op.wat` and `_partial.wat` are
NEGATIVE fixtures on purpose: `(:wat::rete::where (:wat::core::i64::> ?c 0))` — a bare
`:wat::core::` op inside a `where`, meant to be refused by `where`'s OWN compile path
(`compile-condition`, per the codemod's own header note at `wat/rete.wat`) with a Law-A-specific
diagnostic. My census correctly found these hoistable-by-variable-binding (one var, one binder) —
the check has no way to know the predicate is deliberately illegal — but hoisting them moves the
SAME malformed head into the GENERIC inline-clause classifier instead, which reports a DIFFERENT,
less specific error:
```
thread 'probe_fence_names_the_head::core_op_where_names_law_a_not_total' panicked at
tests/rete/probe_fence_names_the_head.rs:109:5:
assertion `left == right` failed
  left: "startup: ... MalformedClause {:message \"defrule `wf::bad-gate` (`:weather::Temperature`):
  malformed rete clause `(:wat.core.i64/> ?c 0)` — not a recognized :when shape\" ...}"
 right: "compile-condition: where expr is not a rete primitive — ':wat::core::i64::>' is not a
  rete primitive; a where admits only :wat::rete:: ops"
```
```
thread 'probe_fence_names_the_head::partial_where_names_the_offending_head_and_axis' panicked at
tests/rete/probe_fence_names_the_head.rs:97:5:
assertion `left == right` failed
  left: "startup: ... MalformedClause {:message \"defrule `wf::bad-gate` ...: malformed rete clause
  `(:wat.core.i64// ?c 1)` — not a recognized :when shape\" ...}"
 right: "compile-condition: where expr is not total — ':wat::core::i64::/' is not total"
```
The `where` fence is not JUST a beta-vs-alpha performance distinction — it is also its OWN
validation surface (Law A enforcement, totality) that inlining bypasses entirely, replacing a
targeted diagnostic with a generic one. Both fixtures still fail to compile before AND after
hoisting; only the ERROR TEXT changes, which is exactly what these two tests assert on.

**(C) — 1 failure — a termination-BOUNDING `where` fence is not equivalent to the same test
inlined, for the round-cap VERIFIER specifically**, independent of match-set:
```
thread 'probe_arc278_fixpoint_round_cap::a_fence_bounded_counter_is_admitted_and_its_wrong_way_twin_is_not'
panicked at tests/rete/probe_arc278_fixpoint_round_cap.rs:396:5:
assertion `left == right` failed: and CONVERGE at 501 — the seed plus every step up to the bound.
Admitting a rule set that then hangs would be worse than refusing it
  left: "\"ARM MayNotTerminate\"\n\"gc::count-up\"\n\"gc::N\""
 right: "\"501\""
```
`probe_arc278_termination_guarded_counter.wat`'s `gc::count-up` uses `(where (< ?k 500))` as a
counter's own termination bound. Once inlined into the alpha condition, the STATIC
may-not-terminate verifier — a SEPARATE analysis from `check_where_hoistable`, reasoning over the
compiled rule graph's shape — now refuses to admit the rule set at all, where it previously proved
termination and ran to completion. This is a genuine BEHAVIORAL regression (refusal vs 501
successful rounds), not a cosmetic diagnostic difference — the two forms are NOT interchangeable
for this analysis, directly contradicting the premise that hoisting is a pure optimization.

**(D) — 1 failure — an alpha condition does not compile the same closure-bearing expression a beta
test does:**
```
thread 'probe_arc278_reduce_arity_totality::the_partial_two_arity_form_is_refused' panicked at
tests/rete/probe_arc278_reduce_arity_totality.rs:93:5:
the refusal must name the admitted spelling so the author knows what to write; got:
#wat.runtime/MalformedForm {:message "malformed :wat::rete::fire-rules form: alpha 0 cond did not
compile — setup should compile every fact-shaped alpha" :location ... {:file
"tests/rete/probe_arc278_reduce_arity_totality_two.wat" :line 7 :col 4 ...} :head
":wat::rete::fire-rules" :reason "alpha 0 cond did not compile — setup should compile every
fact-shaped alpha"}
```
The hoisted predicate is a `reduce` over a `fn` closure (`(:wat::rete::core::reduce (:wat::rete::core::fn
[acc x] -> i64 ...) 0 ?v)`) — legal inside a `where`, but alpha-node compilation apparently cannot
build this shape the way beta-test compilation could.

**(E) — 10 failures — a test's own scaffolding depends on the `where`'s compiled node existing,
for reasons UNRELATED to the where/hoist question itself.** All 10 are `probe_arc278_export.rs`
tests sharing a fixture built around `:exp::cool`'s `where`; they use the exported `:prog`-tagged
node the `where` used to compile to as a convenient, pre-existing depth-probe subject for testing
UNRELATED arity/depth-refusal behavior. With the `where` gone, that scaffolding node is gone too:
```
thread 'probe_arc278_export::a_well_formed_user_call_still_runs' panicked at
tests/rete/probe_arc278_export.rs:755:5:
cool-export packs no [:prog …] — these depth probes would be vacuous
```
(identical panic, same file/line, for `arity_refuses_a_call_with_no_arguments_at_all`,
`arity_refuses_a_surplus_that_falls_past_the_frame`,
`arity_refuses_a_surplus_that_collides_with_a_declared_slot`,
`arity_refuses_arguments_to_a_zero_parameter_callee`,
`arity_refuses_too_few_arguments_on_the_evaluating_path`,
`import_refuses_a_node_graph_with_dangling_child_edges`,
`import_refuses_a_pattern_tower_past_the_depth_bound`,
`import_refuses_a_user_prog_cycle_tower_past_the_depth_bound`,
`import_refuses_an_and_tower_past_the_depth_bound`, `import_refuses_op_outside_rete_ops`).

**9 + 2 + 1 + 1 + 10 = 24, accounts for every failure**, confirmed against the full FAIL list
(`clean.log` line numbers 408, 430, 594, 613, 632, 651, 672, 691, 712, 731, 751, 771, 794, 836, 938,
1035, 1054, 1074, 1093, 1112, 1131, 1150, 1181, 1206 — 24 lines).

### Why this is a check-contract gap, not a codemod bug

The codemod is a faithful, independently-verified mirror of `check_where_hoistable`'s own decision
procedure (var-binding superset, exactly one match). Every one of these 63 sites — the 24 broken
ones included — is genuinely "hoistable" BY THAT DEFINITION. The gap is in the definition itself:
"could legally move" was formalized as "every var is bound by exactly one condition," which says
nothing about whether the MOVED predicate (a) is drawn from the narrower inline-clause grammar
(A), (b) was relying on `where`'s OWN separate validation path for its diagnostic (B), (c) is
transparent to the SAME degree to the termination verifier in both positions (C), (d) compiles
identically as an alpha vs. a beta test (D), or (e) is depended upon structurally by something
that has nothing to do with hoisting at all (E). None of these are things `check_where_hoistable`
or this codemod could have caught by construction — they are facts about OTHER parts of the
compiler (the inline-clause grammar, `where`'s Law-A path, the termination verifier, alpha
compilation) that the hoist transform's own contract never claimed to preserve.

### Action taken: reverted the 32 files, kept the codemod and this SCORE.md

Per the exact precedent this same strike already set once (`23625037b`, "NOT SHIPPED": the check
itself was reverted and only its SCORE.md kept, when it turned out to refuse correct code) — `git
restore`d all 32 real corpus files back to their last-committed (pre-migration) content. Verified
after: `git status --short` shows only the codemod file and this SCORE.md modified — the corpus is
byte-identical to `HEAD` again. **Did NOT re-run the floor after reverting** (a green re-run would
prove nothing new and risks reading as "the problem went away"; the red evidence stands on its own
in `.floor/2026-09-10T00-02-20Z/`, preserved). Did NOT commit anything. Did NOT attempt to patch
the codemod to special-case these 24 sites — narrowing the codemod's OWN hoist criterion to also
require inline-clause-grammar admissibility, Law-A/totality parity, termination-verifier parity,
AND absence of unrelated structural dependents is a real design decision (four independent new
exclusion axes, at least one of which — (E) — is not even a property of the `.wat` source at all,
but of a SIBLING `.rs` test's assumptions) that belongs to whoever owns `check_where_hoistable`'s
contract, not something to improvise mid-codemod.

## Floor, one more time — on the REVERTED state, before committing anything

Not a re-run of the red floor (a different tree: the corpus reverted, the codemod carrying only its
own whitespace refinement). `git status --short` confirmed clean except the codemod file and this
SCORE.md before starting; `pgrep -af 'cargo|nextest'` clear. `./scripts/floor.sh`, foreground:

```
     Summary [ 452.667s] 5490 tests run: 5490 passed (1 slow), 19 skipped
```

Matches baseline exactly. Committing the codemod's whitespace refinement and this SCORE.md only —
per house rule, commit ONLY on green, and this is the first green measured for this exact tree
state (the corpus-reverted one), not a re-run of the red one.

---

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_018ntHDMRNCKDKNr2gVfzXmP
