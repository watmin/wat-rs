# ward `excusare` — vigilia 2026-09-05

> Cast at HEAD `21530efab`, branch `grok-rete`. Read-only unless the ward says otherwise.
> **Verbatim ward return, preserved unedited.** The 2026-08-30 cast lost its 19 returns
> because they lived only as subagent messages; this file exists so that cannot recur.
> Nothing here is adjudicated — see `../WORK-LIST.md` for what the orchestrator credited.

---

I have completed the weighing. Report follows.

---

# `excusare` — wat-rs @ `21530efab` (code only: `src/`, `tests/`, `wat/`)

**Weighed:** 15 live `#[ignore]` attributes + 6 doc-comment `#[ignore]` claims; 93 `#[allow]`/`#[expect]` lines; the 5 `rune:excusare(perennial)` in full, plus sampled `sequi(ambient-context)` (17), `lint(retired-name)` (23), `coverage(unreachable)` (3), `purgare(trait-contract)` (3), and every odd-category singleton.

---

## L1 — the reason is false today

### 1. `tests/kernel/probe_arc259_started_at_boot.rs:27` and `:41` — CLOSED-DEFERRAL, false for ~12 weeks

```rust
#[ignore = "RED: seam must read process boot global (set_process_boot_instant) not fresh now — unlock: implement boot-clock global in the seam"]
#[ignore = "RED: peer-started-at must be strictly after started-at — unlock: both stamps when boot-clock global is in the seam"]
```
plus the module header `:13-15`: *"RED at HEAD: there is no boot clock — `set_process_boot_instant` does not exist, and the seam stamps `now` for started-at (ignoring any earlier capture)."*

**Every falsifiable clause is false. I read each site:**
- `src/time.rs:43` — `pub fn set_process_boot_instant(inst: DateTime<Utc>)` exists, pid-keyed, and its own doc comment names this exact use: *"tests inject a known value for deterministic timing."*
- `src/freeze.rs:1484` — `let boot_nanos = crate::time::process_boot_instant()…`
- `src/freeze.rs:1512` — the seam builds `:started-at (:wat::time::at-nanos {boot_nanos}) :peer-started-at (:wat::time::now)` — literally the two distinct, ordered stamps both ignored tests assert.
- `src/kernel/spawn.rs:701` — same boot-clock read on the thread-peer path.
- The named unlock ("implement boot-clock global in the seam") shipped in **`765a5dd87`, 2026-06-12**, whose subject line is *"arc 259: the timing correction — started-at is the real boot, not the seam."*
- The third test in the same file (`process_boot_instant_is_stable_within_a_process:53`) is **not** ignored and calls `wat::time::process_boot_instant()` — it is on the floor and green, so the API it says is missing is being exercised two functions below.
- The fixture `tests/kernel/probe_arc259_started_at_boot.wat` exists and its assertions (`epoch-seconds started-at == 1000`; `peer-started-at - started-at > 0`) map 1:1 onto what freeze.rs:1512 now produces.

The file was last touched 2026-08-14 (`fe071b6d7`) and nobody re-weighed the exemptions. **Closure:** un-ignore both, correct the header. If either then reds, that is a real finding the floor has been blind to for three months — which is the point.

### 2. `src/process/boot/mod.rs:292` — bare `#[allow(dead_code)]` on a sham keep-the-import function

```rust
#[allow(dead_code)]
fn _span_type_is_used(_: &Span) {}
```

No reason at all. I grepped `Span` across the whole file: **two hits** — the import at `src/process/boot/mod.rs:65` (`use crate::span::Span;`) and this function's parameter at `:293`. `_span_type_is_used` has zero callers anywhere in `src/` or `tests/`. It exists solely so `unused_imports` does not fire on line 65, and the bare `#[allow(dead_code)]` silences the lint that would otherwise expose the whole arrangement. Two checkers defeated, one warrant offered: none.

This also violates the repo's own doctrine, stated three files away at `src/intrinsic/mod.rs:25`: *"`#[expect(dead_code)]` (NOT `#[allow]`): silent while genuinely dead, but [the expectation fires when it stops being dead]."*

**Closure:** delete `:65`, `:292`, `:293`. Contrast the sibling guards at `:616`/`:626` (`_every_boot_frame_variant_is_covered`), which carry a real structural warrant — *"a new variant breaks the build HERE"* — and HOLD.

---

## L2 — warrant weakening

### 3. Nineteen `#[allow(clippy::too_many_arguments)]`, zero reasons — and the repo already ruled against them

Sites (all bare; I grepped for any inline reason and got **0**): `src/macros/expand.rs:1155,1510,1607` · `src/check.rs:9354,9462,11840,13515,14193,16036` · `src/rete/compiled_cond.rs:210` · `src/rete/kernel/fire/pass/{round_census.rs:25, join_after_filter.rs:31, alpha.rs:259, filter_after_join.rs:17, hash_join.rs:41,412,483}` · `src/process/verbs.rs:301` · `src/rete/validate/typing.rs:742`.

`src/rete/validate/mod.rs:392` states the governing judgement in this tree, at a site that *refused* the suppression in favour of a `ClauseCtx` struct: *"the alternative here was an `#[allow(clippy::too_many_arguments)]`, **which silences the signal instead of answering it**."* Nineteen sites take the alternative that doctrine names as wrong, and none says why it is right there.

**Three are additionally inert** — they suppress a lint that cannot fire. `clippy::all = deny` (`Cargo.toml:51-52`) with **no** `too-many-arguments-threshold` in `clippy.toml` (I grepped: no `threshold`/`too-many` key), so the default 7 applies and the lint fires only at 8+. I counted these signatures by hand:
- `src/check.rs:9462` `infer_nth` — **6** params
- `src/check.rs:11840` `process_let_binding` — **6** params
- `src/check.rs:16036` `dispatch_rust_scheme` — **7** params

**Closure:** delete those three outright; for the remaining sixteen, either give each a one-sentence structural reason or take the `ClauseCtx` cure the tree already prefers.

### 4. Twenty-four bare `#[allow(clippy::mutable_key_type)]` made redundant by `clippy.toml`, and now masking it

24 sites, **0 with any reason**. `clippy.toml` sets `ignore-interior-mutability = ["wat::value::value::Value", "wat::Value"]`, landed **`3a0ed6be7`, 2026-07-30**, subject *"clippy stone D3: the exemption the wall earned — mutable_key_type 18 -> 0."* Every site I inspected keys on `Value` (`HashMap<Value,…>` ×10, `HashSet<Value>` ×2, `LruCache<Value,Value>` ×1 within two lines of the attribute; the rest sit on `Value`-domain fns like `hashset_conj_inner`). I blamed five (`src/runtime.rs:4962`, `src/edn_shim.rs:1071`, `src/collection/eval.rs:598`, `src/rust_deps/cache.rs:86`, `src/runtime.rs:20087`) — all predate 2026-07-30.

The harm is not tidiness. `clippy.toml`'s own reason ends: *"If that gate is ever weakened or deleted, this entry must go with it."* Twenty-four inline allows would keep all those sites silent after the entry and its wall were removed — they defeat the one mechanism the config was written to depend on.

**Closure:** strike all 24; the config entry is the single, warranted point of truth.

### 5. `tests/services/probe_arc278_self_scheduling.rs:25` and `:45` — the load-bearing citation dangles, half the reason shipped

```
#[ignore = "item-c: … remaining = the remove-at idx-shift (service.wat:958/961) evicting the client
            peer, + the send'-wall makes the failure legible (DESIGN-send-outcome-wall.md)"]
```

- **`wat/service.wat:958/961` is not a `remove-at` site.** I read them: they are `handle-ty-str` / `handle-base-kw` keyword mints. Every `remove-at` in the file is at `1591, 1594, 1917, 1929, 1963`.
- The mechanism it names appears structurally answered. `wat/service.wat:1817-1824` now states: *"the returned `idx` is a POSITION, valid identically against either vector (the projection preserves order 1:1), so every other `idx` use below (nth/remove-at against the CANONICAL `selectables`) is unaffected"* — the Tuple-wrapped canonical `selectables` change.
- **The send'-wall half has shipped.** `src/types.rs:1181`: *"RecvOutcome / **SendOutcome** / CloseOutcome each **REPLACED** the raiser"* (past tense), and `wat/service.wat:1936-1963` matches `SendOutcome::{Sent,Closed,Stopped,Lost}` exhaustively in the serve loop.
- The reason dates to `eafd26142`, **2026-07-23**.

**Closure:** re-point the citation at the real line, or drop the shipped half and re-run — the exemption may already be a CLOSED-DEFERRAL.

### 6. Six process-tier test files document an `#[ignore]` that no longer exists

`tests/kernel/spawn_program_prime_process.rs:25,163,220` · `peer_verb_round_trip_process.rs:17,30` · `peer_select_prime_process.rs:18,32` · `peer_process_round_trip.rs:53` · `probe_arc278_close_outcome_wall.rs:107` · `probe_arc214_beta_forms_server.rs:40` — each says *"Marked `#[ignore]` — run via `integration-run.sh` or `--ignored`."*

I grepped every one for `^\s*#\[ignore`: **zero hits.** Each carries a plain `#[test]` and runs on the default floor. The attributes were removed in **`70376d634`, 2026-07-28**, *"170: the process-tier quarantine is stale — 8 probes fold into the floor"* — the exemption was struck, its documentation was not. A reader is told the floor does not cover these; it does. **Closure:** delete the stale run-instructions.

### 7. `src/macros/parse.rs:92` — `rune:coverage(unreachable)` whose evidence is a false enumeration

```rust
// rune:coverage(unreachable) — `is_defmacro_form` requires `WatAST::List`; all four
// callers (`register_defmacros`, `register_stdlib_defmacros` × 2) gate on it before dispatch.
```
(the plain comment at `:90` repeats *"All four call sites guard with `is_defmacro_form`"*.)

I enumerated the callers of `parse_defmacro_form`: **six** in production — `src/macros/parse.rs:15, 36, 315` and `src/macros/expand.rs:115, 326, 618` (plus `src/macros/tests.rs:56`). Five gate on `is_defmacro_form`. **`src/macros/parse.rs:315` does not** — it is `register_aggregate_kwargs_companions` (arc 294 item 9a), which reaches the function via `parse_one!(&source)` on synthesized text. The panic remains unreachable *in fact*, but by a mechanism the rune does not name, at the one site a future non-List could enter. The rune's count and its list are both wrong, so an auditor who checks it concludes "verified" from evidence that does not hold.

**Closure:** name the real caller set and the second mechanism, or gate `:315`.

### 8. `src/freeze/validator.rs:52` — inert `#[allow(dead_code)]`

`pub name: &'static str` on `pub struct FreezeValidator`, in `pub mod validator` (`src/freeze.rs:59`) inside `pub mod freeze` (`src/lib.rs:90`). Publicly reachable from the library's API, so `dead_code` cannot fire. The attached note ("diagnostic/introspection field") describes the field's role, not why the lint is wrong. Minor; **closure:** strike the attribute.

---

## L3 — judgement

- **`tests/macros/probe_arc260_keyword_args.rs:20` — HOLDS-with-note.** The exemption is warranted (the probe is genuinely RED), but the prose *"wat has no keyword args"* is now materially false: `wat/core.wat:653-665` ships `defn`'s `& [argspec]` kwargs section (arc 260.1a), and `src/macros/parse.rs:288-343` ships `kwargs-construct` aggregate reordering. What is actually unbuilt is narrower — call-site `:name val` reorder for a *plain* positional `defn`, which is what `probe_arc260_keyword_args.wat`'s `:user::sub [a b]` exercises. Update the sentence; keep the ignore.
- **The five `rune:excusare(perennial)` all HOLD, and I drove the claims.** `src/comms/{process.rs:866,1486, thread.rs:229,319, mod.rs:237}`. The empty-Select guards both exist exactly as described, including the tier distinction the two reasons draw: `src/comms/thread.rs:353` **panics**, `src/comms/process.rs:1536` and `:1768` return **`Err`**. The narrowed-`len()` contract is real. These are the model the rest of the tree's suppressions should be measured against.
- **`clippy.toml` is exemplary and should be cited as the standard** — it names why the lint fires, why it cannot bite, and what would invalidate the exemption, resting the whole thing on `KeyEligibility` plus a mutation-proven gate. Finding 4 exists because 24 inline allows undercut it.
- **The arc-255 `#[ignore]` family HOLDS**, as calibrated. `tests/wat_lang/probe_undefined_builtin_resolves.rs:17,31` and the `probe_arc255_*` reflection set bank unbuilt features, and `tests/lint/every_wat_bad_fixture_actually_fails.rs:297` makes the `bad-is-banked` exemption self-clearing (it reds when a named owner stops being `#[ignore]`d). Do not reap these.
- **`rune:lint(retired-name)` HOLDS** on the two classes I drove: `wat/kernel/readln.wat:59` is a real `defmacro :wat::kernel::readln` whose header at `:34` states the pair exactly as the 7 runes claim; `src/collection/transform.rs:264` is the live `sort'` comparator primitive.
- **`rune:purgare(trait-contract)` at `src/rete/clause.rs:71,75` HOLDS** — every consumer destructures `Accumulate { from, .. }` or `{ .. }` (`compiled_cond.rs:567`, `validate/typing.rs:515`, `matcher.rs:791`, `stratify.rs:156,179`); `var` and `acc_form` are genuinely unread.
- **The odd-category runes are negative controls, not suppressions** — `rune:lint(whatever)`, `rune:lint(too-slow)`, `rune:sequi(made-up)` are string literals inside `tests/lint/` self-tests proving the walkers surface an invented category rather than skipping it. Correct by design.

---

## What I could not check, and why

- **Whether any ignored test actually passes now.** Read-only: no builds, no floor. For finding 1 I proved the *reason* false from four source sites and a commit; I could not prove the test is green. That is the right split — excusare judges the excuse, not the finding — but it means finding 1's closure needs one run to land.
- **The `wat/service.wat` idx-shift (finding 5) may or may not be cured.** I established the citation dangles and the send'-wall shipped; whether the Tuple-canonical `selectables` change actually fixed the eviction is a runtime question I could not settle by reading.
- **`rune:lint(loose-assert)` × 178 — not weighed.** The largest single family. I confirmed its walker exists but did not sample a single site's reason against its assertion. This is the biggest unweighed surface in the repo and the obvious next cast.
- **`rune:lint(cited-name-absent)` (25), `rune:vocare(vantage-bypass-test)` (24), `rune:lint(unused-span)` (24), `rune:lint(census-name-retired)` (10), `rune:lint(no-inlined-edn)` (54), `rune:lint(no-inlined-wat)` (38), `rune:perspicere(read-once)` (36) — not weighed.** The absence-asserting families (`cited-name-absent`, `census-name-retired`) are the highest-risk of these: each claims a name does not exist, and a re-mint inverts them silently. I verified `retired-name` and stopped.
- **`src/process/boot/mod.rs:616`** defers to arc 296.3 via a `docs/` NOTE. Its in-code warrant (the exhaustive-match rung) holds on its own, so I did not chase the target's status — and the builder's ruling puts a docs-only remedy out of scope.
- **My parameter counts in finding 3** were done by hand on three signatures I read in full (`infer_nth`, `process_let_binding`, `dispatch_rust_scheme`); the other sixteen came from a crude awk pass I do **not** trust and did not report numbers from. Two of them (`compiled_cond.rs:210`, `verbs.rs:301`) returned obviously wrong counts because the attribute sits above a doc comment. Re-derive before acting on anything but those three.
- **Finding 4's coverage claim is a sample.** I confirmed the key type at 13 of 24 sites and blamed 5. If any of the remaining 11 keys on something other than `Value`, that site's allow is live and my "all redundant" is too strong for it.
