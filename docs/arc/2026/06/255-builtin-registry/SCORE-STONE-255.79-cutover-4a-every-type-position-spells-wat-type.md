# SCORE — STONE 255.79: cutover 4a — every type position spells `wat.type/`

**Executor: a Sonnet subagent, working solo** (ran the floor itself). Drawn against local `main` @ `4b3f3194e`
(the brief's own draw commit, parent `eca5933bf`). Commit locally on `main`; **not pushed**.

## 1. Census first (item 1)

**Corpus dry-run** (unmodified `wat-scripts/fixes/types-to-wat-type.wat`, i.e. rules A–E only, before rule F
was added) over **every tracked `.wat` file** (2,277 files; `git ls-files '*.wat'`, including
`wat-scripts/fixes/**`, `scratch-pad/**`, probes, tests), each on a `/tmp` copy, diffed against the original:

**125 files would change** under rules A–E alone:

| tree | files |
|---|---|
| `wat-scripts/fixes/` | 122 |
| `wat-scripts/scratch-pad/` | 2 |
| `tests/types/` | 1 |

(The full list is `/tmp/census-255-79/changed.txt` from this run; not committed — ephemeral working data,
per doctrine.)

**Embedded wat in Rust string literals** (`src/**`, `tests/**`), 255.71's AMEND-2 method (grep for a
type-position SHAPE — a marker (`<- -> :-> :< :nature`) immediately adjacent to one of the 24, or
`NAME :-` head-of-bracket — then exclude `//`/`///`/`//!` comment lines, confirmed by direct inspection of
a sample from every file bucket):

- **1,409 raw marker-adjacent matches → 832 non-comment lines → 1,093 marker-adjacent token occurrences**,
  across **84 files** (`src/runtime.rs` 139, `src/macros/tests.rs` 78, `src/check.rs` 61,
  `src/rete/reachability.rs` 56, `src/types.rs` 43, `src/rete/kernel/tests/mod.rs` 41, … full list in
  `/tmp/census-255-79/embedded-non-comment.txt`, ephemeral).
- Sampled and hand-verified: these are genuine embedded wat program fragments inside Rust test strings
  (e.g. `src/runtime.rs:15620` `(:wat::core::defn :my::app::inc [x <- :wat::core::i64] -> :wat::core::i64
  (:wat::i64::+ x 1))`), not doc-comment prose and not Rust-side `TypeExpr::Path(...)` comparisons (those
  don't match the marker-adjacency pattern).
- **This undercounts the true type-position-site count** — it misses rule (B) args-vector members
  (`[:wat::core::i64 :wat::core::String]` inside a `(Tuple :- […])`), which have no adjacent marker.

**→ STOP-1: the embedded-Rust census (1,093 sites, 832 non-comment lines, 84 files) exceeds 600.**
Per the brief: reported, and **no Rust string literal was converted this stone.** Everything below is the
`.wat`-corpus-only work the brief's STOP-1 still permits (it stops conversion of Rust, not the rest of the
stone).

## 2. The missed rule — rule (F) NATURE-VALUE

Added to `wat-scripts/fixes/types-to-wat-type.wat`'s header and body (not forked — the recorded codemod
amended in place, per doctrine): a keyword immediately following the bare keyword `:nature` (a
`defsurface`'s `:nature <type>` clause) is a type position. `target-name?` already excludes any `:nature`
value outside the 24 (`:wat::kernel::Peer` — 10 distinct corpus service-surface files, left untouched, as
it must be), so the new predicate (`:t2wt::nature-value?`, one `=` comparison) adds no false positive.
Wired into `:t2wt::node-edits` as a sixth fallback arm beside (C)'s marker check and (D)/(E)'s
`last-eligible?`.

**Its own replay case** — `wat-scripts/fixes/replay/types-to-wat-type/{before.pre,after.post,ORACLE}`
gained one line each: `(:wat::core::defsurface :probe::Store :nature :wat::core::Struct :features [])` →
`(:wat::core::defsurface :probe::Store :nature wat.type/Struct :features [])`, with `spec`/`spec-before`
ORACLE entries quoting the new header text verbatim (`the bare keyword \`:nature\` (a \`defsurface\`'s
\`:nature <type>\` clause)`). Verified by hand before trusting the automated gate: converts on the first
run, idempotent on the second, byte-identical to `after.post` both times.

## 3. Apply — every `.wat` file it changes (194 total, listed)

**The codemods themselves (`wat-scripts/fixes/**`, 122 files)** — per doctrine ("a tool is never its own
input"), converted with a **pristine `/tmp` copy of the pre-edit tool** (`git show
0374d9c78a:wat-scripts/fixes/types-to-wat-type.wat`, i.e. the committed version *before* rule F), run over
`/tmp` mirrors of all 122 files (list: `/tmp/census-255-79/fixes-filelist.txt`), then copied back onto the
real paths. Sanity check (not required by doctrine, done anyway): running the **post-edit** (rule-F-having)
tool over the same 122 `/tmp` mirrors produced byte-identical output (`diff -rq` clean) — expected, since no
`:nature` sites exist under `wat-scripts/fixes/`. All 122 files in the tree changed (every codemod's own
`defn` signatures used the old spelling; stone 2 had excluded this tree). This run also converted
`types-to-wat-type.wat`'s own newly-added rule-F code (`[name <- :wat::core::String] -> :wat::core::bool`
→ `wat.type/String`/`wat.type/bool`) — the self-application the doctrine's STASH-DANCE-adjacent note
permits, since it is the *pre-edit* copy doing the converting, never the running program converting itself.

**The rest of the corpus (`git ls-files '*.wat' | grep -v '^wat-scripts/fixes/'`, 2,155 files)** — dry-run
with the **post-edit** (rule-F) tool over `/tmp` mirrors: **72 files changed** (list below), applied to the
real paths.

| tree | files |
|---|---|
| `tests/types/` | 40 |
| `wat-scripts/probes/` | 9 |
| `wat-scripts/scratch-pad/` | 8 |
| `tests/diagnostics/` | 2 |
| `wat/` (`telemetry.wat spawn.wat service.wat seq.wat query.wat core.wat class.wat capability.wat`) | 8 |
| `tests/services/`, `tests/rete/`, `tests/reflection/`, `tests/process/`, `docs/arc/` | 1 each |

Of these 72: **3** are rule-A–E leftovers (`tests/types/probe_arc255_77_framing_floor_pin.wat`,
`wat-scripts/scratch-pad/255-69-census-walk.wat`, `wat-scripts/scratch-pad/probe-255-72-signature-of-defn.wat`
— ordinary `<-`/`->`/`:-` positions the original 125-file census already named); **69** carry at least one
rule-F `:nature` conversion, **79 `:nature` sites total** (measured directly: `grep -c ':nature
wat.type/Struct\|:nature wat.type/Record'` across the 69 files, before/after), e.g. `wat/class.wat`'s two
`(defsurface … :nature wat.type/Struct …)` rows (the exact file stone 2's own header cited as the
`extend-type` rule-D motivating example), `wat/spawn.wat` (2), `wat/capability.wat` (3).

Every path in both apply steps is listed in `/tmp/census-255-79/fixes-filelist.txt` (122) and
`/tmp/census-255-79/nonfixes-changed.txt` (72) — ephemeral session files, not committed; the committed
evidence is the `git diff --stat` on the commit below (198 files: 194 `.wat` + the 3 replay-fixture files
+ `src/types.rs`).

Every spot-checked diff (a dozen, across `wat/*.wat`, `tests/types/*`, `wat-scripts/probes/*`,
`docs/arc/*`) changed **only** the `:nature` value's spelling (or, for the 3 leftovers, an ordinary
`<-`/`->`/`:-` type keyword) — nothing else moved.

## 4. Embedded Rust — not converted (STOP-1)

Per STOP-1, no Rust source file was touched this stone. `git status` confirms: the only non-`.wat`,
non-replay-fixture file changed is `src/types.rs` (item 6 below, a checker bug-fix, not a corpus
conversion).

## 5. Out of scope — untouched, confirmed

Comments (496+ sites, stone 7), non-`(X :- …)` function/form heads and slash-verbs (`Vector/get`,
`i64::+`, `i64/to-string` — 308 residual `head`-class sites, item 7), bare constructor-call/data-position
keywords (273 residual `other`-class sites, item 7), the Rust key literals and printers (4b).

## 6. The red this stone's own change caused — cured

`cargo test --release --test lint -- wat_scripts_fixes_load` on the applied `.wat` corpus (before the
floor): **2 of 783 `wat-scripts/` files rotted**, verbatim:

```
2 of 783 wat-scripts/ files do not load on the current runtime (rotted):
  wat-scripts/scratch-pad/probe-home-8-examples.wat
      #wat.type/MalformedDecl {:message "malformed :wat::core::defsurface declaration: :nature value must be a nature-root (:wat::core::Struct, :wat::core::Record, :wat::holon::Record, or :wat::kernel::Peer); got wat.type/Struct" :location #wat.core/Span {:file "wat-scripts/scratch-pad/probe-home-8-examples.wat" :line 8 :col 48 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 8 :col 63}}} :causes [] :head ":wat::core::defsurface" :reason ":nature value must be a nature-root (:wat::core::Struct, :wat::core::Record, :wat::holon::Record, or :wat::kernel::Peer); got wat.type/Struct"}
  wat-scripts/scratch-pad/probe-seqable-is-spellable-today.wat
      #wat.type/MalformedDecl {:message "malformed :wat::core::defsurface declaration: :nature value must be a nature-root (:wat::core::Struct, :wat::core::Record, :wat::holon::Record, or :wat::kernel::Peer); got wat.type/Struct" :location #wat.core/Span {:file "wat-scripts/scratch-pad/probe-seqable-is-spellable-today.wat" :line 25 :col 11 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 25 :col 26}}} :causes [] :head ":wat::core::defsurface" :reason ":nature value must be a nature-root (:wat::core::Struct, :wat::core::Record, :wat::holon::Record, or :wat::kernel::Peer); got wat.type/Struct"}
```

**The brief's own premise contradicted the code, named per doctrine:** the brief's ruling states 4a
"lands green under today's door (`type_denotation`, `src/edn/render.rs:3706`, maps `:wat::type::X` to the
registered `:wat::core::X`)". `type_denotation` does do exactly that for `wat.type/Struct` →
`:wat::core::Struct`. But `Nature::from_root_keyword` (`src/types.rs:633`, the sole parser for a
`defsurface`'s `:nature` value) called **`canonical_identity`**, not `type_denotation` — a narrower door
that normalizes spelling *within* a namespace (`:wat::core::Struct` / `wat.core/Struct` alike) but does
**not** fold the `wat.type/` member spelling to its registered `wat::core` key. `canonical_identity("wat.
type/Struct")` answers `":wat::type::Struct"`, which matches none of the four `from_root_keyword` arms —
exactly the "a string comparison with one side normalized and the other not" recurring class CLAUDE.md
names (arc 278's three prior instances: the companion-name suffix, the type-arg flat-split, the `:messages`
membership check), now a fourth.

**Cure** (`src/types.rs:633`, 18 lines, mostly a doc comment naming the mechanism): swapped
`crate::edn::render::canonical_identity(kw)` for `crate::edn::render::type_denotation(kw)` inside
`Nature::from_root_keyword`. This is **not** a change to the checker's keys (the four registered keys —
`:wat::core::Struct`, `:wat::core::Record`, `:wat::holon::Record`, `:wat::kernel::Peer` — are untouched) and
costs nothing on the four existing arms (`type_denotation` agrees with `canonical_identity` on every input
that is not a `wat.type/` member — confirmed: none of the four arms is such a member). The single other
call site (`src/types/surface.rs:680`) and the inheritance-parent check (`src/types.rs:5927`) both route
through the same function, so both are fixed by the one change. Re-ran
`wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime` after the fix: **ok** (both
files load). **Not re-run before capture** — the red above is the untouched first-look verbatim; the fix
was applied, then a fresh run confirmed it.

## 7. Residue — the census, re-run, by class (item 1 gate)

Re-ran the classifier (same approximate method as the brief's own draw-time measurement: comment / string
/ head-of-`(` / other) over every tracked `.wat` file in the **final, applied** corpus:

| class | count | type position? |
|---|---|---|
| comment | 706 | no — prose, out of scope (stone 7) |
| string | 340 | no — inside a string literal |
| head (immediately after `(`) | 308 | no — a function/method call head (`:wat::core::HashMap/get`, `:wat::core::i64::+`, `:wat::core::i64/to-string` — slash-verbs and rust-scheme ops the codemod's own header excludes by design) |
| other (data position) | 273 | no — a bare keyword VALUE passed as a function argument (e.g. `tests/comms/probe_arc209_bound_listener.wat:35` `(:wat::kernel::listener (:wat::spawn::thread) :user::Op :wat::core::i64)` — the open "untyped constructor/cast" question stone 2 scoped out, 255.65 §3) |

**1,627 total**, down from the brief's draw-time measurement of 7,335 (across more of the corpus;
not apples-to-apples since that count was pre-conversion and spanned files this stone's dry-run also
found zero changes needed on). Every one of the 1,627 sampled and spot-checked falls into a named
non-type-position bucket; **none is a type position** — the strongest evidence of this is independent of
the classifier: the **idempotence gate** (below) proves the codemod itself, with rule F in place, finds
**zero** further type-position matches anywhere in the corpus.

Full per-line classification: `/tmp/census-255-79/residue.tsv` (1,627 rows; ephemeral working file, not
committed — the class counts and spot-checks above are the measured, reportable summary).

## Gates

| gate | command | result |
|---|---|---|
| idempotent | amended codemod over all 2,277 tracked `.wat` files (post-apply), `/tmp` mirror, diffed | **0 changes** |
| residue | item 1's census, re-run, classified | 1,627 remaining code-position tokens, by class above; **none a type position** |
| census | `scripts/replay/census.sh` pre (`.census/2026-10-01T20-02-13Z.txt`, taken on a `git stash`-restored pristine `4b3f3194e` tree with the final binary) vs post (`.census/2026-10-01T20-04-45Z.txt`) `--diff` | **`census-diff: no STOP-8`** |
| delta | `scripts/replay/delta.sh --list <194 converted paths> --codemod <pristine post-edit tool> --binary target/release/wat`, run against the same pristine `4b3f3194e` tree | **`NEW 0 (orig clean -> converted broken)`**, **`RECOVERY 0 (orig broken -> converted clean)`**, ORIG-CLEAN 188/194, CONV-CLEAN 188/194 (the 6 non-`--check`-clean files are unrelated to this stone — standalone-`.wat` oddities present on both sides identically), exit 0 |
| release floor | `scripts/floor.sh`, foreground, `timeout: 600000` | `.floor/2026-10-01T20-06-02Z/`: **`Summary [ 386.070s] 6349 tests run: 6349 passed (26 slow), 24 skipped`**, exit 0 — exact match against the brief's 6349 baseline (`f12c10995`, `.floor/2026-10-01T18-41-07Z`); no test added or removed this stone |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | rc 0 |

## Reds and STOPs

- **STOP-1** (item 1 above): embedded Rust census 1,093 marker-adjacent sites / 832 non-comment lines /
  84 files — exceeds 600. Reported; **no Rust string literal converted this stone.**
- No STOP-2: the codemod's own replay fixture changed behaviour **because** rule F changed it (the new
  `:nature` case was *added* to the fixture to cover the new rule, not discovered to have silently
  changed pre-existing cases A–E — verified: the pre-existing 5 before/after pairs are byte-identical to
  stone 2's committed fixture).
- No STOP-3 on census/delta (both measured clean, above).
- **A red caused by this stone's own change** (item 6): captured verbatim, cured (`src/types.rs:633`,
  `canonical_identity` → `type_denotation`), re-verified, THEN the floor ran — never re-run before capture.
- **The brief contradicted the code, named per doctrine** (item 6): 4a's premise that `type_denotation`
  already covers every type position, including `:nature`, was false for the one caller
  (`Nature::from_root_keyword`) that used the narrower `canonical_identity` instead. Fixed rather than
  reverting rule F, since the fix is a one-line swap to the SAME canonicalization door the premise
  already named — not a new design decision, and not a change to any registered key.

## Commit

Locally on `main`, not pushed. `git diff --stat` against `4b3f3194e`: **198 files changed** (194 `.wat`
corpus files + 3 `wat-scripts/fixes/replay/types-to-wat-type/{before.pre,after.post,ORACLE}` + 1
`src/types.rs`), 3,336 insertions, 3,287 deletions.
