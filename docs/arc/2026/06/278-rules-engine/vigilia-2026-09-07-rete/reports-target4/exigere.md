# EXIGERE — Cast Report (TARGET 4 — the test corpus) — **FINDINGS**

> Written verbatim as returned, before any synthesis. Evidence only — this file carries NO status.

### 1. What I swept — re-derived numbers

| measurement | handed-down | re-derived | verdict |
|---|---|---|---|
| files | 306 | **306** | matches |
| lines | 38,058 | **38,058** | matches |
| `#[test]` fns | 613 | `grep -rno '#\[test\]' … \| wc -l` → **615** | **delta of 2**, not chased further — immaterial to this vigilia's quarry |
| TODO-family | 0 | **0** | matches — confirms the target-4 zero |
| `deferred` | 7 | **7**, read every one | matches |
| `out of scope` | 5 | **5**, read every one | matches |
| `banked` | 6 | **6**, read every one | matches |
| `follow-?up` | 1 | **1**, read | matches |
| **total ≈19** | ≈19 | **19** | matches exactly |

I then swept beyond the handed-down vocabulary against the spell's own full L1/L2 phrase list (`will be`, `would require`, `could extract/refactor/RAII`, `not yet implemented/supported`, `pending arc`, `next arc`, `punted`, `scratch arc`, `small follow-up`, `left for`, `should be/land/consider`, `if pressure/demand`, `when needed/surfaces`, `future arc/cleanup/polish/self/caller`, `land later`) → **4 hits**, and separately for `unimplemented!`, `todo!(`, `stub`, `placeholder`, `TBD`, `WIP` → **3 hits**. All read below.

### 2. The dismissal list (24 of 26 hits examined)

**`deferred` (all dismissed):**
- `probe_arc278_P12_explain_walk.rs:10` "a deferred computation" — CS term for a thunk; present-tense doc of current behaviour.
- `probe_arc278_fixpoint_round_cap.wat:5` "NAMED and deferred a cap for" — past event; the cap now exists and this file proves it.
- `probe_constructor_meta_kwargs_undersupply.wat.bad:6` "the raise deferred to whenever" — describes the PAST BUG this negative fixture guards against.
- `probe_arc278_1b_compile.rs:49` "what the first attempt deferred" — past-tense; the gap is closed by this test.
- `probe_arc278_fixpoint_round_cap.rs:3` "the stone … deferred, and it named the shape exactly" — resolved history.
- `src/rete/kernel/tests/pass_semantics.rs:730` "the deferred-activate loop must not run" — names an actual code mechanism, not a promise.
- (Both operator-pre-dismissed hits re-read and confirmed: `right_index_counter_invariant.rs:454` correction-of-a-past-`#[ignore]`, and `P6_delta_asymmetric_join.rs:60` engine-round domain semantics.)

**`out of scope` (3 dismissed, 1 recorded-not-relitigated):**
- `probe_arc278_P12_explain_walk.rs:16` — inside the `rune:exigere(scope-affirmative)` block; already HOLD-adjudicated by `excusare`.
- `probe_arc278_2a_alpha_match.rs:13` "stone-6 escape (out of scope here)" — verified `stone-6` is a real on-disk family (`DESIGN-STONE-6a-purity-inference.md`, `DESIGN-STONE-6b-where-test.md`); a legitimate navigational cross-reference.
- `probe_arc278_concurrent_retes.rs:20,38` "Multithreaded performance is out of scope for this gate" — present-tense methodology declaration, reasoned inline ("a duration assertion on a shared box is a flake generator").

**`banked` (all dismissed as self-clearing):**
- `probe_arc278_match_arm_is_not_a_call.rs:34,36,41,186,196` — all describe `experiri-then-match.wat`, banked 2026-08-30 with `rune:lint(red-by-design)` and its own disposal clause ("if this file ever loads, D5 is cured and the rune must go with it"). The rune is now gone, the file loads, and `the_banked_d5_repro_pair_both_load` turns it into a standing regression test. **The self-clearing machinery working as designed — closed, not open.**
- `right_index_counter_invariant.rs:452` — past-tense condemnation of an erroneous `#[ignore]`; cure landed, `#[ignore]` removed, "this runs on the floor."

**`follow-up`:** `probe_arc278_return_type_of.rs:7` "de-masking follow-up" — describes a completed fix.

**Extended vocabulary sweep (all 7 dismissed):** `probe_arc278_nested_wall.rs:24,25` "should be reported" (normative test-design language); `probe_arc278_49_one_core_covers_the_surfaces.rs:108` (assertion-failure wording); `probe_arc278_55_slice_one_undefined_mandatory.rs:7` "NOT implemented as a kwargs defmacro (… out of this slice's scope …)" — present-tense architecture explanation of what the function IS versus an alternative it deliberately is NOT; `tests/rete/mod.rs:2` "a thin include! stub" (technical description of the file's real role); `probe_arc278_P6_delta_asymmetric_join.rs:28,68` "FIRE_VERB placeholder" (string-template substitution token).

### 3. The `banked` verdict

**Every `banked` in this target carries the self-clearing machinery — none merely claims the word.** All 6 hits resolve to exactly two passages, and both describe a banking episode that has *already* closed. No open/live `banked` claim exists in this target lacking that machinery.

### 4. Findings (2, both L1, both soft/borderline)

**Finding 1** — `src/rete/kernel/tests/termination_verdict.rs:5`
- Prose: *"surfacing it would need a new `(:wat::rete::CompileOutcome)` variant behind the outcome wall, which is affirmatively out of scope for the strike that split this type."*
- Level: **L1** (scope-defense framing)
- Checked: grepped the whole 221-line file for `arc`, `strike`, `DESIGN`, `rune` — **no arc number, no `DESIGN.md` path, no verifiable tracker anywhere.** "The strike that split this type" names no artifact a fresh reader can locate.
- Direction: the reasoning given reads as a *permanent* architectural invariant, not deferred work — so either drop the "out of scope" framing and state it as a standing invariant, or name the arc and promote to `rune:exigere(scope-affirmative)`.

**Finding 2** — `tests/rete/probe_arc278_P4c_native_retraction.rs:8-9`
- Prose: *"the support store only buys O(delta) retract for a PERSISTENT cross-fire streaming engine, a deferred surface"*
- Level: **L1** (disposition-deferral)
- Checked: grepped the 62-line file for `arc`/`rune` — only self-reference to "Arc 278" in the title; **no tracker named** for the hypothetical persistent-streaming-engine feature.
- Direction: the primary claim ("there is NO … cascade to build") is honest present-tense fact; the parenthetical labelling the hypothetical optimisation "a deferred surface" is the only weak spot — either reword to something non-committal or name the arc.

### 5. The one `rune:exigere` encountered

`probe_arc278_P12_explain_walk.rs:15` — `rune:exigere(scope-affirmative) — Out of P12's scope; DAG-sharing for fan-in is rejected (first producing token only). Tracked as Out of scope in docs/arc/2026/06/278-rules-engine/DESIGN-STONE-P12-explain-walk.md.` Verified: that file exists on disk. Already adjudicated HOLD by `excusare`. Recorded, not re-litigated.

**FINDINGS**
