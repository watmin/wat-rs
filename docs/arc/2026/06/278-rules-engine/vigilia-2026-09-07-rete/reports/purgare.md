# Purgare Ward — Cast Report

> Written verbatim as returned. The `&lt;` / `&gt;` / `&amp;` sequences are HTML-entity artifacts of the
> agent's own output and are preserved rather than rewritten.

**Target:** `src/rete/kernel/**` (excl. `tests/`) + `wat/rete/oracle/**` — 28 files, ~15.8k lines, read in full across 5 parallel passes (kernel top-level: `arm.rs`, `census.rs`, `node.rs`, `outcome.rs`, `insert.rs`, `mod.rs`; fire-core: `fire/mod.rs`, `fire/acc.rs`, `fire/delta.rs`, `fire/rules.rs`; `fire/pass/*.rs` (10 files); `session.rs` + `stratify.rs`; and the six `wat/rete/oracle/*.wat` files). Every file in the target was read in full by at least one pass.

## Findings

### Finding 1 — dead oracle helper `retain-supported`, superseded but not deleted

- **File/line:** `wat/rete/oracle/fire.wat:231-254` (doc header + `defn`)
- **Dead definition:** `(:wat::core::defn :wat::rete::retain-supported [facts &lt;- PersistentVector, supported &lt;- PersistentVector] -&gt; PersistentVector …)` — a foldl-based filter keeping only facts present in `supported`.
- **Evidence of death:** Its designed caller, `fire-support-fixpoint` (`fire.wat:342-375`), does not call it — it instead inlines the equivalent logic directly via `:wat::rete::factbag::retain` (`fire.wat:361-363`). Corpus-wide grep for `retain-supported` (`wat/`, `wat-scripts/`, `wat-tests/`, `tests/`) found exactly two hits total, both in `fire.wat` itself: the definition (line 242) and a self-referencing doc comment at line 327 ("see `retain-supported`: a plain filter"). Zero call sites anywhere. I independently re-ran this grep and confirmed 0 hits in `wat-scripts/`, `wat-tests/`, `tests/`.
- **How it went dead:** `docs/arc/2026/06/278-rules-engine/strike-factbag-one-owner/BRIEF.md:17` / `DESIGN.md:17,47` document the migration that replaced this helper with `factbag::retain` as part of giving `FactBag` a single owner. The migration executed in `fire-support-fixpoint` but the superseded function was left in place.
- **No rune present.** Not `public-api` (private oracle helper, no downstream consumers), not `trait-contract`, not exhaustive-match related.
- **Recommendation:** Delete `retain-supported` (`fire.wat:231-254`) and its self-referencing doc mention at `fire.wat:327`, or wire it up if some other caller was intended and dropped.
- **Removal cost:** Leaf deletion, one site.

### Finding 2 — `TerminationProof` enum's discriminant is never read

- **File/line:** `src/rete/kernel/stratify.rs:444-471` (enum def), consumed only at `stratify.rs:1045-1047`
- **Dead definition:** `enum TerminationProof { FiniteDomain, BoundedMeasure }`, held in `RuleEdge.proof: Option&lt;TerminationProof&gt;` (`stratify.rs:483`).
- **Evidence of death:** Both variants are unit variants (no payload) by explicit design (the doc comments at lines 449-452 and 468-469 say so, and justify why each variant's own *field* is absent). But the discriminant *between* the two variants is never inspected anywhere in the repo. The only two constructions are `TerminationProof::BoundedMeasure` (`stratify.rs:852`) and `TerminationProof::FiniteDomain` (`stratify.rs:946`); the only consumption is `stratify.rs:1045-1047`:
  ```rust
  if let Some(proof) = &amp;e.proof {
      let _ = proof;
      continue;
  }
  ```
  which only asks `Option::is_some()` — the bound value is immediately discarded with `let _ = proof;`. No `match` on `TerminationProof::FiniteDomain` vs. `::BoundedMeasure` exists anywhere. Repo-wide grep for `TerminationProof`/`FiniteDomain`/`BoundedMeasure` across `src/` returns only these 6 lines (445, 483, 779, 852, 946, 1044) plus the comment at 1044; grep across `wat/` returns zero hits — this is not a wat-oracle-mirrored type.
- **No rune present.** It is a private, non-trait, non-exhaustively-matched-elsewhere enum, so none of purgare's three structural exemptions apply. The doc comments explain why each variant carries no *field*, but not why the *variant itself* is unread — that gap is exactly what the rune exists to declare, and it's absent.
- **Recommendation:** Either (a) collapse to a payload-free marker (`Option&lt;()&gt;` or a unit struct) since nothing branches on which proof fired today, or (b) if the two-way distinction is meant for a near-term diagnostic (the comments gesture at "if a diagnostic ever wants to say 'bounded at n'"), add `// rune:purgare(future-fixture) — &lt;reason&gt;` tied to that planned diagnostic.
- **Removal cost:** Small cascade — one enum definition, two construction sites, one consumption site (all in the same file); no external cascade found.

## Runes encountered (non-purgare families, recorded per the ward's instruction to log every rune seen)

None of the following are `rune:purgare(...)` and none required a purgare verdict; recorded for completeness only:

- `rune:struere(lifetime-coupling)` / `rune:struere(invariant-coupling)` — `session.rs:20,61,85,97,109,616,1686,1716`
- `rune:excusare(no-falsifier)` — `session.rs:1834`
- `rune:sequi(performance-counter)`, `rune:perspicere(read-once)` — throughout `census.rs`, `arm.rs`
- `rune:lint(cited-name-absent)` — `fire/pass/alpha.rs:93-94,169-170` (re: `any_mixed`), `fire/pass/hash_join.rs:21-22` (re: `restore_parent`), `fire/pass/production.rs:7` (re: `compiled_rhs_cache`) — each verified consistent with current code by the auditing fork; none is a purgare finding (one documents a completed removal, the others document renamed/absent names in comments, which the wat-rs lint doctrine treats as exempt).
- `rune:temperare(simplicity-win)` — `fire/pass/filter.rs:57-58`, `fire/pass/root_join.rs:53-54`
- `rune:perspicere(intentional-structure)` — `fire/pass/alpha.rs:284`, `wat/rete/oracle/pass.wat:245`
- `rune:intueri(naming)` — `wat/rete/oracle/fire.wat:54-56`
- No `rune:purgare(...)` markers exist anywhere in the 28-file target.

## Convergence per sub-area

- `src/rete/kernel/{arm,census,node,outcome,insert,mod}.rs` — CONVERGED, no findings beyond the above.
- `src/rete/kernel/fire/{mod,acc,delta,rules}.rs` — CONVERGED, no findings.
- `src/rete/kernel/fire/pass/*.rs` (10 files) — CONVERGED, no findings (every pass module confirmed dispatched from `fire/delta.rs`, not an orphaned alternative strategy).
- `src/rete/kernel/session.rs` — CONVERGED, no findings.
- `src/rete/kernel/stratify.rs` — Finding 2 above; everything else confirmed alive.
- `wat/rete/oracle/**` (6 files) — Finding 1 above; everything else confirmed alive (walker chain, fixpoint chain, `FireStratAcc`/`StratifyAcc` fields, accumulate-pass and pass-layer helpers, `insert.wat`/`explain.wat` entry points all traced to live callers across `wat/`, `wat-scripts/`, `wat-tests/`, `tests/rete/`, and `src/`).

Every candidate that looked dead by a zero-or-low Rust-caller count was cross-checked against the `.wat` corpus (`wat/`, `wat-scripts/`, `wat-tests/`, `tests/rete/`) per the CRITICAL instruction; no case in this target turned out to be a Rust `pub(crate)` fn reachable only from `.wat` code that a naive Rust-only grep would have misclassified as dead — the two real findings are a dead `.wat` oracle definition (Finding 1) and a dead Rust enum discriminant (Finding 2), not a wat-reachability miss.
