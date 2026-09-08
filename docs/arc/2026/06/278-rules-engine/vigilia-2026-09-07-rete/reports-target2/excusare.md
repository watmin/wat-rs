## EXCUSARE — Cast Report (TARGET 2 — the compile side and its spec)

> Written verbatim as returned, before any synthesis. Evidence only — this file carries NO status.
> HTML entities (`&lt;` `&gt;` `&amp;`) are artifacts of the agent's own output encoding.

## Enumeration — mine vs. the handed count

I re-derived every rune and every `#[allow(...)]` by reading each of the 25 target files directly (not trusting the pre-supplied grep).

| | Handed | Mine | Note |
|---|---|---|---|
| `rune:` lines | 33 | **34** | The extra one is `wat/rete.wat:524`. Your 33 appears to be a Rust-only count (14 Rust files carry runes, summing to exactly 33); the 5 wat spec files add 1 more, and my target list explicitly includes the wat files. |
| `#[allow(...)]` | 5 | **4** real attributes | Your grep is catching a **prose mention**, not an attribute: `validate/mod.rs:392` reads *"an alternative here was an `#[allow(clippy::too_many_arguments)]`, which silences the signal"* — inside a doc comment on `ClauseCtx`, explaining what they *didn't* do. `validate/typing.rs:44` similarly says *"an `#[allow]` would have been the patch"* — also prose. Real attributes: `clause.rs:71`, `clause.rs:75`, `compiled_cond.rs:246`, `validate/typing.rs:742`. |
| Files carrying | 15 of 20 (Rust) | **15 of 20** | Confirmed independently. |

**Total distinct exemption sites weighed: 36** = 34 rune-carrying lines + 2 rune-free bare `#[allow]`. Two of the 34 rune lines (`clause.rs:69`, `:73`) each pair with an adjacent `#[allow(dead_code)]` on the *same* field — 1 site each, not 2. **Enumerated 36, weighed 36.**

A genuinely useful discovery while enumerating: this repo has real, dedicated automated gates behind several `lint`-owned categories — `rete_citation_resolves.rs` (cited-name-absent), `no_loose_string_assert.rs` (whose own header says *"excusare audits the reason so 'legitimate' stays honest"*), `retired_name_justified.rs`, `no_unknown_sequi_rune.rs`, and `no_unknown_ward_rune.rs` (closed sets for `perspicere`/`purgare`/`excusare`, per `docs/CONVENTIONS.md`). Reading the gates' source is not running them — and it let me mechanically corroborate reasons rather than judge them for plausibility.

One category note: `docs/CONVENTIONS.md` has closed-set vocab tables for `sequi`, `perspicere`, `purgare`, `excusare` only. **`solvere`, `exigere`, and `temperare` have no closed-set gate anywhere in this tree.** That doesn't invalidate them — I verified each on the merits — but it is a documented gap of the same shape `sequi` had before its 2026-08-25 incident.

## Group A — `rune:lint(cited-name-absent)` — 16 sites, all HOLDS

Mechanically checked: grepped the whole repo (excluding `target/`) for each cited name/filename. All resolve to **zero code positions**, only the citing comment — exactly what each rune claims. Sites: `alpha_tree.rs:114` (`any_constrains`), `matcher.rs:499` (`token_element_compatible`), `reachability.rs:1462` (`head_is_boolean_rete_predicate`), `vocabulary.rs:575` + `:1670` (`NoMatchingArm`), `vocabulary.rs:996` (`infer_reduce`), `where_tree.rs:6` (`ShadowNode`), `where_tree.rs:7` (`tree.rs`), `expr_ir/eval.rs:125` (`EXEC_SP`), `expr_ir/mod.rs:55` (`exec.rs` — the real file is `eval.rs`, checked with `find`), `validate/error.rs:5` + `typing.rs:4` (`validate.rs` — `find . -name validate.rs` returns nothing), `validate/error.rs:48` + `typing.rs:286` (`keyword_constant_segment`), `validate/mod.rs:1144` + `typing.rs:88` (`check_field_at` — `grep "fn check_field_at"` empty).

Level L3. Closure: none — backed by a real gate.

## Group B — `rune:lint(loose-assert)` — 5 sites, all HOLDS

`reachability.rs:1798,1816,1942,1949`, `export.rs:2561`. Read the surrounding asserts: every one targets a `Debug`-formatted error containing a `Span`/rendered diagnostic, or a targeted multi-fact presence check inside a long EDN body — exactly the two categories `no_loose_string_assert.rs`'s own header names as legitimate. Level L2. Closure: none.

## Group C — single-instance runes, all HOLDS, individually verified

`matcher.rs:998` `rune:solvere(load-bearing-coupling)` — read the function; it does include keyword/unit/enum-unit at `:1006-1009` exactly as claimed. · `purity.rs:22` `rune:exigere(attested-arc)` — `docs/arc/2026/06/255-builtin-registry/` exists. · `wat/rete.wat:524` `rune:exigere(scope-affirmative)` — narrow, named claim; not a vague deferral. · `purity.rs:1782` `rune:temperare(simplicity-win)` — verified `rete_defn_cycle` is a separate call, not folded into the axis walk. · `purity.rs:2299` `rune:lint(retired-name)` for `:wat::core::sort'` — confirmed genuinely live in `wat/core.wat`, `src/check.rs`, `src/macros/eval.rs`, `src/collection/transform.rs`, `src/runtime.rs`; its dedicated gate requires a same-line rune, confirmed present. · `purity.rs:2587` + `:2593` `rune:perspicere(read-once)` — both verified single-use. · `expr_ir/eval.rs:85` `rune:sequi(ambient-context)` on `EXEC_ARENA` — genuinely a perf cache, not fire-domain state. · `expr_ir/eval.rs:896` on `KINDS` OnceLock — interned opcode table, computed once, immutable. · `expr_ir/eval.rs:413` `rune:perspicere(intentional-structure)` — structural readability call, category valid.

Level L3. Closure: none.

## Group D — the four already-rowed sites (my own verdict, as required)

**`clause.rs:68-71`** — **ILLEGITIMATE-AT-BIRTH — AGREE with purgare, independently confirmed.** `validate/mod.rs:561-562` reads `var` directly. The reason is false, and since `var` is genuinely read, `dead_code` would not fire on it anyway — the inert case is still ILLEGITIMATE-AT-BIRTH, not STALE-GUARD. Closure: remove both the rune and the `#[allow(dead_code)]`.

**`clause.rs:72-76`** — **HOLDS — DISAGREE with purgare's ILLEGITIMATE-AT-BIRTH.** purgare's stated basis is that the *category* ("trait-contract") is wrong for a plain enum field — that is a taxonomy objection, and it is outside excusare's remit (*"a check another ward can decide"*). Excusare's own question is only: does the reason earn the `#[allow(dead_code)]`? I independently confirmed it: `acc_form` is written once at construction (`clause.rs:354`) and never read as this struct field; fire genuinely gets its accumulate form through `node_named_ast(node, "acc-form")` in `kernel/arm.rs`. The reason is TRUE and structural, so under the Phase 1/Phase 2 gate this HOLDS regardless of which spell's category label sits above it. **This is the disagreement the cast asked me to surface.**

**`eval_test.rs:72-73`** — **HOLDS.** Verified both halves: all 4 call sites pass a freshly-built `Environment::new()`; `runtime.rs:4895`'s `eval_inner(ast, env, sym)` genuinely has the identical `(env, sym)` tail this function mirrors. The signature-uniformity argument is a real structural fact, not a convenience plea. (Not strictly an override — nothing is silenced — so it sits in excusare's second kind: a declaration standing where no automated Rust lint could raise.)

**`purity.rs:1513`** — agree this is a real gap, **but it is not an exemption for excusare to weigh** — no rune exists at this site, so there is no excuse text to test. I confirm conformare's premise independently: the doc at `:1495-1512` is a genuine, structurally-argued exception, and it carries no marker. It belongs on conformare's ledger, not mine.

## Group E — the two bare `#[allow(clippy::too_many_arguments)]`

**`validate/typing.rs:742`** — 8 parameters, above clippy's default threshold of 7, so the lint correctly triggers. The comment at `:736-741` gives a structural reason for not bundling: the eight are *"one destination, one value, the two registries needed to type them"* with no natural single subject, contrasted explicitly with `ClauseCtx` which does have one. **HOLDS.** L2.

**`compiled_cond.rs:246`** — **STALE-GUARD (candidate) — the guarded property changed.** `from_parts(ops, zip, n_slots, seed_reads, fact_bind, span, slot_names)` is **7 parameters, exactly** — counted by hand three times. Clippy's default `too-many-arguments-threshold` is 7 and the lint fires only when the count **exceeds** it; this repo's `clippy.toml` sets no override (read in full — it only configures `ignore-interior-mutability`). **The comment immediately above the attribute is the tell:** *"7 args since A3 (was 8: two arrays became the zip)."* The refactor that comment documents dropped the count from 8 (which triggers) to 7 (which, at the default, does not). Per the read-only rule **I did not run clippy and cannot confirm the lint has gone inert** — I assert only the arithmetic and the absence of an override. L2. Closure: if a clippy run confirms it no longer fires at 7 args, remove the `#[allow]`; if some threshold I could not find still makes it fire, this reverts to HOLDS.

## Aggregate

- **Total exemption sites weighed: 36.** **HOLDS: 34. Struck: 2** — `clause.rs:68-71` (ILLEGITIMATE-AT-BIRTH) and `compiled_cond.rs:246` (STALE-GUARD, candidate).
- One explicit **disagreement** with another ward's rowed verdict: `clause.rs:72-76`.
- One acknowledged-but-out-of-scope: `purity.rs:1513`.

**FINDINGS** — Enumerated 36, weighed 36. HOLDS 34. Struck 2.
