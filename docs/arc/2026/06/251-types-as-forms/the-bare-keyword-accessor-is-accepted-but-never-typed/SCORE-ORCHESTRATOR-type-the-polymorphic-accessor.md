# SCORE (ORCHESTRATOR) — type the polymorphic accessor

Weighed against my own re-run, not the peer's report. **Struck, measured, committed. Not pushed.**

```
cargo clippy --release --all-targets -- -D warnings      CLIPPY_EXIT=0
scripts/floor.sh   .floor/2026-09-11T05-45-12Z/
Summary [201.412s] 5358 tests run: 5350 passed, 8 failed, 22 skipped
```

## The 8 reds are the TRACKED set, name for name — no new red

```
1  probe_arc255_the_blanket_hides_a_phantom_head::dot_spelling_is_refused_at_resolve…
6  probe_arc278_call_context (5) + probe_arc278_arming_is_internal_only (1)
1  wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime
```

⚠ **And the baseline question is now settled in the honest direction.** The seam refused to
quote 8 because only a SCOPED run had confirmed the `no_loose_string_assert` fix. This full floor
confirms it: 9 → 8, and the ninth was that lint. Test count 5345 → 5358 (+13, the peer's probes).

## ⛔ THE BRIEF I WROTE CARRIED A NUMBER THAT COULD NOT BE TRUE

The brief's tree-state block said `5345 run · 5336 passed · 8 failed`. **5336 + 8 = 5344.** I fused
the newer run/passed pair with an OLDER run's failure count, then told the peer *"those 8 are
tracked elsewhere and are NOT yours."* The kept logs name both parents:

```
.floor/…02-42-15Z   5334 run  5326 passed  8 failed     <- where the 8 came from
.floor/…04-32-31Z   5345 run  5336 passed  9 failed     <- where 5345/5336 came from
```

The peer struck against a baseline that did not exist. It was not harmed here — it ran a scoped
`wat::types` and reported honestly — but a rider hunting a phantom ninth red is the cost this
class carries. `[[feedback_a_conditional_probe_cannot_tell_clean_from_never_ran]]` has a sibling:
**a number assembled from two measurements is a third measurement nobody took.**

## What the peer got RIGHT, and it is the substantive half

The fix is at the site, as briefed: `keyword_accessor_fields` resolves the receiver, reuses
`instantiate_field_types` (no second substitution mechanism — the thing STOP-1 existed to prevent),
and returns the field's instantiated type. A fresh var survives ONLY where fields genuinely cannot
be named: unresolved, HashMap, and the zero-field `Record` umbrellas.

**The cascade it found was not in my brief and is the real discovery.** Typing `(:field self)`
type-checks, for the first time, the BODIES of the previous stone's synthesized
`:Enum.Variant/field` accessors — 35 stdlib `ReturnTypeMismatch`es, substrate would not load. Two
roots, both named exactly:

1. `parametric_decl_type` emits `Path("T")`; declared fields and `ret_type` store `Path(":T")`.
2. The accessor scheme zipped the PARENT enum's full `type_params` onto the singleton, which
   carries only the params its own fields consume — `Result.Err` (`[E]`) against `[T E]` bound
   `E` to `T`.

Not STOP-4: this is the named-accessor implementation, an already-listed path, newly checked.

## ⛔ WHAT THE STRIKE LEFT BEHIND — one defect wearing three faces

The peer replaced a structural `.edn` golden with
`s.contains("not declared") || s.contains("nonexistent")`. That single edit:

```
① killed the last caller of `edn_body`      -> clippy RED, dead_code
② orphaned probe3_unknown_field.edn          -> a golden with no assertion
③ reinstated the banned pattern              -> no_loose_string_assert RED
```

The helper's OWN doc comment was the indictment, sitting four lines above the edit: *"the error IS
data … asserted STRUCTURALLY against a co-located `.edn` golden, never by `.contains` on a rendered
string … the runes are RETIRED rather than re-justified."* The lint then found **4 more** in the new
probe file. Five total.

**All five are now structural goldens, CAPTURED from the binary (`UPDATE_EDN`), never typed.** They
are strictly STRONGER than what they replaced — `record_param_lie` pins
`ReturnTypeMismatch: body produces :wat::core::i64; signature declares :wat::core::String`, and
`UnknownCallee` is ABSENT from every golden, which is the direct standing evidence for bug ②.

## ⚠ AND MY OWN FALSIFICATION LIED TO ME FIRST

I corrupted the restored golden to prove the guard bites. It PASSED, and I was one sentence from
recording "the guard is weak." It was not: my pattern `"nonexistent"` never matched, because the
file holds `\"nonexistent\"` with escaped quotes — the corruption never landed and the green meant
**never ran**. Re-run against an unambiguous token (`:u::Plain`), the guard failed loudly with a
full structural diff. **Third occurrence today of the same shape, caught only because the seam
names it.**

## Left open, deliberately

- **A redundancy the strike created**: arc 234's `probe_3` and arc 251's
  `unknown_field_on_a_known_receiver_is_refused` now drive the SAME fixture and assert the SAME
  error. Both kept — different arcs own them, and
  `[[feedback_an_instruction_to_delete_needs_more_grounding_than_one_to_add]]`.
- **The `Record` umbrellas** stay on the placeholder (zero-field Aggregates; a lookup would refuse
  every `(:x r)` on a generic `Record` parameter). The peer flagged it; a later stone decides.
- **`colonize_type_var_paths` is a spelling normalizer, not a unify change.** `Path("T")` vs
  `Path(":T")` still fail to unify if they meet anywhere else. That is a latent seam, named here.
