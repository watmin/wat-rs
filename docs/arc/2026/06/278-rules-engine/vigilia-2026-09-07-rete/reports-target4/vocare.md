# VOCARE — Cast Report (TARGET 4 — the test corpus) — **FINDINGS**

> Written verbatim as returned, before any synthesis. Evidence only — this file carries NO status.

### 1. What I swept, and deltas from every handed-down figure

Commands used throughout: `find`, `wc -l`, `grep -rn`/`-rl`/`-c`, `sed -n`, `ls`. No edits, no cargo/nextest.

- **File/line inventory** — confirmed exactly: `find tests/rete src/rete/kernel/tests -type f | wc -l` = **306**; `wc -l` over all of them = **38,058**. Split: `tests/rete` = 100 `.rs` + 144 `.wat` + 19 `.bad` + 23 `.edn` = 286 files / 23,633 lines; `src/rete/kernel/tests` = 20 `.rs` / 14,425 lines. Both sub-totals match the brief exactly.
- **`#[test]` count — DELTA.** Brief said 613. Raw `grep -rn '#\[test\]'` over both trees returns **618**, but 3 of those are prose inside `.wat` comments describing what a `#[test]` needs (`probe_arc278_P2_native_fire_once.wat:17`, `probe_arc278_P4a_native_fire_rules.wat:60`, `probe_arc278_session_ceiling_second_session.wat:55`) — trap #2, prose matching a grep for the thing. Real count, `.rs` only: `tests/rete` 490 + `src/rete/kernel/tests` 125 = **615**. Delta from the brief: **+2** (613 vs 615), unexplained but small; no test macros generate multiple fns from one `#[test]` (`grep -rl macro_rules!` over both trees returned nothing).
- **Sampling rule for 615 tests.** I did not read all 615. I read: every rune site verbatim (24 grep hits, 23 real), the 5 files named as unconfirmed leads in full, every file matching the Aggregate-poking pattern in full (4 files), and a ~35-file stratified sample of the remainder for import-mechanism variety (which verbs they call). I did not individually read the bodies of the other ~550 tests.

### 2. Architectural-exemption ruling — `src/rete/kernel/tests/`

**The `src/*.rs` exemption applies, and cleanly.** `src/rete/kernel/mod.rs:43-44` reads `#[cfg(test)] mod tests;` — this is a real in-crate unit-test module, not a relocated integration suite. `src/rete/kernel/tests/mod.rs` documents it as a 2026-08-30 split of a single `tests.rs`, and every child file opens with `use super::*;`, reaching kernel's `pub(crate)` internals (census, fire, stratify) directly by the same route any other kernel-internal module would. Spot-checked `accum_alpha_cost.rs`, `accum_cost.rs`, `alpha_discrimination.rs` — all consistent with "the test's vantage matches its actual caller's vantage" (another in-crate module). I did not find grounds to disagree with the brief's reading. All 125 `#[test]`s here are out of scope for findings.

### 3. Vantage verdict on the `tests/rete/` corpus as a whole

**Consumer-vantage by construction, with a caveat on the specific "2 verbs / 99 of 100" mechanism-count — DELTA.** Re-derived:

- `grep -rl 'startup_beside(' tests/rete/*.rs` → **7 files** (40 call-sites), not 35.
- `grep -rl 'call_beside_value(' tests/rete/*.rs` → **62 files**, not 64.
- Combined, exactly these two named verbs: **65 of 99** non-`mod.rs` `.rs` files, not "99 of 100."
- The other ~34 files are not a gap in the architecture — they use adjacent, equally consumer-facing drivers I read directly: `call_beside` (5 files), `run()` spawning the compiled `wat` binary via `Command::new` and checking stdout/stderr (e.g. `probe_arc278_fixpoint_round_cap.rs`, `wat_scripts_grid_axes_live.rs` — exactly what a CLI user does), or `startup_from_file` + `apply_function` (`probe_construction_headline.rs`, `probe_fence_names_the_head.rs`). Also confirmed no test imports a deep internal module path (`grep` for `wat::rete::kernel::`/`fire::`/`alpha::` etc. as an import returned nothing) — the only non-`freeze`/`runtime` imports are `load::InMemoryLoader`, `assertion::AssertionPayload`, `check::CheckErrorKind`, `ast::WatAST`, and `value::AggregateValue`, all legitimate embedder-facing surfaces.
- `WatAST` deserves a note since it looked implementer-ish: it is a first-class wat **value type** consumers pass and receive (`(:wat::rete::eval-test <quoted-expr: :wat::WatAST> …)`), so `probe_arc278_P12c_explain_payload.rs` matching on `WatAST::List`/`IntLit` is a consumer inspecting a returned value, not a vantage breach.

**Verdict: the corpus's vantage is right by architecture**, but the specific "2 verbs, 99/100" figure in the brief does not hold as stated — the mechanism set is wider than 2 verbs, and the file count using exactly those two is 65, not 99.

### 4. Findings

**Finding A — `tests/rete/probe_arc278_import_fold_key.rs`: 6 unruned tests do the identical "host `Aggregate.fields` poke; wat has no Export setter" bypass that `probe_arc278_export.rs` already runes 7 times over, elsewhere in the same corpus.**

- Tests: `import_refuses_a_fold_key_no_condition_binds` (line 33), `import_refuses_an_unpacked_fold_key_no_condition_binds` (180), `import_refuses_an_unpacked_fold_key_bound_to_a_string` (185), `import_refuses_a_slot_fold_key_bound_to_a_string` (246), `import_refuses_a_slot_fold_key_no_condition_binds` (295), `import_refuses_a_slot_min_fold_key_no_condition_binds` (341).
- Mechanism: `field_of`/`poke`/`rewrite_sum_keys`/`rewrite_sum_folds_to` (lines 88-143) reach into `Value::Aggregate { a.names, a.fields }` directly and rebuild the `Export` via `AggregateValue::record(...)` to tamper a `:sum` fold's key on the wire, then re-enter through `:wat::rete::import` (public verb, confirmed at `probe_arc278_import_fold_key.wat:70-71`, `:ifk::fired-sum (:wat::rete::import e)`).
- I confirmed there is no wat-level Export setter (`grep` across `src/rete/*.rs` and `wat/*.wat` for any export-mutation verb returned nothing) — the exact fact export.rs's own runes cite as the reason the bypass is necessary (`probe_arc278_export.rs:136,177,254,462,500,556,570`, e.g. "host Aggregate.fields poke; wat has no Export setter").
- Presented vantage: consumer (the file's header calls this a "DISCONFIRMING PROBE" about what `import` refuses). Actual vantage: implementer/protocol — hand-building a malformed wire value no wat consumer can construct. Recommended surface: none exists in wat (confirmed above), which is exactly what the sibling file's rune records — so the fix is a rune, not a rewrite.
- Judgment: **real, load-bearing test** (it caught/guards a genuine host-panic-on-wire-poison class per its own docstring), but it needs the same `rune:vocare(vantage-bypass-test)` annotation its sibling `export.rs` already carries for the identical maneuver. This is a documentation/consistency gap in the ward's own coverage, not a defect in the test's logic.

**Finding B — `tests/rete/probe_arc278_import_accounting.rs`: 2 unruned tests, two different bypass shapes.**

- `import_refuses_a_node_count_past_the_cap` (line 131-132): `poke_named` (line 229-243) is the same `Value::Aggregate` field-poke as Finding A, tampering the wire `nodes` field to test `MAX_IMPORT_NODES`. Same missing-rune gap as Finding A.
- `an_origin_already_filed_is_never_re_based` (line 176-177): calls `wat::alloc_counter::mark_session_origin_at`, `thread_bytes`, `session_bytes` **directly** — zero wat evaluation, zero `Session`/`Export` value in play. The test's own docstring says why: *"The two `.wat` arms above cannot see this — each files its key exactly once — so this arm has its own probe or it has none."* That is a self-declared vantage-bypass, word for word the shape the `vantage-bypass-test` rune category exists to mark, sitting in the same file as (and presented alongside) two tests that genuinely do drive through the wat surface (Arms 1-2, via `run()` against `.wat` twins).
- Presented vantage: this file's docstring calls all three "arms" one probe (Arm 1/2/3 numbering), giving the appearance the whole file verifies from the consumer side. Actual vantage of Arm 3: a Rust-internals unit test of `alloc_counter`'s own non-clobber invariant — the same vantage as a `src/*.rs` unit test, but it lives in the external integration-test crate instead.
- Judgment: **real defect-guard** (protects against `mark_session_origin_at` clobbering, which the comment says was previously reachable) — needs either a `rune:vocare(vantage-bypass-test)` annotation (it names its own justification already, almost verbatim to the rune-reason format) or relocation to `src/rete/export.rs`'s own unit-test module where the `src/*.rs` exemption would cover it outright.

I checked whether these five sites might be covered by the **defect-exposure-fixture** exemption ("sets up a state the caller can produce"); they are not — the whole point of export.rs's precedent is that wat has no Export setter, so this state is *not* caller-producible, which is exactly what makes it a vantage-bypass needing the rune rather than an exempt defect fixture.

**Non-findings confirmed by direct read (the other 3 of the brief's 5 leads):** `probe_arc278_4a_production_fire.rs`, `probe_arc278_accessor_purity.rs`, `probe_arc278_d7_parametric_erasure_differential.rs` — all clean. Each test calls only `call_beside_value`/`startup_beside` against public wat verbs (`query`, `pure?`/`deterministic?`, `fire-rules`/`fire-rules$oracle`); the words `production-memory`/`Aggregate` that tripped the original grep are exclusively in doc-comment prose explaining mechanism to the reader, never in test code. Confirmed by reading both `.rs` and `.wat` siblings and the actual wat-level entry points each test calls.

### 5. The 23 runes — recorded as skipped, not re-adjudicated

Re-grepped `rune:vocare` across the whole target: 24 raw hits, of which `probe_arc278_join_carries_both_sides_into_the_rhs.rs:10` is prose *about* the rune (referring to `pass_semantics.rs`'s runes), not a rune itself — trap #2 again. Real count: **23**, matching both the brief and excusare's figure exactly:

- `probe_arc278_4c_retraction.wat:62,69` (2)
- `probe_arc278_2b_insert_alpha.wat:35,46,54,64` (4)
- `probe_arc278_alpha_is_fire_scoped.wat:32,41,51` (3)
- `probe_arc278_P2_native_fire_once.wat:59` (1)
- `probe_arc278_compiled_where_ops.rs:53,71` (2)
- `probe_arc278_export.rs:136,177,254,462,500,556,570` (7)
- `src/rete/kernel/tests/pass_semantics.rs:233,333,450,523` (4, and redundantly exempt anyway as `src/*.rs`)

Per the spell: skipped, not re-adjudicated. Also verified `probe_arc278_export.rs`'s 7 runes fully cover that file's 7 `Value::Aggregate(a)` poke sites (no gap there — the gap is only in its two siblings, above) and `probe_arc278_compiled_where_ops.rs`'s 2 runes fully cover its Aggregate-poke.

### 6. What I looked for and did not find

- No test in `tests/rete/` imports a private/deep-internal Rust module path beyond `freeze`/`runtime`/`load`/`assertion`/`check`/`value`/`ast`/`alloc_counter` — checked via import-line grep across all 100 files.
- No macro-generated `#[test]` fns (would have inflated the true count silently).
- No other file besides the 4 already discussed (`export.rs`, `compiled_where_ops.rs`, `import_fold_key.rs`, `import_accounting.rs`) touches `Value::Aggregate`/`AggregateValue::record` at all — this bypass shape is fully enumerated, not a sample.
- `probe_arc278_49_one_core_covers_the_surfaces.rs` self-admittedly models shapes locally rather than calling any real interface ("this is an integration test, `Op` is `pub(crate)` … the probe models the shapes locally and cannot be held against the real type") — this is a vacuous-probe concern, not a vantage-breach (it never reaches past an interface; it never reaches an interface at all), so I did not report it under vocare.

### 7. **FINDINGS**
