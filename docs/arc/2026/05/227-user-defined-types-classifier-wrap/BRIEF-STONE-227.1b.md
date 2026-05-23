# BRIEF — Arc 227 Stone 227.1b — Rename `:wat::holon::defclass` → `:wat::holon::defrecord`

**Stone scope:** Substrate-wide rename of `:wat::holon::defclass` macro to `:wat::holon::defrecord`. File rename + verb rename + test rename + doc updates. HARD CUT — no aliases.

**Type:** Sonnet Mode A.
**Time budget:** 15-45 min target; 90 min STOP.
**Depends on:** Stone 227.1 v3 SHIPPED (commit `0956d25`).
**Calibration:** Smallest precedent — Stone 226.1 (~11 min for 10 new verbs + 27 tests). This is rename-only (no new substrate; no new tests); should be even faster.

## v3 doctrine — why rename now

User articulated 2026-05-22 night, after Stone 227.1 v3 shipped:

> *"this needs to be defrecord who is immutable .... mutations return a new instance.... the data that the holon holds doesn't change - a new holon can be made who holds different data - that's the agreement?"*

**The honest distinction:**

| | defservice (arc 209) | defrecord (renamed from defclass) |
|---|---|---|
| What it wraps | Mutable state | Immutable data |
| Protection model | Mutex (admin/user caps) | None needed (immutability IS protection) |
| "Mutation" | Handler returns `(Tuple NEW-state ...rest)` | Caller constructs NEW instance |
| Methods bundled | YES — protocol-bound | NO — separate defns |
| Analog | Erlang gen_server / Akka actor | Clojure defrecord / Rust struct |

"Class" implies methods + mutable state (OO baggage). "Record" is honest about immutable data-only. Rename locks the honest name BEFORE arc 232 (defprotocol) builds on it.

## Working dir + constraints

- **Working dir: `/home/watmin/work/holon/wat-rs/`**
- Branch: `arc-170-gap-j-v5-deadlock-state` (already current)
- DO NOT commit. Orchestrator commits after independent scoring.
- DO NOT touch holon-rs (substrate settled; this is wat-rs-only).
- DO NOT touch wat-edn.
- **HARD CUT — defclass DELETED, not aliased.** No `pub use defclass = defrecord` shims.

## BASH DISCIPLINE

- ONE cargo command at a time, foreground; no piping; no concurrent runs
- 5 known signal-handler test hangs (task #413) — skip per Verification

## Pre-flight verified (orchestrator-grep'd 2026-05-22 night)

### Sites to rename

| Path | Sites | Action |
|---|---|---|
| `wat/holon/defclass.wat` | 8 mentions | `git mv` → `wat/holon/defrecord.wat`; edit `:wat::holon::defclass` → `:wat::holon::defrecord` (line 56); update header doc-comment (line 1) + 4 inline example references (lines 8, 35, 36, 52); update error message (line 70: `"defclass: FQDN must have at least one segment"` → `"defrecord: FQDN must have at least one segment"`) |
| `src/stdlib.rs` | 3 mentions (lines 74, 82, 83) | Edit comment (line 74); change path string (line 82); change `include_str!` argument (line 83) |
| `tests/probe_arc227_stone1_defclass.rs` | 69 mentions | `git mv` → `tests/probe_arc227_stone1_defrecord.rs`; edit ALL `:wat::holon::defclass` → `:wat::holon::defrecord` in WAT source strings + edit test fn names `probe_defclass_*` → `probe_defrecord_*` + edit header doc-comments |
| `docs/arc/2026/05/227-user-defined-types-classifier-wrap/SCORE-STONE-227.1.md` | 1+ mentions | APPEND a "Stone 227.1b rename addendum" section at end (per `feedback_inscription_immutable` — do NOT rewrite the body; ADD a note that defclass renamed to defrecord post-ship; cite commit) |

### Sites to LEAVE ALONE (historical references)

| Path | Why keep "defclass" mention |
|---|---|
| `docs/arc/2026/05/232-defprotocol-extend-type/DESIGN.md` | Historical reference in "Origin" section quoting user's 2026-05-22 dialogue — preserves the realization context; do NOT rewrite |
| `docs/arc/2026/05/227-user-defined-types-classifier-wrap/STONE-227.2-NOTES.md` | Future-stone notes; reference defclass as the predecessor name; leave intact (the notes already reflect the rename direction in body) |
| `docs/arc/2026/05/227-user-defined-types-classifier-wrap/BRIEF-STONE-227.1.md` | Historical BRIEF — immutable historical record per `feedback_inscription_immutable` |
| `docs/arc/2026/05/227-user-defined-types-classifier-wrap/EXPECTATIONS-STONE-227.1.md` | Same — historical immutable |
| `docs/arc/2026/05/170-program-entry-points/INTERSTITIAL-CLIFFNOTES.md` | If it mentions defclass in a historical Currently section, leave it; Currently is refactorable but past Currently entries marked SUPERSEDED stay |

The CLIFFNOTES Currently section (current head) can be refreshed by orchestrator post-rename if needed; sonnet does NOT touch CLIFFNOTES.

## Your scope (sonnet)

### Phase 1 — Rename wat-level macro file + verb

```
git mv wat/holon/defclass.wat wat/holon/defrecord.wat
```

Edit `wat/holon/defrecord.wat`:
- Line 1 doc-comment: `:wat::holon::defclass — arc 227 stone 227.1.` → `:wat::holon::defrecord — arc 227 stone 227.1b.`
- Line 8 example: `(:wat::holon::defclass :myapp::Voltage)` → `(:wat::holon::defrecord :myapp::Voltage)`
- Lines 35, 36 examples: `(:defclass :appA::Voltage)` → `(:defrecord :appA::Voltage)` (etc.)
- Line 52 doc-comment: `Single-arg defclass only` → `Single-arg defrecord only`
- Line 56 defmacro head: `(:wat::core::defmacro (:wat::holon::defclass ...))` → `(:wat::core::defmacro (:wat::holon::defrecord ...))`
- Line 70 error message: `"defclass: FQDN must have at least one segment"` → `"defrecord: FQDN must have at least one segment"`

(Verify ALL mentions via `grep -n "defclass" wat/holon/defrecord.wat` post-edit — should return 0.)

### Phase 2 — Update src/stdlib.rs

In `src/stdlib.rs`:
- Line 74 comment: `Arc 227 Stone 227.1 — :wat::holon::defclass macro.` → `Arc 227 Stone 227.1 — :wat::holon::defrecord macro (renamed from defclass per Stone 227.1b).`
- Line 82 path: `"wat/holon/defclass.wat"` → `"wat/holon/defrecord.wat"`
- Line 83 include_str: `include_str!("../wat/holon/defclass.wat")` → `include_str!("../wat/holon/defrecord.wat")`

### Phase 3 — Rename + edit probe test file

```
git mv tests/probe_arc227_stone1_defclass.rs tests/probe_arc227_stone1_defrecord.rs
```

Edit `tests/probe_arc227_stone1_defrecord.rs`:
- ALL `:wat::holon::defclass` → `:wat::holon::defrecord` in WAT source strings (~69 sites; mostly in `r#"..."#` test bodies)
- ALL test fn names `probe_defclass_*` → `probe_defrecord_*`
- Header doc-comment `:wat::holon::defclass macro` → `:wat::holon::defrecord macro` (top of file)
- Any other `defclass` mention in comments/strings → `defrecord`

(Verify ALL mentions via `grep -n "defclass" tests/probe_arc227_stone1_defrecord.rs` post-edit — should return 0.)

### Phase 4 — Append rename note to SCORE doc (HISTORICAL-PRESERVING)

Per `feedback_inscription_immutable`: DO NOT rewrite SCORE-STONE-227.1.md. APPEND a new section at the END:

```markdown
## Addendum 2026-05-22 night — Stone 227.1b rename (defclass → defrecord)

Per user direction post-ship: defclass renamed to defrecord. Rationale: "class" implies methods + mutable state; "record" is honest about immutable data-only. Locks the honest name before arc 232 (defprotocol) builds on it.

- Macro: `:wat::holon::defclass` → `:wat::holon::defrecord` (HARD CUT — no alias)
- File: `wat/holon/defclass.wat` → `wat/holon/defrecord.wat`
- Probe: `tests/probe_arc227_stone1_defclass.rs` → `tests/probe_arc227_stone1_defrecord.rs`
- Commit: [TBD by orchestrator]

This SCORE doc's body above remains unchanged as historical record per `feedback_inscription_immutable`.
```

### Phase 5 — Verification

Run each ONE AT A TIME, foreground:

```
cargo build --release -p wat
cargo test --release --lib -p wat -- --skip reset_sighup --skip reset_sigusr1 --skip sigusr1_query --skip sigusr2_and_sighup --skip user_signal_predicates --skip reset_sigusr2
cargo test --release --test probe_arc227_stone1_defrecord
cargo test --release --test probe_arc226_stone1_type_predicates
cargo test --release --test probe_arc216_stone1_hashset_roundtrip
cargo test --release --test probe_arc216_stone2_vector_roundtrip
cargo test --release --test probe_arc216_stone3_hashmap_roundtrip
cargo test --release --test probe_arc216_stone4_predicate_composition
cargo test --release --test probe_arc216_stone7_tuple_roundtrip
cargo test --release --test wat_arc221_keyword_nil_tag_atomization
cargo test --release --test wat_arc143_manipulation
cargo test --release --test mvp_end_to_end
cargo test --release -p wat-edn
cargo clippy --release --all-targets -p wat-edn -- -D warnings
```

All must complete cleanly.

**Holon-rs untouched** — `git -C /home/watmin/work/holon/holon-rs/ diff --name-only` empty.

**Post-rename verification:**

```
grep -rn "defclass" --include="*.wat" --include="*.rs" .
```

Should return ZERO live-code matches (only doc/comment references in historical artifacts per "Sites to LEAVE ALONE" table above).

**Write `wat-rs/docs/arc/2026/05/227-user-defined-types-classifier-wrap/SCORE-STONE-227.1b.md`** mirroring SCORE-STONE-226.1.md shape (small; 8-row scorecard at most).

## STOP triggers

- **STOP-1 (compile error UNEXPECTED):** STOP and report
- **STOP-2 (test failure beyond rename consequences):** STOP + diagnose
- **STOP-3 (90 min elapsed):** wall-clock STOP
- **STOP-4 (holon-rs touched accidentally):** STOP and report
- **STOP-5 (alias added):** if you find yourself adding `pub use defclass = defrecord` or similar shim, STOP — HARD CUT discipline
- **STOP-6 (historical artifact rewritten):** if you find yourself editing BRIEF-STONE-227.1.md / EXPECTATIONS-STONE-227.1.md / SCORE-STONE-227.1.md body / STONE-227.2-NOTES.md (any pre-existing artifact) instead of leaving it alone OR appending a new section, STOP — `feedback_inscription_immutable`
- **STOP-7 (bash discipline):** cargo hang from accidental pipes

## Out-of-scope

- Multi-field defrecord support (Stone 227.2)
- defprotocol / extend-type (arc 232)
- Doctrine memory entry inscription (orchestrator-side; lands separately)
- CLIFFNOTES Currently update (orchestrator-side; lands separately)
- holon-rs / wat-edn changes
- Any defclass alias (HARD CUT)

## Doctrine context

Stone 227.1b is paperwork-rename, not new substrate. The substance was decided in dialogue 2026-05-22 night:

```
defservice (arc 209)   = wrap mutable state in mutex
defrecord (arc 227)    = wrap immutable data in classifier
defprotocol (arc 232)  = polymorphic dispatch over types (future)
```

The honest naming makes the future arc map readable.
