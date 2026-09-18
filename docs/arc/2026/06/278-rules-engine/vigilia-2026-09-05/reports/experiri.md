# ward `experiri` — vigilia 2026-09-05

> Cast at HEAD `21530efab`, branch `grok-rete`. Read-only unless the ward says otherwise.
> **Verbatim ward return, preserved unedited.** The 2026-08-30 cast lost its 19 returns
> because they lived only as subagent messages; this file exists so that cannot recur.
> Nothing here is adjudicated — see `../WORK-LIST.md` for what the orchestrator credited.

---

## VIGILIA — `experiri` cast on `wat-rs` @ `21530efab`

**CALIBRATION: PASSED.** Four calibration drives across two independent surfaces, two fire and two refuse:
- `probe_vig_value_hash_collision::calibration` — equal `Vec` hashes equal (fire ×2), differing element changes the hash (refuse ×2). PASS.
- Phantom-head grid — `p0` (`:wat::core::+`, forced) → `Ok(i64(3))` **fire**; `p5` (`:vph::…`, forced) → `LOAD REFUSED — UnresolvedReference … "call head — not a builtin, not a registered function"` **refuse**, and the refusal names the thing under test.
- Probe 1 carries its own in-fixture non-vacuity control (`OutP`), which **passed**: the no-guard chain over the *same* data returns 2, so the fixture provably reaches a second round with a non-empty Δright against a non-empty `old_left`.

**Tree state on exit:** `git status --porcelain` empty. Every probe file removed (copies below). Every `src/` mutation reverted by file restore, rebuilt (2m42s), and the four gates I touched re-run: `4 tests run: 4 passed`. No floor test was left red by me.

---

## L1 — driven defects, ranked by blast radius at the public surface

### L1-1 · A query silently loses rows. `:where` + ≥2 fact conditions + a right fact derived one round later for an already-seen key.

**Driven:**
```
cargo nextest run --release -p wat --test rete -E 'test(probe_vig_left_idx_latch)' --no-capture
```
```
PROBE REPORT (deliberate): native [OutW=1 OutP=2 C=2 OutN=2] oracle [OutW=2 OutP=2 C=2 OutN=2]

assertion `left == right` failed: native and $oracle disagree.
native=[OutW=1,OutP=2,C=2,OutN=2] oracle=[OutW=2,OutP=2,C=2,OutN=2]
```
`OutW` is the guarded chain: **native 1, oracle 2**. `OutP` (same three facts, no guard) is 2 on both — the round-2 arm really is reached. `C=2` — the derived right fact really did arrive a round late. This is the same class as the bug `probe_arc278_where_is_positionally_free` gates: it compiles, runs, exits 0, and returns a short answer.

**Mutation proof (predicted mechanism, driven):** seeding `left_idx[J]` from `wm.beta[parent]` immediately before `hj_step4_term2` (`hash_join.rs:358`) moves native to `[2,2,2]` and nothing else moves. So the missing `left_idx[J]` at step 4 is exactly why `term2 = old_left ⋈ Δright` is skipped.

**The mechanism — and `solvere`/`sequi` named the wrong writer.** `join_keys_cache` (`session.rs:168`) doubles as the latch at `hash_join.rs:120`; the first-keying catch-up it guards is the only bulk builder of `left_idx` (`hash_join.rs:273`). The second writer is **not** `keyed_join_persistent` reached from `join_after_filter` (pass 3.6) — those joins have a *filter* parent, `kind_ids.join_parent` is `RootJoin | HashJoin` only (`arm.rs:542`), so pass 3 never visits them and no collision is possible there. The collision is **`left_activate_join` (`fire/pass/mod.rs:107`) called from `filter_after_join` at `filter_after_join.rs:75`** — a HashJoin child of a *frontier HashJoin*, which therefore **is** in `join_parent`. It writes `join_keys_cache[J]` (via `keyed_join_persistent`, `fire/mod.rs:791`) and never touches `left_idx`. Next round `first_keying` is false, the catch-up is skipped, and `left_idx.get(child_id)` at `hash_join.rs:429` is a silent `None`.

Ladder (extirpare): the top rung is available — `join_keys_cache` and `left_idx` are two maps that must agree, exactly the shape D2 already cured for `buckets`/`indexed_n`. Make the join's *left* index and its key list one type whose only door builds both, so a writer that sets the keys without the index cannot be written.

### L1-2 · `fire-rules-explain$oracle` attributes derived facts **nondeterministically**, and its own doc says it matches the native.

**Driven, 8 samples** (`probe_vig_explain_order`, 8 rules producing the same `Out`, 1 rule producing the control `Solo`):
```
Out native="vex::aaa" oracle="vex::ggg" | CONTROL Solo native="vex::solo" oracle="vex::solo"
Out native="vex::aaa" oracle="vex::zzz" | ...
Out native="vex::aaa" oracle="vex::aaa" | ...
Out native="vex::aaa" oracle="vex::zzz" | ...
Out native="vex::aaa" oracle="vex::ggg" | ...
Out native="vex::aaa" oracle="vex::ccc" | ...
Out native="vex::aaa" oracle="vex::ggg" | ...
Out native="vex::aaa" oracle="vex::aaa" | ...
```
Native: stable `vex::aaa`, 8/8. Oracle: **four distinct answers** (`ggg`, `zzz`, `aaa`, `ccc`), agreeing with native in 2/8. The single-producer control is stable on both 8/8, so this is not a harness artefact.

`harvest-support` (`wat/rete/oracle/explain.wat:10–49`) folds over `(:wat::core::PersistentMap/keys network)` — HAMT order — with no `sort`, and is first-producer-wins. Its sibling `fire-once$oracle` (`wat/rete/oracle/fire.wat:157`) sorts and says in writing *"oracle-derived changed every run, sometimes []. Native sorts (`sorted_node_ids`); the spec must too."* That law was never applied to `explain.wat`, whose header at `:53` asserts *"First-producer-wins, matching the native index."* It does not match, and it does not match itself twice running.

Blast radius: `fire-rules-explain$oracle` is the **referee** for explain. `probe_arc278_P12a`'s oracle row compares only `PersistentMap/length`, which is invariant under this. `conferre` was right that the differential is length-only; it is worse than a stable disagreement — the reference is not a function.

Ladder: one `sort` in `harvest-support` is the check rung; the shape rung is a shared `topological-node-ids` verb both walkers must call, so a walk over raw `PersistentMap/keys` has no form in this file.

### L1-3 · `impl Hash for Value` collides *structurally*, not probabilistically — two witnesses, and the collision survives into `wat__std__HashSet`.

**Driven** (`probe_vig_value_hash_collision`):
```
nested_empty_vectors_do_not_collide  FAILED
  assertion `left != right` failed: STRUCTURAL COLLISION: hash_sequence writes no length
  and no terminator, so Vec[Vec[],Vec[1]] and Vec[Vec[Vec[],1]] emit identical write
  streams. left=0x13e79b5f72cca234 right=0x13e79b5f72cca234

a_shifted_nesting_boundary_does_not_collide  FAILED
  STRUCTURAL COLLISION #2: left=0xd8bba23a5fb7e60b right=0xd8bba23a5fb7e60b
  ( Vec[Vec[1], 2]  vs  Vec[Vec[1,2]] )

the_collision_reaches_hashset  FAILED
  the element collision propagated into the HashSet arm:
  0x50e1ab376c2feba4 == 0x50e1ab376c2feba4
```
All three pairs are `assert_ne!`-unequal under `PartialEq` first, so these are genuinely distinct values. `impl Hash for Value` (`value/value.rs:751`) early-returns `Vec`/`List` at `:759` into `hash_sequence` (`:556`), which writes `SEQ_TAG` and each element and **no length, no terminator**. The doc above it (`:553`) claims *"correctness rests on the full 64-bit hash output (collision ~1/2^64)"*. That claim is false: these are constructions, not lottery tickets.

**Blast radius, established rather than reasoned.** I chased the three channels a hash could turn into a wrong answer and all three are clear: the rete's `JoinKey` (`session.rs:197`) is interned filler *ids*, not a `Value` hash; `seen_insert` (`fire/delta.rs:194`) keys on `Arc` identity for aggregates and on `FxHashSet<Value>` (Hash **+ Eq**) otherwise; content addressing goes through `src/hash.rs`. So the live consequence is **degenerate bucketing in `wat__std__HashSet`/`HashMap` over nested sequences, plus a false documented guarantee** — the `HashSet` arm hashes element hash *values*, so it amplifies the collision one level up rather than absorbing it. Not a wrong answer today; a wrong answer the moment anything treats a `Value` hash as an identity.

Ladder: `hash_sequence` should write the length (or a terminator byte) — one line, and it moves the class from "structurally constructible" to "1/2^64", which is what the doc already promises.

---

## L2

### L2-1 · The `:wat::` vocabulary is **open at load**. Every position accepted; the only refusal is at execution.

Grid, all seven cells driven through `startup_from_file` + `apply_function`:

| cell | result |
|---|---|
| `p0` real head, forced (calibration fire) | LOADED, `Ok(i64(3))` |
| `p5` `:vph::PHANTOM`, forced (calibration refuse) | **LOAD REFUSED** — `UnresolvedReference … "call head — not a builtin, not a registered function"` |
| `p1` `:wat::core::PHANTOM`, unforced `defn` body | **LOADED**, `Ok(i64(7))` |
| `p2` `:wat::core::PHANTOM`, forced | **LOADED**, then `Err(UnknownFunction { :path ":wat::core::VIGILIA-NEVER-EXISTED-ANYWHERE" })` |
| `p6` `:wat::kernel::PHANTOM`, unforced | **LOADED**, `Ok(i64(7))` |
| `p3` `:wat::kernel::abort`, arm **taken** | **LOADED**, then `Err(UnknownFunction { :path ":wat::kernel::abort" })` |
| `p4` `:wat::kernel::abort`, arm **not taken** | **LOADED**, `Ok(i64(5))` |

So `cernere`'s one-line probe answers: **no position rejects a `:wat::`-prefixed phantom at load** — not resolve (`resolve/walk.rs:268`, `is_reserved_prefix` returns `true` for the whole `:wat::` root, `resolve/reserved.rs:14`), not the checker (`check.rs:4884` for `kernel`/`std`, `check.rs:5558`'s `fresh.fresh()` fallback, `check.rs:5585`'s `!k.starts_with(":wat::")` exemption). The refusal exists only at runtime dispatch. `cernere`'s two shipped phantoms are real: `:wat::kernel::abort` in five `tests/reflection/*.wat`, `:wat::kernel::panic!` at `wat-scripts/scratch-pad/arc109-type-equal-acceptance.wat:16`.

**Correcting the ward's severity:** in the reflection fixtures the phantom sits in the `Err`/`None` reporting arm, and `p3` shows that arm does **not** go silent — it raises `UnknownFunction: :wat::kernel::abort` instead of aborting with the message. So those fixtures still fail when they should; they just report the wrong reason, and are green only because `p4`'s arm is the one taken. That is a diagnostic defect, not a false green.

### L2-2 · `record_token`'s "a future site cannot push without counting" is false at four sites — and the census gate cannot reach them.

The claim at `fire/pass/mod.rs:27`. Still open-coding push-and-count: **`fire/pass/mod.rs:151`** (in the file that makes the claim, 124 lines below it), and **`fire/mod.rs:2070`, `:2080`, `:2104`**. `struere` read this correctly; here is the driven half.

Discriminating mutation pair, same 100-test gate set both times (`beta_write_read_traffic`, all `*_cost`, all `census`, `round_census`, `node_share`, `rank_and_instrument`):

- **M2** — drop `beta_written` at the open-coded `left_activate_join` site (i.e. write the exact future site the doc says cannot be written): `Summary [20.272s] 100 tests run: 100 passed`. **Invisible.**
- **M1** (control) — drop the census *inside* `record_token`/`record_tokens`: `Summary [20.103s] 100 tests run: 99 passed, 1 failed`, `fanout_cost::beta_write_read_traffic` RED at `fanout_cost.rs:100` with `recorded no beta writes — the instrument is not armed` and a table showing `node 1: written 0, read 2001`.

So the gate is real and it works — for sites the helper covers. The three census worlds (`fanout`, `cascade`, `tri`) contain no `:where`, and all four open-coded sites live behind a filter parent, so no world in that gate can reach them. `struere`'s unpaid claim is **confirmed**.

### L2-3 · `retract` removes *every* equal fact where `insert` stages one.

**Driven** (`probe_vig_retract_multiplicity`), two identical `(:vrm::F :k 1)` inserts plus one `(:vrm::G :k 1)`:
```
facts_after_two_identical_inserts=3
facts_after_one_retract=1
seen_rows_before_retract=1
seen_rows_after_retract_and_refire=0
```
Insert preserves multiplicity (3 staged facts). **One** `retract` drops **both** copies — `wat/rete/oracle/insert.wat:100`'s `foldl` keeps `f` only when `(not (= f fact))`. The derived consequence disappears with it. `conferre` is confirmed. (Ranked L2, not L1: the retract docstring says "by value equality", so this is an asymmetry the code documents, not a divergence from its own spec.)

### L2-4 · `reachability.rs:1661–1665` — a discrimination row that has never executed.

`probare` is confirmed, twice over. Simulated first: for both loop iterations the `replacen` pattern (`"::= :v :beta"`, `"::= :v :probe::E::B"`) is **ABSENT** from both source literals — it targets the *miss face* of the constant, which appears in the fact `insert`, never after `::=`. Then driven: replacing the silent `if never != *src {` at `:1660` with the `assert_ne!` its two siblings at `:1328` and `:1654` already use goes **RED on the first iteration**:
```
thread 'rete::reachability::a_keyword_operand_is_a_field_ref_or_a_constant_by_one_rule'
panicked at src/rete/reachability.rs:1660:9:
assertion `left != right` failed
  left: "(:wat::core::defrecord :probe::In …" 
 right: "(:wat::core::defrecord :probe::In …"   (byte-identical)
```
The `if` converts "the rewrite didn't happen" into silent success, so the row reads as covered.

---

## Refuted — leads I drove and found FALSE

- **The named second writer in probe 1 is not the one that can collide.** `join_after_filter` (pass 3.6) only ever visits HashJoins whose parent is a Test/Negation/Exists/Accumulate; `kind_ids.join_parent` is `RootJoin | HashJoin` (`arm.rs:542`), so pass 3 never visits those joins and `left_idx` is irrelevant for them. The colliding writer is `left_activate_join` from pass **3.7**. The defect is real; the citation was wrong.
- **The `:not` variant of probe 1 does not reproduce.** I reasoned from `filter_or_acc` = Test|Negation|Exists|Accumulate that any filter would do. Driven: `[A3 ?k] (:not (:vlx::Neg ?k)) [B ?k] [C ?k ?v]` gives `OutN=2` on **both** engines. Only the `:where` cell is confirmed. My generalisation was wrong.
- **Probe 5 — D2's chain-row observable is currently correct.** `./target/release/wat wat-scripts/scratch-pad/d2-derived-fact-axis.wat` (binary rebuilt at HEAD) → `native Hit=12 Hit2=6 chain-rows=12` / `oracle Hit=12 Hit2=6 chain-rows=12`. The 18-vs-12 that caught D2 does not reproduce; the cure holds. But the file **is** blind to L1-1, and by construction: its wave-2 derived facts all carry *new* keys, so `old_left` is empty for every Δright and `term2` has nothing to lose. Same blindness the wards named in `probe_arc278_where_is_positionally_free`. A floor-resident observer for the D2 axis therefore needs the *key-reuse* stagger my probe 1 fixture supplies, not d2p's fresh-key stagger.
- **Probe 6 at two producers is invisible.** With `aaa`/`zzz` only, native and oracle agreed (`vex::aaa`/`vex::aaa`) and my assertion passed. It took eight producers to shift HAMT order off ascending-id order. A single sample of this probe is worthless.

---

## What I could not check, and why

- **`:exists` and accumulate as the filter parent in L1-1.** `:where` confirmed, `:not` refuted. I did not drive the other two; each is one more rule in the same fixture and one re-run (the `.wat` is read at runtime, so no rebuild).
- **Phantom heads in non-call positions** — type position, pattern position, macro-head position. I drove only the call-head position, forced and unforced, under two `:wat::` sub-namespaces. `position-not-modelled`.
- **The hash collision's reach into user programs.** I established the three rete channels are clear by reading `JoinKey`, `seen_insert` and `src/hash.rs`; I did **not** drive a user program that puts nested sequences in a `wat__std__HashMap` key.
- **The full floor was not re-run.** Everything I added is removed and `git status` is empty, so the floor is unchanged by construction; but the last number I can quote is the brief's `5420 run / 21 skipped`, not one I measured. The four gates I mutated are green at exit.
- **L1-2's sample is 8 processes on one machine.** I did not establish whether the HAMT order is seeded per-process by an ASLR-derived hasher or by something else — only that it varies per process and the native does not.

**Probe files** (removed from the tree, kept intact for landing) at
`/tmp/claude-1000/-home-john-work-holon/fd01e281-0457-4e4a-a481-acd7beca46ad/scratchpad/probes/` — 15 files. They report through deliberate `panic!`s so a bare run prints the numbers; the assertion-style tests beside them (`native_agrees_with_the_oracle_on_the_guarded_chain`, `native_and_oracle_attribute_the_same_rule`, the three `assert_ne!` hash rows) are the landable halves. Raw logs are in the same scratchpad as `run1`–`run9`.
