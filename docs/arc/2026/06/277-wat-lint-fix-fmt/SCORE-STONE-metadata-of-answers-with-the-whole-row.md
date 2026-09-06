# SCORE — STONE: `metadata-of` answers with the whole row

No commit. Floor and clippy left to the orchestrator. No wat stdlib. No rule files. No formatter.

One shared emitter, both branches. Six doc-contract keys on the map. `:ret` is the pair the decoder reads. The acceptance is the round trip.

---

## RELAND — two lints in the stone's own new test file. No runes.

The substance was accepted. Two lints went red on `tests/reflection/probe_stone_metadata_of_whole_row.rs`. Neither rune'd.

- **`no_inlined_edn` (line 227).** The attr needle was `#[wat_intrinsic(":wat::rete::step-payload")]`, which opens with `#`. Reshaped to `wat_intrinsic(":wat::rete::step-payload")` — still unique, not EDN-esque.
- **`no_loose_string_assert` (four sites).** `:ret` description is now byte-identical `assert_eq!` against the `@ret` remainder. The printed `#wat.doc/Row` is `wat::assert_edn_matches_file!` against `probe_stone_metadata_of_whole_row__step_payload_row.edn` (captured, five `@arg`s, `:ret` pair, `:examples` populated). A `contains` on that row would have passed a reorder or a dropped key.

Row 5's `:args` count of **5** is unchanged. `cargo test --release --test lint -- tests_carry_no_inlined_edn tests_carry_no_loose_string_assert` → **2 passed**, this file absent. Whole-row tests **6 passed**.

Census corrections recorded as the RELAND named them: `yields` IS a field; HolonAST live count is **6**, not 9.

---

## REFUTE — a metadata map is not a hypervector. Reverted.

Copied the iv-c fixture's `HashMap :- [keyword HolonAST]` annotation onto the three new defns. Builder: *"how the fuck did it learn about holon-ast?"* Both spellings `--check`. The wall (`tests/lint/holon_is_vsa_only.rs`) is `src/` and `crates/*/src/` only — the `.wat` corpus is outside it, which is why the copy compiled.

**Reverted the three annotations to `:wat::core::Value`.** `--check` of the fixture is **0**. This stone's `.wat` files carry **zero** `HolonAST`.

Did **not** chase the rest. Did **not** widen the wall (STOP-1). Live `HashMap :- [keyword HolonAST]` annotations remaining in `.wat`: **6**, all pre-existing metadata-of fixtures (iv-c, iv-b1, spec_complete, reflection_parity ×2, dump-to-hex-metadata). The refute counted 9; the extra three are comments on those same fixtures. They need the wall's scope widened. Named, not this stone.

---

## Row 1 — it builds

`cargo test --release --test reflection -- probe_stone_metadata_of_whole_row` compiles and runs. `--check` of both new fixtures is **0**.

## Row 2 — the six keys are present; `:ret` is the pair

On `:wat::rete::step-payload` (registry branch): `:args` · `:examples` · `:see` present. `:deprecated` / `:alias` omitted when `None` (presence-as-nil is `MalformedDirective` — STOP-1 is the round trip, not a key count of empty/nil). `:ret` is `Value::Vec` of 2.

## Row 3 — THE ROUND TRIP CLOSES

`wat_doc::from_metadata(metadata-of :wat::rete::step-payload)` **succeeds**. The resulting `DocComment` equals `wat_doc::parse` of that entry's own `///` (`src/rete/step_payload.rs`): prose, added, ret_type, ret, args, see, deprecated, alias, five axes, examples. Empty vectors would pass a key count and fail this.

Wat branch (`:wat::core::sort`): `from_metadata` also succeeds.

## Row 4 — BOTH BRANCHES AGREE, KEY FOR KEY

`:wat::rete::step-payload` → `:defined-in Rust`. `:wat::core::sort` → `:defined-in Wat`. Shared keys (`:args` `:examples` `:see` `:ret` `:doc` `:added` and the five axes) are the same shape on both. One `emit_doc_contract` feeds both branches.

## Row 5 — the vectors are POPULATED

`:args` has **5**. `:examples` **≥1**. The example expr is a `WatAST` List (the real `@example` form), not a string.

## Row 6 — `:ret` carries BOTH halves

`[":wat::rete::DerivationStep", "the per-edge explain payload …"]`. Type keyword and description String.

## Row 7 — existing readers still work

`:arity` i64 · `:name` keyword · `:purity`/`:determinism`/`:totality` Enum. iv-c **PASS**. iv-b1 (key presence of `:added`/`:ret`) **PASS**. 14 `metadata_of` reflection tests **PASS**.

**STOP-5 re-census:** 0 production sites read `:ret` *out of a metadata-of map*. iv-b1 asserts presence only. iv-c's module doc still says `:ret -> Value::String`; the test *body* never matches `:ret`. Agrees with the DESIGN's 0. The shape change is the stone.

## Row 8 — `:yields` is NOT emitted from the registry branch

Absent on both branches.

**Named gap (census disagreement with DESIGN).** DESIGN: *"`IntrinsicEntry` has no `yields` field."* Disk: `pub yields: &'static [(&'static str, &'static str)]` at `src/intrinsic/mod.rs`. `DocComment` has it too. STOP-3 said do not emit from the registry (empty/fabricated is papering-over). STOP-2 said both branches or neither. Neither emits. Round-trip on step-payload holds because that entry has no `@yields`. A hologram-style row with `@yields` would decode with empty `yields`. Closing it is a later stone that emits from **both** sources.

## Row 9 — `:syntax` is NOT added

Absent on both branches.

## Row 10 — a doc row renders from the LOOKUP ALONE

`from_metadata` of the lookup → `wat_doc::print` → `#wat.doc/Row` carrying `:args` and `:examples`. No `:wat::intrinsic::examples` scan.

## Row 11 — negative control, committed

`deleting_examples_makes_from_metadata_refuse` — drop `:examples` from the returned map → `DocError::MissingExample`. `ret_as_a_bare_string_makes_from_metadata_refuse` — `:ret` as a String → `MalformedDirective { tag: ":ret" }`. Un-arm a `put` and these go red. Not a scratchpad probe.

## Row 12 — the formatter still dresses it

`tests/cli/metadata_of_example_formats.wat` loads the fmt rules, takes the example **from the lookup**, `format-source`. SOURCE widest **1153** → FORMATTED widest **72** (`<= 120`).

## Row 13 — nothing else moved

`wat/deporder.wat` `wat/spawn.wat` `wat/io.wat` `wat/fmt.wat` — git status clean. Not touched. The EXPECTATIONS hashes (`c69f0460` / `1c4bbb19` / `ba89b65c` / `cae52502`) are a different instrument; this stone did not rewrite those files.

## Row 14 — floor (ORCHESTRATOR)

Not run.

## Row 15 — clippy (ORCHESTRATOR)

Not run.

---

## Other named cuts

- **Alias + axes.** `:wat::rete::i64::>` carries `:alias :wat::i64::>` *and* the target's five axes (metadata-of has always reported the axes). `from_metadata` refuses that pairing (`AliasDeclaresAxis`). Round-trip uses a non-alias FQDN. Emitting `:alias` without axes would change `:purity` for alias rows (row 7). Not papered over.
- **Optional keys.** `:deprecated` / `:alias` are omitted when `None`. An empty vector or nil is illegal for those keys in the decoder.
- **FQDN spelling** is stored as stored. Printer converts to dotted. Not this stone.

## Files

```
src/runtime.rs                                          emit_doc_contract, both branches
src/intrinsic/mod.rs                                    args/ret_type/deprecated now live; dead_code attrs gone
tests/reflection/probe_stone_metadata_of_whole_row.rs   rows 2–11 (lints relanded)
tests/reflection/probe_stone_metadata_of_whole_row.wat  :wat::core::Value (reverted from HolonAST)
tests/reflection/probe_stone_metadata_of_whole_row__step_payload_row.edn  row 10 golden
tests/cli/metadata_of_example_formats.rs                row 12
tests/cli/metadata_of_example_formats.wat               lookup → format-source
```

---

## ORCHESTRATOR VERDICT — 2026-09-06 (after the RELAND)

**ACCEPTED, with one orchestrator fix.** Floor **5199 run, 5199 passed, 0 FAILED**. Clippy **0**.

| what | my own re-run |
|---|---|
| ★★★ the round trip, both branches | 6/6 `probe_stone_metadata_of_whole_row::*` **PASS**, including `from_metadata_of_the_lookup_equals_the_entry_doc` and `both_branches_agree_key_for_key` |
| ★★★ **the controls DISCRIMINATE** | removed the `:examples` `put` → **3 of 6 go RED**, the round trip among them. Restored |
| ★★★ row 12 · the lookup feeds the formatter | `example_from_the_lookup_formats_widest_le_120` **PASS** |
| ★★★ the refute was taken, not runed | **0** runes added; the EDN literal moved to a real `.edn` golden. `no_inlined_edn` and `no_loose_string_assert` both green |
| ★★ the `.wat` carries no `HolonAST` | three annotations back to `:wat::core::Value` |

## ⛔ CLIPPY CAUGHT WHAT THE FLOOR DID NOT

```
error: for loop over a single element
   --> tests/reflection/probe_stone_metadata_of_whole_row.rs:291
        for key in [":see"] {
```

A leftover from the reland: pulling `:args`/`:examples` out into exact compares left `:see` alone in
its loop. Floor **green**, clippy **101**. Inlined the two assertions with a comment saying why it is
not a loop. **Seventh time this arc the floor or clippy found what a targeted run could not** — and
the first time this arc it was clippy alone, with the floor fully green.

## ★★ TWO CENSUS CORRECTIONS THE STRIKE MADE, AND BOTH ARE MINE

- **`yields` IS a field** — `src/intrinsic/mod.rs:510`. The DESIGN asserted *"`IntrinsicEntry` has
  no `yields` field."* My enumeration windowed `awk 'NR>=415 && NR<=490'` and **clipped the struct**,
  then read the absence as a fact. The carried-but-unput count is **eight**, not seven — and I made
  that error **inside the correction that raised the builder's "five" to seven.** A truncating window
  making an absence unfalsifiable, in the very document that existed to fix a census.
  `[[feedback_a_truncating_pager_makes_absence_unfalsifiable]]`
- **`HolonAST` in `.wat` is 6, not 9.** My refute's pattern was looser than the site; the strike's
  6-live figure is right.

★ **The peer has now corrected my census twice in two stones** — the missed `starts-with?` files,
and now `yields` — and both times because I matched a PATTERN instead of enumerating the SITE. Four
instances of one class today, every one caught by review rather than by my own gate.

## The named cuts, all correct and all reported rather than buried

- **`:yields` emitted by NEITHER branch**, per STOP-2/STOP-3. The entry HAS the field, so closing it
  is a later stone that emits from both sources — not an empty vector to make a row pass.
- **`:deprecated` / `:alias` omitted when `None`** — presence-as-nil is `MalformedDirective` in the
  decoder, so omission is the decoder's own contract, not a gap.
- **The alias/axes pairing** — `from_metadata` refuses `:alias` beside axes (`AliasDeclaresAxis`),
  and `metadata-of` has always reported axes, so the round trip uses a non-alias FQDN. Named.

## What this unlocks

A `#wat.doc/Row` now renders **from one lookup** — `metadata-of` → `from_metadata` → `print` —
with `:args` and `:examples` in hand and no `:wat::intrinsic::examples` scan. Arc 255 step 4 does
that **576 times**; each one was a linear scan over 615 an hour ago.

## The board

```
✅ metadata-of answers with the whole row   six keys · :ret is the pair · round trip both branches
✅ three spellings, one seam · if · cond · Break.kind · Node.kind · :then · the fence · variant-name
   ⛔ 6 HolonAST annotations in .wat        the wall is src/-only and cannot see them
   ⛔ :yields on neither branch             the field exists; both sources or neither
   E · where a TRAILING comment goes        UNRULED
   ns-of / in-ns?                           NOTE in arc 109; blocked on 251.8b
```
