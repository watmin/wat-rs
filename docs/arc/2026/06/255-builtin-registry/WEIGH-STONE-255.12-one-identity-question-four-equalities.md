# WEIGH — STONE 255.12: ACCEPTED. ⭐ The adversarial row caught the cure, and I proved it load-bearing.

Weighed against `d63134565` by independent re-run, **including three binaries built here**.
8 `src/` files, +18 tests. `git status --porcelain -- '*.wat'` = **0**. Not pushed.
⚠ An IDE diagnostic flagged a missing symbol at `probe_arc255_12_one_identity_question.rs:299` — the
file is **292 lines** and the symbol is nowhere in `src/`. **Stale.** `cargo build --release --tests`
→ **exit 0.** Discarded, not carried.

## ⭐⭐ THE DANGEROUS DIRECTION WAS REAL, AND THE GUARD IS LOAD-BEARING — I PROVED IT

The brief's one demand was an **adversarial divergence `type_denotation` would COLLAPSE**. ⭐ **It
caught the rider's own first draft.** I did not take that on report — **I deleted the carve-out
myself and rebuilt:**

```
carve-out DELETED  → (defrecord :p::Inf [n <- :wat::type::Infer])
                     (defrecord :p::Inf [n <- :wat::core::Infer])   → rc=0  ⛔⛔ ACCEPTED
carve-out RESTORED →                                                   rc=1  DuplicateType
cross-spelling legitimate case                                         rc=0  ✅ still passes
```

⛔ **Without it, an ILLEGAL second declaration is swallowed as a benign no-op** — the exact
red→false-green this stone risked. ⭐ **And the carve-out already existed inside
`check::format_type_path` — A RENDERER.** Every other consumer of `type_denotation` was collapsing
`Infer` silently; it only became visible when an **equality** was pointed at the same door.

⭐ **`type_defs_same` is `==` over denotation-normalized defs, NOT a hand-written field walk** — so
`nature`, `purity`, `restrictions`, `type_params`, variant names and field order stay byte-compared.
**That shape is what makes the permissive direction survivable**, and it is the right call.

## ⭐⭐ SITE 3 — THE 8d-ii STOP, PROVEN ON TWO BINARIES, WITHOUT CONVERTING THE STDLIB

```
PRE-CURE   (:wat::edn::validate 42     :wat::type::i64) → Invalid {:expected ":wat::core::i64" :got "Integer"}
POST-CURE  (:wat::edn::validate 42     :wat::type::i64) → Valid
POST-CURE  (:wat::edn::validate "nope" :wat::type::i64) → Invalid {… :got "String"}   ⭐ STILL REFUSED
```

⛔ The pre-cure message is **self-contradicting** — 42 *is* an Integer and the target *is* i64; the
comparison failed on **spelling**. ⚠ **Neither state is visible to `--check`.** ⭐ **The
wrong-type row is the non-vacuity and it holds: the cure did not over-widen.**

## Gates re-derived

| gate | mine |
|---|---|
| delta — **via the rider's new `scripts/replay/delta.sh`** | **160 / 157 · NEW 3 · ⭐ RECOVERY 0** · `paths=179 missing=0`, list-sha printed. **4 → 3**; `wat/source.wat` closed |
| floor | RED **5984/2** captured with `ARM.txt`, then **5986/5986** green |
| clippy / census | exit 0 · `no STOP-8` |
| `.wat` converted | **0** |
| adversarial + coerce rows | ⭐ **verified above on three binaries** |

## ⛔ THE FLOOR RED — arm 2 is NOT this diff, and it is a FINDING, not a flake

The rider cured its own arm 1 (a `contains()` the house lint caught) by asserting **structurally** on
`MacroErrorKind::DuplicateMacro` — stronger than the lint demanded. ⭐ **Arm 2 it did NOT dispose
of**, and its analysis holds on inspection:

- `harvest_cost.rs:337` is a **wall-clock apportionment bound**, and ⭐ **the test's own comment says
  so**: *"Bounds are deliberately loose (0.5x–2x) because these are wall clocks on a shared runner."*
- Its timed closures call `PVec::iter`, an inline `matches_class`, `PMap::from_pairs`, `Value::clone`
  — ⭐ **none of this stone's 8 changed files is on that path**, verified here.
- Last touched **2026-09-21 by a different stone**.
- `w > h` is arithmetically impossible if each closure measures its name; the unsound reading is `w`'s.

⛔ **BUT "timing" IS NOT A DISPOSITION IN THIS REPO, AND I AM NOT RECORDING IT AS ONE.** ⭐ **The
finding is about the TEST:** a ratio over single-iteration wall clocks, sampled while 5,986 tests run
in parallel, **cannot be a floor gate** — `[[feedback_a_wall_clock_ratio_is_not_a_gate]]`, which this
repo already learned once. **It will fire again. It needs a stone, not tolerance.** ⭐ The rider's
*"Run 2 does not erase run 1"* is exactly right and I am keeping it.

## ⭐ I NARROWED THE RIDER'S OWN MOST-WANTED OPEN QUESTION

It flagged: *"I did not enumerate `:wat::type::` to prove `Infer` is the only wrongly-collapsed
name — this is the one I'd most want checked."* ⭐ **Measured here:**

- 70 `:wat::type::` names; my counterpart grep leaves ~35 candidates, ⚠ **but that grep is too crude
  to conclude from** — most are error-struct EDN tags (`#wat.type/MalformedDecl`), never annotation
  members, and the counterpart namespace is not always `core` (`:wat::WatAST`).
- ⭐ **The structural answer:** I tested four (`Seq`, `Stream`, `HolonAST`, `Thermometer`) — **all
  refused as `UnknownNamedType`, by the ANNOTATION WALL, before the duplicate comparator runs.**
  **`Infer` is dangerous precisely because it PASSES that wall while having no `core` counterpart.**
- `INFER_TYPE_PATH` is the **only** such constant, with 4 dedicated carve-out sites.

⛔ **Not a proof — a narrowing.** The remaining risk is any *other* name that passes the annotation
wall with no counterpart. **Materially smaller than "35 candidates", and still open.**

## The brief was wrong four ways — the twentieth correction

1. ⭐⭐ **Site 2's mechanism.** `is_reference()` is **not** the blocker — the cross arm compares
   `canonical_identity`, and `"<-"` ≠ `":-"`, returning false first. **My lead was wrong and it was
   refuted three ways**, including restoring *exactly the two arrows inside Ngram's template*.
2. **My arrow count was file-level** — three of five are the macro's own param vector, consumed at
   parse. **Only the two inside the template matter.**
3. ⛔ **The four-site table was INCOMPLETE — site 5** (`register_stdlib_types_replacing`), found by
   ⭐ **censusing the registration gate's own `Equivalent`/`Divergent` decision sites instead of
   grepping `==`.** A better instrument than the one I handed over.
4. **"The 2 remaining stdlib files should close"** — **one did.** `Ngram` cannot: the collision is the
   `include_str!`-baked copy meeting the converted on-disk one, **proved path-independent**. Curing
   it means teaching a blind walker `<-` ≡ `->` ≡ `:-` inside a template — **two macros emitting
   different surface forms become one.** ⭐ **Correctly refused. 8d-ii's job / builder's call.**

## ⭐⭐ BOTH INSTRUMENT DEBTS CLOSED — after three stones

**`scripts/replay/delta.sh`** — RECOVERY standing; non-zero exits 9. ⭐⭐ **It caught a defect in
ITSELF on the first run:** `wat` reads stdin, so the draft's `while read … < "$LIST"` was eaten after
10 of 179 files **and printed a plausible-looking delta.** Now `xargs` with `/dev/null` stdin, and it
**refuses to report a partial measurement.** ⭐ *A instrument that fails loudly on its tenth file is
worth more than one that has never been wrong.*

**`src/freeze/pass_order.rs`** — 14 passes announce themselves; a unit test pins the sequence.
⭐ **It has FAILED once:** moving `resolve_references` ahead of `normalize_symbol_refs` reds it **and
the mutated program still froze clean — nothing else would have noticed.** It carries its own
non-vacuity test (`expand < normalize < check`) and ⭐⭐ **the instruction that makes it a gate rather
than a pin: do NOT re-order the array to match the code until you have re-read every "post-step-7 ⇒
unreachable" disposition.** It also corrected the documented pipeline
(`normalize_stored_function_bodies` runs **twice**; the header lists it once).
⛔ **Its doc states what it cannot see: it pins ORDER, not the CODE-vs-DATA half that actually misled
255.9 and 255.10.** That needs a post-normalize residue census — **a stone.**

## VERDICT

**ACCEPTED.** Pushing. ⭐ **The 8d-ii blocker is cured and proven on two binaries.**

Queue: ⛔ **the `harvest_cost` wall-clock gate** (it will fire again) · the post-normalize
code-vs-data residue census · the 416-test rete remainder · the 2 arc-170 files (**not** sites 3/4 —
a type-parameter substitution question, named but undiagnosed) · the shape-B/D sweep ·
`:wat::keyword::canonical-identity` · 255.8's wrong-join hole · **then 8d-ii (4th draw)** · 8d-iii.
⛔ **Builder's call:** `Ngram`'s mixed-dialect collision · `is_quasiquote_form`'s asymmetry ·
`set-redef!`/`set-eval-redef!`.
