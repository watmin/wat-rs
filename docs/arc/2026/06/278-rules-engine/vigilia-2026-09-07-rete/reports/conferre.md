# CONFERRE — Cast Report

> Written verbatim as returned. `&lt;`/`&gt;`/`&amp;` are HTML-entity artifacts of the agent's output.

**Working tree:** `/home/john/work/holon/wat-rs`, branch `grok-rete`, HEAD `bc0700d38` (2026-09-07 20:30:50 -0700). Confirmed live via `git branch --show-current` and `git log -1` this session — not a stale checkout.

**Target as scoped:** spec = `wat/rete/oracle/**`; code = `src/rete/kernel/**` excluding `tests/`.

**Method:** every doc-comment in the code tree naming an oracle function was enumerated by grep, then each claim was checked by reading the named oracle function's current body against the current Rust body. Work was split across four parallel sub-audits, each independently grounding both sides this session; the two highest-stakes findings were then personally re-verified against the live files.

## NEW FINDINGS

### Finding 1 — `rule_negates` misses negation nested under a top-level `:and`/`:or` in the ORACLE but not in the Rust. Active, dated "DOES mirror" claim contradicted.

- **Spec:** `wat/rete/oracle/stratify.wat:138-160` (`rule-negates`). For each top-level `form` in `lhs`, it checks only whether `form`'s own head is literally `":wat::rete::not"` (line 150). If so it descends via `negated-types-under`. If the head is anything else — including `":wat::rete::and"`/`":wat::rete::or"` — the fold contributes **nothing** (`acc` returned unchanged at `:159`); a `:not` buried inside a top-level `:and`/`:or` is invisible to the oracle.
- **Code:** `src/rete/kernel/stratify.rs:88-94` calls `negate_types(form, &amp;mut out, false)` on every top-level form. `negate_types` (`:103-122`) has an unconditional `Not(inner) =&gt; negate_types(inner, out, true)` arm and an `And(xs) | Or(xs)` arm recursing into every child **with `under_not` passed through** (`:108-112`). So a `:not` nested inside a top-level `:and`/`:or` IS found.
- **Which is likely right:** ambiguous on principle, but a real disagreement producing different `rule_negates` output → different stratum numbers for a rule with this LHS shape.
- **Why reportable now:** `stratify.rs:12-14` lists `rule-negates` among the functions the file is "a Faithful Rust port" of, and `stratify.rs:26-27` — dated **2026-09-07, today** — states: *"Every OTHER function in the list above was re-read against its oracle twin the same day and DOES mirror."* `rule-negates` is one of those "OTHER" functions. The dated claim is an explicit, current, in-scope assertion that it mirrors — and it does not, for this shape.
- **Reachability:** `wat-scripts/perf/grid/where-nested-combinators.wat:29` contains exactly `(:wat::rete::or (:wnc::A) (:wat::rete::and (:wnc::B) (:wat::rete::not (:wnc::C))))` as a top-level clause (in a `defquery`, proving the grammar accepts it). I found no existing `defrule` exercising this nesting, so I cannot additionally cite an observed differing stratum number, and the ward is read-only. Per the grounding clause that additional fact is **withdrawn** rather than asserted; the divergence itself is grounded in the two function bodies.
- **Recommended update:** **both** — decide the intended semantics first, then fix whichever side is wrong; if the oracle's narrower behaviour is intentional, scope the blanket "DOES mirror" claim the way the `stratify-sweep` divergence already is two paragraphs above it.
- **Severity: Level 1** — an explicit, dated, in-file "DOES mirror" assertion is false for a reachable clause shape.
- **Grounding:** `wat/rete/oracle/stratify.wat:107-160` vs `src/rete/kernel/stratify.rs:12-27, 88-94, 103-122`; shape at `wat-scripts/perf/grid/where-nested-combinators.wat:29`.

### Finding 2 — Stale characterization of `rule-produces`'s post-cure mechanism

- **Spec:** `wat/rete/oracle/stratify.wat:47-85` — `rule-produces` no longer does an inline `ast-name` + colon-strip; it evaluates the `:then` head, resolves through the PRIME convention, and calls `return-type-of`. Its header at `:52-60` states there is **no colon-strip fallback**.
- **Code:** `src/rete/kernel/stratify.rs:35-37`: *"Mirrors the inline `ast-name` + colon-strip done identically in both `rule-produces` and `rule-negates`."*
- **Which is right:** the code is fine — `produced_type` (`:63-81`) layers the real resolution on top. Only the comment's description of the oracle's CURRENT shape is stale; it predates, or was not updated after, the `rule-produces` cure.
- **Severity: Level 2** (latent drift — a maintainer trusting it would hunt for a colon-strip that no longer exists there).
- **Grounding:** `stratify.rs:35-37` vs `stratify.wat:47-85`. Independently reproduced by three separate readings this session.

### Finding 3 — Stale line-number citation for `seed-token`

- **Spec:** `wat/rete/oracle/pass.wat:76-90` — `seed-token`'s only definition.
- **Code:** `src/rete/kernel/fire/mod.rs:262`: *"Mirrors seed-token (wat:544-551)."*
- **The divergence:** `pass.wat:535-551` is inside `binding-extensions`'s combinator dispatch — an unrelated function. The *algorithmic* claim holds exactly; only the line numbers are wrong, most likely drifted when `pass.wat` was reorganized.
- **Recommended update:** fix to `:76-90`, or drop the line numbers as every other "Mirrors" comment in the same file does (they cite by name only).
- **Severity: Level 2.**
- **Grounding:** `fire/mod.rs:262` vs `pass.wat:76-90` and `:535-551`, both ranges read this session.

## KNOWN AND ALREADY CURED — reconfirmed, not re-reported

1. `rule-produces` resolving a `:then` head to a fn's return type — still correct at `stratify.rs:63-81` against `stratify.wat:47-85`.
2. `native_stratify_sweep`'s extra `exists_and_from_types` (+1) term — disclosed **identically on both sides** (`stratify.rs:219-231`, `stratify.wat:166-184`): same divergence, same test, same direction and example.

## EVERYTHING ELSE CHECKED: CONVERGED

Grounded pairs, both sides read this session, no finding: `native_stratify_fix` ↔ `stratify-fix`; `native_stratify` ↔ `stratify`; `native_rule_stratum` ↔ `rule-stratum`; `fire-stratified`/`fire-stratified-loop` per-stratum drive (Rust's network-sharing optimization is disclosed honestly with its own correctness argument); `merge-facts` value-dedup; `collect-derived` flatten; `fire-once$oracle` keeps alpha+beta; `insert$oracle`/`insert-all$oracle`; `eval_fire_rules_explain` / first-producer-wins support; six `#[cfg(test)]` reference-double "Mirrors `pass.wat`" comments (disclosed as non-shipping doubles); `fire/mod.rs:1455`'s export refusal; all Rust-to-Rust "mirrors eval_X" claims.

## OUT OF SCOPE (noted, not evaluated — spec restricted to `wat/rete/oracle/**`)

`node.rs:18,129`; `outcome.rs:35,42,124,193`; `mod.rs:15`; `session.rs`'s `wat_field_names_from!` macros — all cite `wat/rete.wat`. `fire/acc.rs:1,52` cites `wat/rete/acc.wat`. `fire/pass/accumulate.rs:50-51`'s disclosed "native 0 vs oracle 1" trailing-`:where` divergence against `accum-pass.wat` — already honestly disclosed.

## RUNE AUDIT

`grep -rn "rune:conferre" src/rete/kernel --include="*.rs"` (excluding `tests/`): **zero matches**. No divergence in this subsystem currently carries a conferre exemption.

## VERDICT

**NOT CONVERGED.** One Level 1 (Finding 1) and two Level 2 (Findings 2, 3). Every named oracle function was located and read; every claim not listed above held under direct comparison.
