# ward `complectens` — vigilia 2026-09-05

> Cast at HEAD `21530efab`, branch `grok-rete`. Read-only unless the ward says otherwise.
> **Verbatim ward return, preserved unedited.** The 2026-08-30 cast lost its 19 returns
> because they lived only as subagent messages; this file exists so that cannot recur.
> Nothing here is adjudicated — see `../WORK-LIST.md` for what the orchestrator credited.

---

I hold the spell and have completed both phases. Report follows.

---

# `complectens` — wat-rs @ `21530efab` (code only: `src/`, `tests/`, `wat/`)

**Phase 1 (reproducible):** 4381 `#[test]` bodies extracted by brace-scan across `tests/` + `src/**`, sorted by body line-count with `let`-binding and assert counts; plus a helpers-vs-tests-per-file census and a self-comparison scan (`assert_eq!(f(..), f(..))`). Scripts in the session scratchpad. **Phase 2** below is judgement on what I read.

---

## L1 — defects

### 1. The file's only nesting proof has exactly one possible outcome — and the layer under it collides

**`tests/value/probe_arc216_stone5a_value_hash.rs:299-319`** — `probe_9_deep_nesting` builds `nested`, then:

```rust
let h1 = hash_value(&nested);
let h2 = hash_value(&nested);
assert_eq!(h1, h2, "deep nested Value hashes consistently");
assert_eq!(nested, nested, "deep nested Value equals itself");
```

Both assertions hash/compare **the same binding**. They hold for any deterministic function, a constant included. The file header declares this probe as chapter 9, "Deep nesting" (`:17`), so it is the whole of the file's advertised coverage for nested `Value` hashing.

**And the layer it stands over is measurably wrong.** `impl Hash for Value` (`src/value/value.rs:751-762`) early-returns for `Value::Vec` and `Value::wat__core__List` into `hash_sequence` (`src/value/value.rs:558-567`), **skipping the `std::mem::discriminant` tag every other arm gets** (`:763`). `hash_sequence` writes `SEQ_TAG` (one `u8`) then each element — **no length, no terminator**. Depth is therefore not encoded, and two unequal values emit an identical write stream:

- `Vec[ Vec[], Vec[i64 1] ]` → `A5, A5, A5, disc(i64), 1`
- `Vec[ Vec[ Vec[], i64 1 ] ]` → `A5, A5, A5, disc(i64), 1`

They are unequal under `PartialEq` (`src/value/value.rs:628`, `(Vec(a), Vec(b)) => a == b`, so lengths 2 vs 1 differ). The doc directly above the helper (`src/value/value.rs:553-556`) asserts *"correctness rests on the full 64-bit hash output (collision ~1/2^64)"* — **false for this family**: the collision is structural, not probabilistic.

*Blast radius, checked:* content-addressing does **not** run through this impl — `src/hash.rs` uses canonical EDN bytes → `[u8;32]` (`:152, :282`), and `hash_value` has no callers in `src/` outside `hash.rs`. So today the damage is degenerate bucketing in `HashSet<Value>`/`HashMap<Value,_>` (Eq still disambiguates) plus a false documented claim — but the `wat__std__HashSet` arm (`src/value/value.rs:786-795`) hashes *element hash values*, so it amplifies this class rather than absorbing it.

**Fix:** hash the length in `hash_sequence` (`pmap.rs:359-361` already does the equivalent — it hashes a length-bearing `Vec` of sorted pair hashes; the asymmetry is unexplained), and replace `probe_9`'s tautology with the two-value discrimination assertion above. The fix and the test are independent; do the test first — it reddens today.

### 2. A probe whose name states a finding its body never checks

**`tests/comms/probe_arc278_partial_frame_residue.rs:189`** — `probe_sender_send_leaves_headless_partial_frame_on_shutdown`, 242-line body, 38 sequential bindings, no named layer, no rune. Its subject quantity is computed at **`:327`** (`let partial_residue = total_now - clean_filler_remaining;`) and appears exactly once more, **printed** at `:330-334` as `"non-empty = {}"`. There is no assertion on it — `grep -n "partial_residue\|assert"` returns four asserts, all on setup (`:199` pipe2, `:262` filler decode, `:343` filler decode). Both arms of the ★ question (case (a) `:365-379`, case (b) `:412-425`) `println!` on either branch, timeout included.

The name asserts a conclusion; the body has one outcome. A regression that made `send` leave `written == 0` reads green under a name saying the opposite, and the whole `#71` chain rests on this probe's *printed* facts. It also spends ~3s child sleep + up to 20s×2 bounded waits of floor wall to assert nothing.

`:428-429` says *"measures and reports; it does not assert a verdict (STOP-1)"* — that is a legitimate stance, but then the name must not carry the verdict, and there is no `rune:complectens(...)` declaring the exemption (the tree uses that rune in 9 other places, e.g. `src/macros/tests.rs:837`).

**Fix:** either assert `partial_residue > 0` (the one fact the file was commissioned to buy — the setup already guarantees the Shutdown arm at `:309-321`), or rename to `measure_…` and carry a rune.

---

## L2 — weaknesses

### 3. A shared instrument proved on one of its two guards, by a test whose name claims both

**`src/rete/kernel/tests/mod.rs:466-590`** — `render_phase_table` is the one copy of the instrument-subtraction arithmetic (its own doc at `:474-477` says why one copy matters), consumed by three phase-census tests. It asserts `required` phases are present (`:520-527`), then computes every printed percentage against `total_net`, summed over `top` with a **silent `filter_map`** (`:549-557`). `top` is never checked for membership.

At **`src/rete/kernel/tests/fanout_cost.rs:341-346`** and **`src/rete/kernel/tests/node_share_cost.rs:904-909`**, `TOP` contains `"IN: to_transient"` and `"OUT: to_persistent"`, both **absent from that call's `REQUIRED`**. If either mark stops firing, nothing panics, the denominator silently shrinks, and every percentage in the table inflates.

Its only proof, **`src/rete/kernel/tests/rank_and_instrument.rs:261-274`**, is named `render_phase_table_proves_missing_phase_and_zero_total` — but its census supplies `("IN: to_transient", 1, 1)`, which *is* the `top` phase, so `assert!(total_min > 0.0)` at `mod.rs:553` is **never reached**: the panic comes solely from the `required` arm at `:525`. One mutation, two claimed arms.

**Fix:** assert `top ⊆ samples` (or `top ⊆ required`) beside the `required` check, and add the second mutation to `rank_and_instrument.rs` so the name is earned.

### 4. The load-bearing extractor for the only native-vs-oracle differential is duplicated and unproven

**`tests/rete/wat_scripts_grid_port_check.rs:281`** and **`tests/rete/wat_scripts_grid_axes_live.rs:350`** define byte-identical `extract_vector_field` parsers. `port_check.rs` has **one** `#[test]` over **eight** helpers (`:188, :193, :211, :229, :258, :281, :290, :295`) — none individually proven. `extract_vector_field` is what the whole port differential's `:derived` vs `:oracle-derived` comparison rests on.

Its correctness has been reasoned about in prose **and gotten wrong twice**: `axes_live.rs:342-346` states as fact that `":oracle-derived"` CONTAINS `":derived"`; `axes_live.rs:465-470`, 120 lines below in the same file, says *"It cannot — the colon is part of the needle"* and leaves the wrong doc-comment standing. `port_check.rs:270-278` records the same correction a third time, citing a hand-run `grep` dated 2026-09-03 as its evidence. Three comments, zero tests.

**Fix:** one `#[test]` on the extractor (`" :oracle-derived [9] :derived [1 2] "` → correct value and span for each key) in the shared position, delete the second copy, and delete the two contradictory paragraphs. That settles mechanically what has now cost three rounds of comment archaeology.

### 5. The gate's own justifying example is false

**`tests/lint/every_walking_gate_declares_non_vacuity.rs:24-29`** cuts "gates that name their files" out of its population, and the load-bearing example for the cut is: *"`gen_doc_surface_matches.rs` parses 27 verbs out of a named file and **would pass on 0**"*. It would not — **`tests/lint/gen_doc_surface_matches.rs:146-151`** asserts `verbs.len() >= 20`. The excluded population's cited exemplar is guarded; the scope cut's stated cost is fictional.

**Fix:** correct or replace the example in the module header. It is prose, but it lives in a `tests/` code file and it is the sole evidence for a population boundary.

### 6. `wat/lint.wat`'s top layer one-shots the hardest input, and its proof is switched off

**`wat/lint.wat:632`** defines `lint-stdlib`. Its only test — **`wat-tests/lint.wat:97-107`** — runs it over the **entire stdlib**, and is (a) `ignore`d for exceeding the harness's 5000 ms per-deftest cap under floor contention and (b) tautological by the file's own admission (`assert-true (>= (length findings) 0)`; the file records it returns 136 findings). The test's own title claims "rule-zero present", which the body asserts nowhere.

The file documents all of this honestly, so the gap is known. What is missing is the complectens remedy: the layer is untested **because its one test one-shots the largest possible input**. A `lint-source` over one fabricated file — the shape three sibling deftests in the same file already use (`:118-131`, `:136+`) — would fit inside the cap and prove the same entry point.

### 7. Six 200–300-line cost bodies with no named layers and no rune

**`src/rete/kernel/tests/accum_alpha_cost.rs:75`** (303 lines / 41 `let`), **`:383`** (300 / 56), **`:987`** (293 / 48); **`accum_cost.rs:1043`** (258 / 43), **`:813`** (226 / 38), **`:597`** (213 / 43); **`gather_probe_cost.rs:923`** (259 / 46); **`fanout_cost.rs:472`** (202 / 22). `grep -c rune:complectens` over all four files: **0**, while the sibling `node_share_cost.rs:97` does carry one for its 716-line body.

These are not embedded-literal fixtures — the bindings are genuine outer composition (`wp, w, c, t, m, a, a_prod`, six `f64::INFINITY` accumulators, three nested closures). When the derived `A−M` ladder goes negative, no name in the trace says which arm moved. Phase-2 verdict: real Level-1 shape, uneven rune discipline.

**Fix:** name the ladder arms as helpers (`isolated_class_extract`, `isolated_candidates`, `isolated_exec`, `isolated_activate`), or add the rune with the reason. Both are cheap; the current state is neither.

---

## L3 — judgement

- **`tests/lint/minimum_label_matches_its_estimator.rs:274-276`** walks `src/` only. No `MINIMUM of` header exists outside `src/` today (checked `tests/`, `benches/`, `examples/`, `tools/`), so the cut is currently harmless — but it is undeclared, and perf-shaped tests already live under `tests/rete/`. One sentence in the header, or extend the walk.
- **`src/value/pvec.rs:147-153`** — `impl Hash for PVec` writes neither a tag nor a length; the file's only two hash assertions (`:190`, `:211`) are cross-arm *agreement*, and the file contains **zero** `assert_ne!`. The sibling `src/value/pmap.rs:359-361` hashes a length-bearing `Vec`. Same family as L1 #1; separate impl, so it needs its own line.
- **Positive control, worth saying:** `src/rete/kernel/tests/where_tree_branch_differential.rs` and `tests/lint/minimum_label_matches_its_estimator.rs:361-446` are the shape this ward exists to find *present*. The latter proves its reader against five spellings and three negative controls **before** the corpus gate uses it; the former asserts the swap took (`:262-268`), that the population is non-empty (`:583`), that no fixture is vacuous (`:592-599`), and that **both** obligations are reached (`:604-618`). Copy these when fixing #3 and #4.

---

## What I could not check, and why

- **Nothing was executed.** Read-only ward: no builds, no floor, no `cargo`. The L1 #1 collision is derived by reading `hash_sequence` and `impl Hash for Value` and tracing the hasher write stream — it is a reading, not a run. **Drive it before acting on it:** two `assert_ne!(hash_value(&a), hash_value(&b))` lines settle it in seconds, and per my own standing note *a reading cannot see an execution defect*. If it comes back green, the defect is elsewhere in the stream and the test is still worth keeping.
- **I could not verify the `top ⊆ samples` hole reddens anything today** (#3) — it is a latent silent-shrink, argued from the `filter_map` at `mod.rs:549-557`, not observed.
- **`wat/` coverage was surveyed by namespace grep, not by resolution.** Per this repo's own CLAUDE.md, an unforced `def` body is never resolved, and the resolve gate covers only `:wat::rete::` names under `wat-scripts/`. My "26 defns, 5 named in tests" count for `wat/lint.wat` is a grep — and I explicitly checked and *retracted* an apparent finding there (`apply-fixes`, `lint-file` are reached transitively from `wat/lint.wat:579` and `:683`). Other `wat/` modules were not audited to that depth; **`capability`, `doctest`, `grep`, `query`, `Record`, `repl`, `runtime-meta`, `string`, `telemetry` returned zero `wat-tests` hits on my namespace grep and I did not follow any of them through to confirm.** Treat that list as a lead, not a finding.
- **I did not audit `benches/`, `examples/`, `crates/`, `wat-scripts/`, or `tools/`** — outside the stated scope.
- **4381 tests found by my scanner vs the 5420 in the brief.** The gap is real and unexplained: my brace-scan misses macro-generated tests, `wat-tests/` deftests (harnessed separately), and any `#[test]` whose attribute I did not pattern-match. My phase-1 census is therefore a **lower bound** on the population, and a monolithic body could sit in the ~1000 I never enumerated.
