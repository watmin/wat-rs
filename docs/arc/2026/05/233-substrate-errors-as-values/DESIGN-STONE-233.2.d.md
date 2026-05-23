# Sub-DESIGN — Stone 233.2.d — substrate-symmetry uniform `list_span` threading

**Status:** ACTIVE (2026-05-23 night, post-compaction). Sub-DESIGN under arc 233 Stone 233.2.

**Driver:** the substrate-symmetry gap surfaced during Stone 233.2.c's `eval_edn_read` signature plumb. Originally filed as "arc 234 candidate" in INVENTORY § P (commit `e31b479`); post-compaction four-questions revealed the work belongs IN arc 233 as Stone 233.2.d. Its purpose serves 233.2's Provenance population thesis directly; splitting it out would be FM 11 deferral one level up.

## Why Stone 233.2.d (not arc 234)

Four-questions on "is this an arc or a stone?":

- **Honest?** — Arc 233's thesis is "substrate diagnostic-richness." Uniform `list_span` IS that thesis's foundation; Provenance variants can't be populated honestly without it. Splitting it out lets arc 233's INSCRIPTION read *"errors teach... mostly; 56% of arms still drop coordinates."* Deferral framing one level up — same FM 11 shape that was just collapsed within 233.2.c.
- **Obvious?** — Surfaced DURING arc 233 work (Stone 233.2.c audit), through arc 233's lens, in service of arc 233's goal. Arc-boundary minted for it would be administrative ceremony.
- **Simple?** — One umbrella, one INSCRIPTION, sequential stones.

The "arc 234" framing was scope inflation. Honest scope: Stone 233.2.d (uniform `list_span`) precedes Stone 233.2.e (AST-derived provenance — shifted from prior provisional 233.2.d slot, because SymbolBound's `head_span` needs uniform `list_span` availability to populate honestly).

## What 233.2.d does

Every eval fn dispatched from `dispatch_keyword_head` (src/runtime.rs) threads `list_span: &Span` as a **structural invariant**. ~245 dispatch arms updated to pass `list_span`; ~245 eval fn signatures gain the parameter. After this stone, the canonical signature is uniform across the dispatch table.

## The doctrine (load-bearing)

Every eval fn dispatched from `dispatch_keyword_head` threads `list_span` as a structural invariant. Same family as:

- `feedback_fqdn_is_the_namespace` — every name is namespaced
- `feedback_zero_mutex` — every shared-state path uses the three tiers

Asymmetry is not honest exception; it is accreted absence. The user audit during 233.2.c collapsed every "doesn't need list_span" category to zero under the four-questions ("would it act on `list_span` if given it?"). The collapse is documented at INVENTORY § P; the doctrine here makes it structural.

## Canonical signature template

```rust
fn eval_X(
    args: &[WatAST],
    list_span: &Span,    // structural invariant; always threaded
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError>
```

Parameter ordering matches Stone 233.2.c's `eval_edn_read` precedent (the one-arm preview that shipped before the doctrine was named). Sonnet mirrors the convention.

## Scope

- **~245 dispatch arms** updated to pass `list_span` to their called eval fn
- **~245 eval fn signatures** gain `list_span: &Span` parameter
- Pure mechanical sweep — most fns won't use the new parameter initially; the addition is structural
- The 5 already-tagged producers (`keyword/from-string`, `from-holon`, `edn::read`, `recv`, `try-recv`) stay in place — they already follow the convention
- The ~194 arms that already thread `list_span` stay in place

## Out of scope (affirmative scope-bounding)

- Populating `list_span` use sites — each fn decides separately whether/how to consume it (organic per-stone work)
- Renaming the parameter — `list_span` is the established convention; **do NOT propose** `call_span` / `form_span` / `list_call_span`
- Special forms with their own routing path (not dispatched from `dispatch_keyword_head`) — stay independent
- holon-rs — NOT touched
- New behavioral semantics — this is plumbing only
- AST-derived provenance (Stone 233.2.e)
- Errors-as-EDN (Stone 233.3)
- HARD CUT — no deprecation aliases

## Four-questions verdict

| Question | Verdict | Why |
|---|---|---|
| Obvious? | YES | One uniform rule; one signature template; mechanical replication |
| Simple? | YES | No shape decisions; pure plumbing; no semantic change |
| Honest? | YES | Closes the asymmetry the user audit collapsed under interrogation |
| Good UX? | YES | Future eval fns get `list_span` without ad-hoc plumbing |

## Sub-stone sequencing

Stone 233.2.d ships as ONE sweep — no sub-substones. Per FM 15 (substrate-as-teacher) and the precedent of arcs 111/112/113/114/115/117 / 163 slice 3e: substrate-wide mechanical sweeps land as one BRIEF; sonnet iterates from compiler errors. The cargo-fail-count IS the progress meter.

If the sweep proves larger than predicted (~245 arms is the upper-bound estimate; actual count may differ post-grep), sonnet may surface as honest delta + spawn a follow-up at the orchestrator's call. Default plan: ONE atomic sweep.

## Builds-on / unblocks

**Builds on:**
- Stone 233.2.c's `eval_edn_read` precedent — one-arm preview of the canonical signature shape
- The 5 already-tagged producers — they composed cleanly with their existing signatures; arc 234-equivalent work doesn't disturb them

**Unblocks:**
- **Stone 233.2.e** (AST-derived provenance) — `SymbolBound`'s `head_span` and `Literal`'s `span` need uniform `list_span` availability to populate honestly
- Any future producer addition — the template becomes the convention
- **Arc 232 resume** (defprotocol) — cleaner substrate to build dispatcher on; the protocol's call-by-name path benefits from uniform call-site coordinates

## Trap-door audit (lessons from arc 232.0 / 233.2.a-c)

- **NO scope expansion.** Plumbing only. Eval fns that don't use `list_span` today receive the parameter and continue not using it; do NOT refactor their bodies in the same stone.
- **NO renaming.** `list_span` is settled — do NOT propose synonyms. Per `feedback_no_new_types` + `feedback_wat_llm_first_design`.
- **NO touching the 5 already-tagged producers' bodies.** They follow the convention; only the ~245 other arms gain plumbing.
- **Substrate-as-teacher loop.** Sonnet's iteration shape: change dispatch arm → `cargo check` → fix called fn signature → `cargo check` → next. The compiler is the worklist. Per FM 15.
- **Empirical probe before BRIEF (FM 2-bis).** A short `tests/probe_substrate_symmetry_list_span_threading.rs` that asserts ≥440 eval fn signatures contain `list_span: &Span` (via grep over the source, run from a test). Probe ships pre-BRIEF, fails initially, flips PASS post-stone. Probe is permanent regression guard against future asymmetry.

## Risks + honest deltas

- **Signature ripple in tests.** Some tests use reflection or signature-introspection; if they break on the signature change, sonnet surfaces as honest delta in SCORE, does NOT paper over.
- **Existing fn bodies with other span parameters** (`head_span`, `arg_span`, etc.) — these STAY; `list_span` is ADDITIVE. The fn body picks which span it needs per call site.
- **Clippy may flag unused `list_span` parameters** — acceptable; `#[allow(unused_variables)]` per-fn is fine; the parameter is structural invariant. If clippy baseline goes up, document the increase in SCORE.
- **Compile-time cost** — trivial; no monomorphization implications.
- **Count uncertainty** — INVENTORY § P said "245 of 439 arms (56%) don't pass `list_span`." Actual numbers post-grep may differ. Sonnet reports actual counts in SCORE; the upper-bound estimate is for time-boxing, not a hard contract.

## Calibration prediction

Per FM 15 precedent (arc 163 slice 3e was a similar substrate-wide structural change; iteration count was 7 rounds; total wall-clock ~60 min after the BRIEF clicked):

- **Target runtime:** 60-90 min Mode A
- **Upper bound:** 150 min (STOP-3)
- **Confidence:** medium-high — mechanical but volume-sensitive; substrate-as-teacher iteration shape is well-established

The BRIEF is SHORT: "thread `list_span` uniformly per canonical template; iterate green." Sonnet doesn't need a per-arm enumeration; the compiler enumerates by failing.

## Cross-references

- `docs/arc/2026/04/109-kill-std/INVENTORY.md` § P — the original gap-surfacing; this sub-DESIGN supersedes its "arc 234 candidate" framing. § P stays as historical record; status section updates to point here.
- `docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.c.md` — the one-arm preview (eval_edn_read signature plumb)
- `docs/arc/2026/05/233-substrate-errors-as-values/DESIGN-STONE-233.2.md` — parent sub-DESIGN; sub-stone table updates to insert 233.2.d (this work) + 233.2.e (was old 233.2.d)
- `docs/arc/2026/05/233-substrate-errors-as-values/DESIGN.md` — umbrella; sub-stone count reflects the corrected slicing
- `docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 15 — substrate-as-teacher doctrine
- `docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 2-bis — probe-before-BRIEF for non-trivial work
- `feedback_fqdn_is_the_namespace` — structural-invariant family
- `feedback_zero_mutex` — structural-invariant family
- `feedback_refuse_easy_solutions` — the "intentional gap" framing was L2 reach
- `feedback_no_known_defect_left_unfixed` — known gap; in scope of active umbrella; ship don't defer
- `feedback_sonnet_writes_substrate` — protocol; sonnet writes; orchestrator briefs + scores + commits
- `feedback_inscription_immutable` — SCORE is a new file; DESIGNs are living
