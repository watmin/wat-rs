# ward `exigere` — vigilia 2026-09-05

> Cast at HEAD `21530efab`, branch `grok-rete`. Read-only unless the ward says otherwise.
> **Verbatim ward return, preserved unedited.** The 2026-08-30 cast lost its 19 returns
> because they lived only as subagent messages; this file exists so that cannot recur.
> Nothing here is adjudicated — see `../WORK-LIST.md` for what the orchestrator credited.

---

Cast complete. Report follows.

---

# `exigere` — wat-rs @ `21530efab` — CODE ONLY (`src/`, `tests/`, `wat/`)

Every `file:line` below was read this session. Every "nothing enforces / not yet" claim was grepped for its enforcement before being believed — two candidates died that way (see § What I killed).

---

## L1 — DEFECTS

### 1. `src/channel/transfer.rs:298-306` — defers work that shipped, in a helper nothing calls, citing a function that does not exist

Comment claims:
```
/// Helper for `:wat::kernel::select` — extracts the underlying
/// `comms::thread::Receiver` if the inner is `Comms`. Returns `None`
/// for `PipeFd` (select is tier-1-only today; piped channels
/// would need an epoll/poll integration that's substrate work
/// for a follow-up arc).
///
/// Arc 214 Stone 5.1 — replaces `try_as_crossbeam_receiver`; the
/// `eval_kernel_select` memory path now registers via
/// `comms::thread::Select` instead of `crossbeam_channel::Select`.
```
Three measured facts against it:
- **The deferred work landed.** `src/comms/process.rs:1533` `pub fn select(&mut self) -> Result<SelectOutcome<T>, io::Error>` is a **persistent io_uring `POLL_ADD` over fds** — literally the "epoll/poll integration" named as a follow-up arc. The live select verb's own doc (`src/runtime.rs:32734-32752`, `eval_peer_select_prime`) states: *"Process tier: same with `comms::process::Select`"*. Select is **not** tier-1-only.
- **`eval_kernel_select` exists nowhere.** Repo-wide grep (`--include=*.rs --include=*.wat`, excluding `target/`) returns only the two comment lines above. The real verb is `eval_peer_select_prime` (`src/runtime.rs:32753`).
- **`try_as_comms_receiver` (`:307`) has zero call sites.** Only hit besides its own definition is the re-export at `src/channel/mod.rs:79`.

Remedy: delete the function and its re-export, or — if it is kept — replace the whole doc with the present tense. The deferral is the worst of the three: it tells a reader a capability is missing that ships one module over.

### 2. `src/types/surface.rs:406` — "a LATER stone" for three things that landed, one of them 95 lines below in the same function

```
// a value <= 0. (Enforcement / checker rule / codegen are a LATER stone — parse only.)
```
All three shipped:
- **Codegen (16.2)** — `src/types.rs:3572`: *"Arc 278 #16.2 — build one `(:wat::core::def :<S>::<OP>-MAX-REQUEST-BYTES <n>)` `WatAST` per…"*, emitted at `src/types.rs:3646`, `:3669`, `:3734`.
- **Checker rule (16.3)** — `src/types.rs:3137-3160`: the MANDATORY-key lock, `if enforce_rtl_lock && !max_request_bytes_explicit { return Err(… MalformedDecl …) }`.
- **The wiring for 16.3 is in this very file**, at `src/types/surface.rs:501-504` and `:515` — `max_request_bytes_explicit` is captured *"BEFORE defaulting: this is what `synthesize_surface_protocol`'s mandatory-budget lock consults"*.

The same declaration's doc in `src/types.rs:455` already speaks of *"the whole point of Stone 16.2's per-op enforcement codegen"* as an existing thing. The parser comment never got the back edge.

Same line also carries a second, weaker deferral at `src/types/surface.rs:392`: *"the design needs a second option `:max-page-bytes` in a later stone"* — grep finds no `max_page_bytes` anywhere in `src/`. That one is honest-but-untracked (L2 shape).

### 3. `src/types.rs:464` — "(or record, future arc)" for a capability arc 293.2b closed, ~260 lines below in the same file

```
/// structural surface. A struct (or record, future arc) satisfies a surface
```
Records satisfy surfaces today, on the identical path:
- `src/types.rs:481` — *"Arc 293.2b — unified product-type declaration (struct or record, discriminated by kind)"* — `TypeDef::Aggregate`.
- `src/check.rs:15612-15618` — `derived_nature` returns `agg.nature` for any `Aggregate`; `Nature::Record` ranks 0 on the satisfaction ladder (`src/types.rs:232`), and `nature_floor_ok` (`src/check.rs:15641`) admits it.
- `src/types.rs:722-740` registers `:nature :Record` surfaces as subtypes of `:wat::core::Record` *"exactly like a concrete Record aggregate: every value satisfying the surface is a record"*, with a named counter-probe (`probe_arc293_holder_ladder_foreign`).

Remedy: strike the parenthetical. Arc 293 is still open (no `INSCRIPTION.md` in `docs/arc/2026/06/293-struct-record-symmetry/`), so this is a *within-arc* stale edge — the arc's own landed stone.

### 4. `src/kernel/mod.rs:49-52` — an `exigere(attested-arc)` rune whose deferred item shipped, spelled in a syntax that is illegal

```
//! - No-prime wat-level type registration (`:wat::kernel::Thread<I,O>` /
//!   `Process<I,O>`) — still pending Stone 4.6.
//!   rune:exigere(attested-arc) — arc 214 Stone 4.6 design at
//!   `docs/arc/2026/05/214-concurrency-toolkit/DESIGN-STONE-4.6-POLYMORPHIC-VERBS.md`.
```
The cited DESIGN file exists (verified). The claim does not:
- **The no-prime parametric types are registered and used in the wat stdlib.** `wat/test.wat:372` `-> (:wat::kernel::Thread :- [R S])`; `wat/test.wat:376` `-> (:wat::kernel::Process :- [I O])`; `wat/spawn.wat:235-236`, `:245-246` `derive` on both; `wat/spawn.wat:357`. The checker matches them at `src/check.rs:11656-11658` (`head == "wat::kernel::Thread" || … "Process"` with `targs.len() == 2`), and `src/kernel/spawn.rs:122` pins `THREAD_PEER_TYPE_PATH = ":wat::kernel::Thread"`.
- **The `<I,O>` spelling the rune uses is hard-cut.** `src/types.rs:4746`: *"angle-bracket type parameters are illegal; write `Head :- [T …]`"*, arc 109 ③ — five enforcement sites (`src/types.rs:3101, 3540, 4736, 4746, 5358`). A rune whose deferred item cannot be written in the language is not deferring anything.

This is the highest-severity rune in the file: a rune's whole job is to be the verification surface, and this one has been false since arc 109.

### 5. `tests/function/wat_arc170_closure_extraction.rs:806-811` — a doc-comment calling a *passing* test a "RED gate", describing a raise arc 278 deleted

```
/// A top-level `def` read from a fn body currently raises
/// `Internal("captured `def`-bound name … not yet supported by closure
/// extraction (slice 1)")`. That arm's own comment says a future arc opens
/// IFF a caller surfaces wanting it — `defservice` is that caller: …
```
- **That raise no longer exists.** Repo-wide grep for `"not yet supported by closure extraction"` in `src/` + `tests/`: zero hits. Arc 278 shipped the carry — `src/closure_extract.rs:376`: *"2b. (Arc 278) `def`-bound values carried from the parent"*; the walker arm is at `:847`.
- **The test asserts success.** `:827-830` `assert_eq!(collect_def_names(&package.prologue), vec![":my::LIMIT"])`; `:839` `assert_i64(&result, 520)`. It is a green gate wearing a RED gate's doc, and it is the only in-file record of what the fulfilment was.

The repo already has the correct pattern for exactly this: `tests/collection/wat_arc167_vector_ast.rs:117-126` rewrote its doc as a **HISTORICAL NOTE** naming the arc that closed the deferral (*"Arc 167 slice 1 said 'a future arc enables vector literals…'. Arc 215 stone 2 is that future arc"*). T22 needs the same treatment.

---

## L2 — WEAKNESSES

### 6. `src/check.rs:11655` — bare `TODO` whose tracker is a CLOSED arc

```rust
if (head == "wat::kernel::Thread"       // TODO(arc-109/arc-170 cleanup): remove Thread'/Process' here —
```
`docs/arc/2026/05/170-program-entry-points/INSCRIPTION.md:3` — **`Status: CLOSED, 2026-07-29`**. Arc 109 is still open (no INSCRIPTION). Half the named tracker can no longer execute the cleanup; the work is now unowned, and there is no rune. Remedy: re-home to a live arc and rune it, or drop arc-170 from the citation.

### 7. `src/value/value.rs:230-234` — "future Vector-tier primitives" that shipped in arc 053/061

```
/// materialization) or by future Vector-tier primitives. Consumed
/// … and by Vector-tier ops
/// shipping in follow-up arcs.
```
Five Vector-tier ops exist: `:wat::holon::vector-bind`, `vector-blend`, `vector-bundle`, `vector-bytes`, `vector-permute`. `src/runtime.rs:24700-24702` says outright *"Mirrors `:wat::holon::Permute` at the **Vector tier**. Arc 053."* and `src/runtime.rs:24743` returns `Ok(Value::Vector(…))` — a Vector-tier primitive constructing the variant the doc says only `encode` constructs. Remedy: name the five, past-tense.

### 8. `src/runtime.rs:22648` vs `src/runtime.rs:40632` — a comment ordering work a gate in the same file forbids

```
// ("has no holon flavor") dies into to_holon_inner's own honest error for now;
// to_holon_inner must be extended to lift base records (STOP-1 gap, 294.a report).
```
`src/runtime.rs:40632` `fn to_holon_inner_base_record_returns_err_with_teaching_message` asserts (`:40645`) *"to_holon_inner(base_record) must return Err, not Ok"* and pins the message parts (`:40653-40657`). The comment says "must be extended"; the gate says "must not." Cited tracker resolves only partly: `docs/arc/2026/06/294-holon-returns-to-vsa/BRIEF-294.a-direct-edn-measurement.md` exists, but there is no "294.a report". Remedy: one of the two is wrong — resolve it, don't leave the substrate holding both.

### 9. `src/rete/reachability.rs:16` — "NOTHING gates …" while six gates for exactly that sit at `:1191-1199`

```
// `every_rete_row_is_total` and siblings). NOTHING gates "can a user actually get here".
```
`reachability_shard_0..5_of_6` (`:1191-1199`) call `sweep_shard` (`:870`), which drives **every** `RETE_OPS` row at both `CallSite::InlineConstraint` and `CallSite::WhereFence` (`:918-925`), with a non-vacuity floor (`:874`, `all.len() >= 74`) and an arity cross-check (`:907`).

I rate this L2, not L1, deliberately: unlike the `purity.rs` precedent, the sentence reads as the file's *motivation clause* ("nothing else gates it, hence this file"). But it is written unqualified and present-tense in a file that has since become the gate, so a fresh reader takes away a false fact. Remedy: past tense — "nothing gated".

### 10. `src/intrinsic/mod.rs:254` and `:261` — "reader lands later", twice, no tracker

```
// `deprecated` is still unread — reader lands later; keep its `#[expect(dead_code)]`.
#[expect(dead_code)] // reader lands later → keep
```
Every sibling field on this struct names its reader by function (`doc_arg_ret_types_match_checker_scheme`, `eval_render_doc`). `deprecated` alone defers, and the deferral is the *stated justification for a lint suppression* — which is what makes it more than prose. Remedy: name the arc, or delete the field.

---

## L3 — JUDGEMENT

- **`src/types.rs:2557-2565`** carries *"Single field for now (the diagnostic message); extensible to kind / location if a real consumer surfaces"* — verbatim duplicated at **`wat/kernel/diagnostics.wat:36-38`**. The Rust copy sits directly above its own `⛔ ARC 296 — GENERATED FROM WAT … wat is the source of truth; Rust consumes it` (`:2571-2574`). The prose whose home moved did not move. One home per fact.
- **`src/kernel/spawn.rs:100-101` and `:114-115`** — two `exigere(scope-affirmative)` runes (plus a third at `src/kernel/mod.rs:47-48`) all defer to *"the runtime.rs flat-sea (Phoenix) warding campaign"*. That is a named **campaign**, not an on-disk arc: "Phoenix"/"flat-sea" appear only in prose across seven `docs/arc/**` files, none of them an arc directory. The `scope-affirmative` category tolerates non-tracking *with substrate-architectural reasoning*; "rides another campaign" is a tracker claim, and the tracker is unresolvable. Give it an arc number or state it as untracked.
- **`src/kernel/mod.rs:47-48`** — the rune is written with `//` line comments spliced between two `//!` module-doc runs (`:46` is `//!`, `:49` returns to `//!`). It is therefore not in the rendered doc. Cosmetic, but a rune that does not appear where its subject appears is half a rune.
- **`wat/query/mem.wat:37`** *"A convenience constructor is future work once/if the substrate grows a scope-detach primitive"* and **`:42`** *"a later stone may add per-index structures"* — both honest about the present, both untracked. The first is the better-formed of the two (it names its unblocking condition).
- **`src/load.rs:123-125`** *"`:wat::verify::http-path`, `:wat::verify::s3-path` are reserved but not implemented. Add new enum arms … when needed."* — reserved-but-absent is a defensible present-tense fact; "when needed" is the disposition-deferral tacked onto it.

---

## What I could NOT check, and why

- **I ran nothing.** No build, no floor, no `cargo`. Every claim above is a static reading of files at `21530efab`. Where I assert a gate "asserts", I read its `assert!`/`assert_eq!` — I did not observe it pass or mutate it to observe it fail. **Finding 5's "the test asserts success" is a code reading, not an execution**; if `t22_toplevel_defn_references_def_bound_value` is currently `#[ignore]`d or failing, my characterisation of it as a green gate is wrong. Nothing in the file suggested that, but I did not drive it.
- **Finding 1's "zero call sites"** is a grep over `*.rs` and `*.wat` in the repo, minus `target/`. `try_as_comms_receiver` is `pub` and re-exported, so an **out-of-tree consumer crate** could call it. I could not check outside this repo. The dead `eval_kernel_select` citation and the shipped-io_uring contradiction stand regardless.
- **Arc-closure status is inferred from the presence of `INSCRIPTION.md`.** That is the convention I observed (arc 170 has one saying `Status: CLOSED`; arcs 109, 251, 261, 293, 294 have none). If an arc can close by another mechanism, my "arc 109/293 still open" reads are wrong — which would *strengthen* findings 3 and 6, not weaken them.
- **Coverage is grep-bounded, not exhaustive.** I ran two pattern sweeps (~70 + ~40 sites) over 327k lines across 2,073 files and read roughly 45 of them. A deferral phrased in wording neither sweep carried is invisible to this cast. Per the standing breadcrumb: a grep that found nothing is evidence about the query. **`tests/` in particular got one shallow pass** (TODO-family + six deferral phrases) — it is 1,803 files and I read three. Finding 5 came out of that thin pass, which is weak evidence that the thin pass was thin.
- **`docs/` was excluded by the builder's ruling**, so where a `src/` comment cites a docs artifact I checked only that the **path exists**, never that its content supports the claim. `docs/arc/2026/06/251-types-as-forms/DESIGN.md` and `docs/arc/2026/06/261-eval-stack-safety-cek/STUB.md` (the two clean `attested-arc` runes at `src/resolve/normalize.rs:430` and `src/distribution/mod.rs:392`) resolve on disk; whether either arc is still open, and whether stone 251.5 already landed without fixing the normalization it promises, I did not determine.

## What I killed before reporting it

Two candidates that read as findings and were not — recorded because the grep is the whole point:

- **`wat/gen.wat:67`** — *"✗ NOTHING GATES THE NAMES ABOVE … `tests/lint/retired_name_justified.rs` … scans `src/**/*.rs` ONLY."* **True at HEAD.** `tests/lint/retired_name_justified.rs:69-83` (`collect_rs`) filters `extension == Some("rs")`; `:223` roots the walk at `manifest/src`. It also names its handoff with a path (arc 255 NOTE). Honest.
- **`wat/gen.wat:52`** — *"handed to arc 293; until it lands, nothing stops a Gen crossing a boundary."* Arc 293 has **not** landed — `docs/arc/2026/06/293-struct-record-symmetry/` has no `INSCRIPTION.md`, only SCOREs through 293.4d. The deferral's condition has not expired. I had this drafted as an L1 on the strength of 22 `Arc 293.x` references in `src/`; the arc-status check refuted it.
