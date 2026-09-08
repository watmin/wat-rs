# SEQUI — Cast Report (TARGET 4 — the test corpus) — **FINDINGS**

> Returned verbatim, before synthesis. Orchestrator verification follows the ward's text.

## Re-derivation of handed-down numbers

Anchored against known-positives before trusting any zero (all commands run directly against `tests/rete/` + `src/rete/kernel/tests/`):

- File/line counts: **306 files / 38,058 lines** — confirmed exactly.
- `Mutex`, `RwLock`, `RefCell`, `thread_local!`, `static mut`, `OnceLock`, `OnceCell`, `lazy_static`, `par_iter`, `AtomicUsize`, `env::set_var`, `env::var`, `set_current_dir`, `current_dir` — reconfirmed **0 files** each, per-pattern (not combined).
- `Arc<` — 7 hits / 5 files — confirmed exactly (harvest_cost.rs ×2, where_tree_branch_differential.rs ×1, alpha_discrimination.rs ×1 [comment], accum_cost.rs ×1, accum_alpha_cost.rs ×2). All are plain value types (`Arc<[(Value,Value)]>`, `Arc<InternedNetwork>` return type, `Arc<str>`) — none wrap `Mutex`/`RefCell`/`Cell`, so none are ambient-mutation channels.
- `thread::spawn` — 1/1, confirmed (`arm_lease.rs:59`).
- `ARM_BUILDS` — 22 hits / 2 files, confirmed exactly (arm_lease.rs 19, cascade_cost.rs 3).
- `&mut` in fn signatures — exactly 4, confirmed by direct read of each site: `collect_wat(dir, out: &mut Vec<PathBuf>)`, `poke_first_call_op(v: &mut Value, ...)`, `poke_first_prog_root(v: &mut Value, f: &mut dyn FnMut...)`, `scan_into(out: &mut Vec<PMap>, ...)`. All four are honest visible-mutation out-params — no finding.
- `let mut` — 569, confirmed exactly.
- **Delta found:** `AtomicU64` — handed down as "4 hits/2 files." Direct grep across the target finds **3 code hits, all local (not global) in `harvest_cost.rs`**, plus 3 *prose* mentions in `probe_arc278_concurrent_retes.{wat,rs}` (2 in the `.wat`, 1 in the `.rs`). "4/2" only reproduces if the `.wat` prose hit is excluded and one `.rs` prose hit counted — a real but harmless discrepancy since none of it is load-bearing (harvest_cost.rs's `AtomicU64` is a function-local micro-benchmark instrument fully scoped inside a closure, not shared state).
- **Delta found:** `#[test]` count — handed down as 613. Direct count is **618** (493 in `tests/rete` + 125 in `src/rete/kernel/tests`). Not load-bearing for any finding below.
- `rune:sequi` in target — reconfirmed **0**, case-insensitive too.
- No `utils.rs`/`helpers.rs`/`common.rs`/`misc.rs` in either tree — reconfirmed.

## Verified negatives (checked, found clean)

- **`arm_lease.rs:53-91` (`intern_index_thread_owned_workers_do_not_collide`)** — the one `thread::spawn` site. Each spawned closure owns its own `world`/`fired` via `fire_cascade`; nothing crosses thread boundaries except joined return values. This is the honest shape (the test's entire point is proving the thread-local `ARM_TABLE` doesn't collide across threads). Clean.
- **`fire_cascade`, `session_with_facts`, `session_facts`** (the corpus's core fixture-composition helpers, `src/rete/kernel/tests/mod.rs:208`, `src/rete/kernel/session.rs:1488,1552`) — all take and return explicit `Value`/`(FrozenWorld, Value)`. No interior mutability, no ambient reach. Clean (duplication-of-shape is 4S1's territory, not sequi's).
- **`probe_arc278_import_accounting.rs`, ARM 1 and ARM 2** (`import_refuses_a_build_that_outgrows_the_session_ceiling`, `import_refuses_a_node_count_past_the_cap`) — ARM 1 crosses a subprocess boundary (`run()`); stdout/stderr *is* the visibly-threaded state. ARM 2 threads an explicit `world: &FrozenWorld` handle through `import_one`/`poke_named`. Both honest chains. Clean.
- **`cascade_cost.rs:348-389`** — reads `ARM_BUILDS` before/after a single bracketed call (`cascade_phase_census`) purely to print it in a report table; never gates pass/fail. Distinct from `arm_lease.rs`'s usage (below) and already inside 4S1/4S2/4L1's territory, not a new sequi finding.

## Ruling on Question 1 (the `performance-counter` rune with no measurement, `arm.rs:727`)

**Distinct from Q2, and reachable from target 4 — I row it, scoped to the target-side consumption, not the rune's own text.**

- `ARM_TABLE` (Q2, `arm.rs:705`) and `ARM_BUILDS` (`arm.rs:727`) are two different statics with two different rune categories on two different lines — not the same finding restated.
- Per the ward's own text, `performance-counter` requires "cite the measurement" — verified true of its two in-subsystem siblings (`census.rs:618` *"~75-80ns... measured 2026-08-01 against a no-sub-marks control"*, `:673` *"Cell increment ~1-2ns vs timer tax"*). `arm.rs:727`'s reason — *"test-only intern-miss count; not fire domain"* — states what the counter is, not why hiding it is justified by a performance cost. No citation exists.
- But the sharper issue than the missing citation is that **ARM_BUILDS is not merely a metric here** — in the five `arm_lease.rs` tests, it is the sole oracle deciding pass/fail for a domain claim ("did fire rebuild the arm"), not an aside measurement `performance-counter`'s own definition says "cannot change a result." That tension is argued fully in Finding B below, which is where I land the row (on the target-side consumption, since `arm.rs:727` itself is context, not target).

## Ruling on Question 2 (`SEQUI_CATEGORIES` "five entries")

**Refuted.** I read `tests/lint/no_unknown_sequi_rune.rs:31-36` directly (not grep):
```rust
const SEQUI_CATEGORIES: &[&str] = &[
    "ambient-context",
    "performance-counter",
    "host-idiom",
    "reclassified-by-caller",
];
```
That is **four** entries, matching the violation message's *"is not one of the four categories"* at `:104` exactly. There is no count-drift bug. The handed-down premise is false — this is one more of the eighteen. (Separately, `docs/CONVENTIONS.md:1054-1070` documents this same four as the repo's *local* vocabulary, substituting `reclassified-by-caller` for the ward's own `legacy-shape` — a deliberate local convention with its own rationale table, not an uncounted drift.)

On scope (b): moot given (a) is false, but for the record — `no_unknown_sequi_rune.rs`'s `collect_rs` walks all of `src/`, not `tests/rete/`. Unlike 4P1's precedent (a lint whose entire subject *is* fixtures physically inside `tests/rete/`), this lint's subject is the codebase-wide rune vocabulary; it only touches the target's `src/rete/kernel/tests/` slice as an incidental byproduct of walking all of `src/`. Since the target carries zero `rune:sequi` markers (confirmed above), this lint has zero live effect on target-4 content today regardless of the scope ruling.

## Finding A (sharpest, new) — unmarked ambient thread-local state, no rune anywhere

**File:** `tests/rete/probe_arc278_import_accounting.rs:177-215`, fn `an_origin_already_filed_is_never_re_based`

Chain:
1. `:191-192` — allocate + `black_box` a 1 MiB ballast (keeps it live).
2. `:193` — `late = wat::alloc_counter::thread_bytes()` — reads the ambient `THREAD_LIVE: Cell<usize>` thread-local (`src/alloc_counter.rs:87-114`, read for context).
3. `:201` — `wat::alloc_counter::mark_session_origin_at(key, 0)` — writes the ambient `SESSION_ORIGINS` thread-local map. Signature is `fn(SessionOriginKey, usize)` — returns nothing; the mutation is invisible in the type.
4. `:204` — same call again with `late`, relying on `or_insert` semantics to be ignored.
5. `:206` — `used = wat::alloc_counter::session_bytes(key)` — reads the *same* ambient state (plus a `LAST_ORIGIN` cache), combined with `thread_bytes()` read at call time.
6. `:207-212` — `assert!(used >= 1<<20, ...)`.

Where the thread breaks: every one of steps 2–5 coordinates through `src/alloc_counter.rs`'s three `thread_local!` cells, reached by free functions whose signatures carry none of that state — this is the spell's own "Hidden state via global counter" example, verbatim in shape (`mark_session_origin_at` ≅ the spell's `next_id`). The test's author was visibly aware of the risk: the comment at `:178-180` defends against **key** collision ("a key no `PMap` will ever mint... colliding with a live session would make the reading below measure that session instead") — but that defense is reasoning about an untyped invariant, not something the type checker enforces, and it says nothing about `thread_bytes()` itself being shared thread-wide state that any other code touching this OS thread would also perturb.

Zero `rune:sequi` exists anywhere in this file or the whole target (confirmed above) — this hidden-state read/write is entirely unacknowledged, unlike `ARM_BUILDS`/`ARM_TABLE` which at least carry a (contested) rune at their definition site.

Recommendation: **thread explicitly, or rune it.** Since `alloc_counter.rs` is out-of-target for editing, the actionable move on the target side is a `rune:sequi(ambient-context)` (or a category the repo mints) at this call site, naming the invariant the comment at `:178-180` already argues informally — that no other code path touching this thread's byte counter runs between steps 2 and 5. Absent that, this reads as the exact unmarked case the spell says "has no rune category by design... thread explicitly or restructure into a service-with-handle."

## Finding B — the `ARM_BUILDS` chain's rewireability (answers 4D1's open question)

**File:** `src/rete/kernel/tests/arm_lease.rs`, five chains: `:12-48`, `:99-133`, `:137-161`, `:167-195`, `:259-291`.

Each chain does: call something that fires (`fire_fixpoint_delta`/`with-overlay`, itself `Result<Value, EvalBreak>` — its signature reveals *nothing* about whether an internal arm rebuild happened) → snapshot `ARM_BUILDS.load(Relaxed)` before → call again → snapshot after → `assert_eq!`/`assert!` on the delta. Example, `:110-116`:
```rust
let builds = ARM_BUILDS.load(std::sync::atomic::Ordering::Relaxed);
fire_fixpoint_delta(&fired, world.symbols(), None).expect("second fire HIT");
assert_eq!(
    ARM_BUILDS.load(std::sync::atomic::Ordering::Relaxed),
    builds,
    "fire HIT must not rebuild"
);
```

This is 4D1's own open question, answered: **the chain does not thread its state visibly — the counter's ambient read/write is what makes the test's steps un-reorderable and un-composable.** Nothing in `fire_fixpoint_delta`'s signature says "and here is whether I rebuilt"; the only channel is a global the test peeks at outside any call's own return value. The correctness of attributing a given delta to a *specific* call in the chain is an invariant enforced purely by the reader's trust that nothing else in the chain (`fire_cascade`, `session_with_facts`, a future inserted setup call) also touches the same counter — nothing here is type-checked. A maintainer inserting a new step between two `ARM_BUILDS.load()` calls, or reordering the fire/overlay calls, gets no compiler help; a wrong reorder silently breaks the attribution and the test would still compile.

This differs from 4D1 (process-fork race-safety) on a different axis (this holds even single-threaded, single-process) and from Q1/Q2 above (this is the *consumption*-side finding on the target, not the rune-text finding at the out-of-target definition). It also sharpens Question 1: the rune's own category (`performance-counter`, "arming it cannot change a result, only a measurement" per `CONVENTIONS.md:1063`) is in tension with how these five tests actually use the counter — as the load-bearing oracle for a correctness claim (arm-reuse semantics), not a side measurement.

Recommendation: thread the build-vs-hit outcome explicitly — e.g., have `fire_fixpoint_delta_armed`/a test-only variant return `(Value, bool)` (rebuilt-or-not) so these five tests assert on a value the call itself hands back, rather than diffing a global the call's signature says nothing about. Short of that, the rune at `arm.rs:727` should at minimum cite why threading a bool out of a hot production path is unjustified (matching its siblings' citation bar), and the target files should each carry their own `rune:sequi` acknowledging the ambient-read idiom, since right now zero exist anywhere in the 306-file target.

## Ranking

1. **Finding A** (`probe_arc278_import_accounting.rs`) — sharpest: a completely unacknowledged hidden-global chain, in the target itself, matching the spell's canonical example almost exactly, with zero rune coverage anywhere.
2. **Finding B** (`arm_lease.rs`) — answers a question a sibling ward explicitly left open; real, but partially mitigated by an (imperfect) rune at its definition site.
3. **Q1 / Q2 rulings** — one real (narrow) defect confirmed distinct from prior art, one handed-down premise refuted outright.

FINDINGS

---

# ⚖ ORCHESTRATOR VERIFICATION — my own read of the disk, this session

## ⛔ QUESTION 2 IS REFUTED AND THE ERROR WAS MINE — A NINETEENTH WRONG FIGURE, AND THE FIRST THAT WAS A PLAIN MISCOUNT

`SEQUI_CATEGORIES` holds **four** entries. `sed -n '31,36p' | grep -c '"'` → **4**. `ambient-context`, `performance-counter`, `host-idiom`, `reclassified-by-caller`. The violation message's *"is not one of the four categories"* is **correct**, and there is no count-drift defect.

⭐ **The other eighteen were grep artifacts. This one was not.** I had the six-line block printed in front of me and read the *line range* `31-36` as the entry count. No tool misled me; I miscounted a four-item list I was looking at. **That is a distinct failure mode from every prior instance and it argues for the same cure from the other end: derive the count with a command (`grep -c`) even when — especially when — the list is short enough to eyeball.**

⭐⭐ **And this is exactly what handing the lead down AS A QUESTION bought.** Stated as a finding, it would have come back agreed and I would have rowed a defect that does not exist. `[[hand-a-lead-down-as-a-question]]`, fourth confirmed instance.

## ⛔ THE WARD'S `618` IS WRONG, AND IT IS TRAP #2 — COMMITTED WHILE CORRECTING A NUMBER

The ward reports `#[test]` = **618** (493 + 125) against the handed-down 613. I re-derived three ways:

```
grep -rhE '^\s*#\[test\]' … --include='*.rs'   → 613   (line-anchored, .rs only)
grep -rn  '#\[test\]'      … --include='*.rs'   → 615   (490 + 125, any position)
grep -rn  '#\[test\]' tests/rete                → 493   ⛔ NO --include
```

**Its 493 is `tests/rete` swept with no `--include`, which pulls in three `.wat` COMMENT lines** that use the token `#[test]` in prose: `probe_arc278_P2_native_fire_once.wat:17`, `probe_arc278_session_ceiling_second_session.wat:55`, `probe_arc278_P4a_native_fire_rules.wat:60`. 490 real + 3 prose = 493; + 125 = 618.

⛔ **Trap #2 — a grep matching PROSE ABOUT the thing — was in the ward's brief, by number, and it fired anyway, in the act of correcting someone else's count.** ⭐ **613 now stands for the SIXTH derivation.**

## ✅ ITS `AtomicU64` CORRECTION IS RIGHT AND MINE WAS THE TRAP — SAME TRAP, OPPOSITE DIRECTION, SAME EXCHANGE

`grep -rn 'AtomicU64'` over the target returns **6** lines: three in `harvest_cost.rs` (`:365` a `use`, `:399` and `:431` `AtomicU64::new(1)` — both **function-local**), and three in **prose** (`probe_arc278_concurrent_retes.wat:13,23` and `.rs:4`, all describing a production global). My "4 hits / 2 files" was `--include='*.rs'`, which excluded the two `.wat` prose lines and kept the one `.rs` prose line. **Arithmetically defensible, semantically misleading** — it implied two files carry an `AtomicU64` when in code only one does, and none is global.

⭐ **So both of us hit trap #1/#2 in the same exchange, in opposite directions — my number over-counted by including prose, its number over-counted by including prose.** The trap does not care which side of the brief you are on.

## Finding A — ✅ VERIFIED, every citation, both halves

`an_origin_already_filed_is_never_re_based` is at `:177`; the comment at `:178-180` reads verbatim as quoted; `:191-193`, `:201`, `:204`, `:206`, `:207-208` each carry exactly the step claimed. The ambient cells verify at `src/alloc_counter.rs`: `THREAD_LIVE: Cell<usize>` `:91` (read at `:115`), `SESSION_ORIGINS: RefCell<FxHashMap<…>>` `:168`, `LAST_ORIGIN: Cell<…>` `:192`. Signatures confirm the shape: **`pub fn mark_session_origin_at(key, origin)` at `:247` returns nothing** — the mutation is invisible in the type — while `thread_bytes() -> usize` (`:114`) and `session_bytes(key) -> usize` (`:272`) read ambient state.

⭐ **One piece of evidence the ward had in reach and did not use:** `alloc_counter.rs:157-159` documents the coupling itself — *"which is why `LAST_ORIGIN` sits in front of this map and carries the measurement… This map is NOT read from inside the allocator — only `THREAD_LIVE` is."* The file knows the thread-local is the live channel.

⚠ **I do not adopt the recommendation as written**, and the reason is a doctrine split recorded below.

## Finding B — ✅ VERIFIED, and its central argument is STRONGER than the ward made it

All five chains verify: `ARM_BUILDS` reads sit in the tests at `:12` (`:17`), `:99` (`:110,113,124`), `:137` (`:151,157`), `:167` (`:181,184,192`), `:259` (`:263,284`) — **five, exactly, out of nine tests in the file.** The quoted `:110-116` block reads verbatim.

⭐⭐ **THE REPO'S OWN CONVENTION TABLE CONTRADICTS THE USE OF THE EXAMPLE IT NAMES.** `docs/CONVENTIONS.md:1054-1070` defines the closed set, and its `performance-counter` row reads:

> | `performance-counter` | instrumentation, off by default | **nothing about the answer — arming it cannot change a result, only a measurement** | the fire census TLS, **`ARM_BUILDS`** |

`ARM_BUILDS` is the table's own cited example of *"arming it cannot change a result."* In `arm_lease.rs` it is the **sole oracle** of `assert_eq!(ARM_BUILDS.load(…), builds, "fire HIT must not rebuild")`. **If it stopped counting, all five assertions would pass vacuously** — `0 == 0`. The result does not merely depend on it; it is nothing but it. ⛔ **This is `Q2`'s shape recurring one line down the same file** — a rune whose own adjacent definition contradicts its use — which is the strongest reason to keep Finding B distinct from `4D1` rather than folding it in.

## ⚠ A DOCTRINE SPLIT I AM RECORDING, NOT RULING ON — the spell and this repo's vocabulary disagree

The spell says of hidden domain state: *"has no rune category **by design**… A rune would suppress the finding without justifying it; sequi refuses that suppression for the case the spell exists to catch."* This repo's `CONVENTIONS.md:1054-1070` mints exactly such a category — `ambient-context`, defined as *"real DOMAIN state, reached globally or per-thread instead of threaded"* — and names `ARM_TABLE` and `EXEC_ARENA` as its examples.

**The repo's local vocabulary licenses precisely what the spell refuses.** That is why the ward's Finding A recommendation (*"rune it as `ambient-context`"*) is correct under this repo's convention and forbidden under the spell's, and it is why `Q2` (target 1) is still an open builder decision rather than a settled row.

⛔ **I am not routing this as a finding.** `CONVENTIONS.md` is a `docs/*.md` file, and the builder's standing ruling for this cast is that `docs/*.md` is **stale by default and out of scope** — *"they will be addressed in time, not now."* It is recorded here because it changes how **every** `rune:sequi` in this repo must be read, and a later reader who takes the spell's clause as governing will misjudge them.
