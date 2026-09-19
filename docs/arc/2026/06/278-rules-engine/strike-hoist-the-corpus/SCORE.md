# SCORE — Phase 1 (`tests/`, 32 files): dry-run diff, reported, nothing applied

Executing `DESIGN.md`'s Phase 1 only. Written as-I-go per house rule. Branch `grok-rete`, floor
baseline `5490/5490` confirmed green both before touching anything and after (see "Floor" below —
the working tree is IDENTICAL at both ends; only `wat-scripts/fixes/hoist-where-into-condition.wat`
is new).

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

## What I did NOT do

- **Did not apply the codemod to the real `tests/` corpus.** All 32 files remain exactly as they
  were; only `/tmp` copies were rewritten, per this phase's explicit "STOP after the dry-run diff"
  instruction.
- **Did not touch `wat-scripts/`** — Phase 2's tree, untouched, per the ⛔ in the brief.
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

---

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_018ntHDMRNCKDKNr2gVfzXmP
