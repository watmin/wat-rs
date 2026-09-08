## PROBARE — Cast Report

> Written verbatim as returned, before any synthesis. Evidence only — this file carries NO status.
> Status for every row lives in `FINDINGS.md`, and nowhere else.

## SCOPE

Read in full: all 6 oracle files (`wat/rete/oracle/{accum-pass,explain,fire,insert,pass,stratify}.wat`, 2,320 lines) and, at header/module-doc depth plus targeted body reads, all 22 kernel Rust files (13,451 lines). Also read `tests/lint/rete_header_claims_are_asserted.rs` in full (308 lines) per the cast's instruction, as the reference for "what is already mechanically held to its word."

Commands run (all against this session's HEAD, 28-file list only):
- `wc -l` over the 28 files → 15,771 total (matches the cast's stated scope).
- Per-file `grep -c '^\s*;;'` / `'^\s*$'` for the 6 `.wat` files, plus `grep -c '^\s*('` for the literal spell-defined form measure.
- Per-file `grep -cE '^\s*(//|\*)'` / blank for the 22 `.rs` files, and a declaration grep for Rust top-level declarations.
- `grep -nE 'TODO|FIXME|XXX|HACK|unimplemented!|todo!\('` over all 28 files → zero hits (confirms prior art independently, third measurement).
- `grep -n "rune:"` over all 28 files, then `grep -n "rune:probare"` → zero `probare`-category runes anywhere in target.
- Targeted `grep -n` / `sed -n` to verify two specific header claims against their referents.

## CLAIM-FALSIFIABILITY ANALYSIS (the sharp question)

**Finding 1 — L1. `src/rete/kernel/outcome.rs:22-25` cites a caller count and a line number for `fire_fixpoint_delta_armed`, and both are wrong today, and neither is covered by `rete_header_claims_are_asserted.rs`.**

The module doc reads (verbatim, lines 22-25):

> "Pushing the enum down into `fire_fixpoint_delta_armed` itself is the tidier shape and is deliberately NOT done yet — it has three callers (`fire-once`, `fire-rules`, and the query path at `fire/rules.rs:425`), so it becomes worth doing when the second door arrives, not on the strength of one."

I grepped every call to `fire_fixpoint_delta_armed(` (excluding its own `pub(crate) fn` definition at `fire/delta.rs:271`) and found **four** call sites, not three:
- `src/rete/kernel/fire/mod.rs:1177` — the fire-once path.
- `src/rete/kernel/fire/delta.rs:266` — inside `fire_fixpoint_delta`, the **non-stratified** fire-rules fast path (itself called once, from `fire/rules.rs:452`).
- `src/rete/kernel/fire/rules.rs:193` — inside `fire_rules_stratified` (the stratified per-round loop) — the **other** fire-rules path.
- `src/rete/kernel/fire/rules.rs:433` — the query path.

`fire/rules.rs:425` (the cited line) contains no call to `fire_fixpoint_delta_armed` at all — I read lines 400-435 and it is inside unrelated `q_ids`/`q_front` set-building code. The actual query-path call is 8 lines later, at `fire/rules.rs:433`. Confirmed by reading `fire_rules_on_session` (`fire/rules.rs:648-700`): it runtime-dispatches to `fire_unstratified` (delta.rs route) or `fire_rules_stratified` (rules.rs:193 route) depending on `max_s`, so "fire-rules" genuinely has two distinct call sites in the source, not one, even if one chooses to bucket them as a single logical door.

I checked `tests/lint/rete_header_claims_are_asserted.rs` end to end: it pins `fire/mod.rs`'s `#[cfg(test)]` set, `FireCtx`'s field count, `export.rs`'s lossy-field sites, `alpha_tree.rs`'s `AlphaRoots`/`root_for` shape, and the termination verifier's one call site. `outcome.rs` and `fire_fixpoint_delta_armed` appear nowhere in it. This is precisely the shape the gate's own header warns about — a specific, checkable structural claim (a caller count plus a `file:line` citation) sitting in prose with nothing holding it to its word, and it is checkably wrong today, not just liable to rot.

**Finding 2 — L2. The "Session stays 8 fields" claim is true today but has zero mechanical backing, unlike its sibling `FireCtx` claim.**

`src/rete/kernel/mod.rs:15` documents: `## Session record (8 fields, declaration order — wat/rete.wat defrecord Session)`, and `src/rete/kernel/arm.rs:706` repeats it as a load-bearing invariant: `(DESIGN-STONE-intern-zero-mutex THE ONE CONTRACT: Session stays 8 fields; ...)`. I verified `wat/rete.wat:199-207`'s `defrecord :wat::rete::Session` has exactly 8 fields today — the claim is currently **true**. But `rete_header_claims_are_asserted.rs::fire_ctx_field_count_matches_its_doc` exists to pin the *sibling* struct's field count (`FireCtx`, 14) for exactly this reason ("its doc said thirteen while the struct held fourteen"); no equivalent test exists for Session's 8, in `tests/lint/` or anywhere I found under `tests/`. This is the same class of unverifiable totality the gate's own header names as the cost center, sitting one door over from the one already cured. Not a lie today; a structural mumble waiting to become one.

**Minor, softer note (not scored):** `src/rete/kernel/stratify.rs:602-616` justifies `MAX_PROVABLE_FACT_POPULATION = 1_000_000` with a dated, hedged memory measurement (`Measured 2026-08-29`) and states a design floor — "never below the corpus's own 40_000." Only the constant's existence and the above-cap refusal behavior are tested (`tests/rete/probe_arc278_fixpoint_round_cap.rs:340`); nothing asserts the constant stays >= 40,000. This is the honest, dated, hedged form the gate's own header recommends, so I am not scoring it — flagging only because the design invariant it states has no gate either, should a future edit shrink the constant silently.

**What I looked for and did not find:** described/hollow forms. Zero `TODO`/`FIXME`/`XXX`/`HACK`/`unimplemented!`/`todo!()` across all 28 files (grep, this session, third independent measurement of the same zero the cast already recorded). I read every line of the 6 oracle `.wat` files and spot-read bodies in the Rust kernel files cited above; no stub bodies, no `pass`-equivalents. The two already-rowed hollow/dead items (`retain-supported`, `TerminationProof`'s undiscriminated variants) were the only hollow shapes visible, and both are prior art — not re-reported.

## RUNES

Zero `rune:probare(...)` markers exist anywhere in the 28-file target (grep, this session). Other rune categories are present and dense (`rune:sequi`, `rune:struere`, `rune:perspicere`, `rune:temperare`, `rune:circumspicere`, `rune:excusare`, `rune:lint`) — these belong to other wards' vocabularies, not probare's, and I left them alone per the cast (including the disputed `rune:intueri(naming)` at `fire.wat:54`, untouched).

## PER-FILE RATIO TABLE

Oracle (`.wat`) — code = non-comment/non-blank lines (the literal "line begins with `(`" measure undercounts this multi-line Lisp style severely, since most continuation lines don't open with `(`; I report both but verdict on the honest one):

| File | Total | Comment | Code(non-blank,non-comment) | Literal `(`-lines | Ratio (code:comment) | Verdict | Exempt? |
|---|---|---|---|---|---|---|---|
| accum-pass.wat | 323 | 56 | 259 | 144 | 4.63:1 | substance-rich | — |
| explain.wat | 118 | 31 | 84 | 41 | 2.71:1 | mixed | doc-comment-rich (real bodies) |
| fire.wat | 597 | 183 | 390 | 151 | 2.13:1 | mixed | doc-comment-rich (real bodies, incl. corrected-belief blocks) |
| insert.wat | 112 | 44 | 61 | 24 | 1.39:1 | mixed | doc-comment-rich |
| pass.wat | 810 | 125 | 654 | 356 | 5.23:1 | substance-rich | — |
| stratify.wat | 360 | 99 | 246 | 127 | 2.48:1 | mixed | doc-comment-rich (corrected-belief block at 28-38) |

All six: every form I read is Expressed (real recursive/foldl bodies, no stubs). Recommendation: ship as-is.

Rust kernel — code = non-comment/non-blank lines (Rust's literal "declaration count" measure produces nonsense ratios like 1:57 for files that are one large function moved out of a bigger loop with heavy rationale, so I do not use it as the verdict basis):

| File | Total | Comment | Code | Ratio | Verdict | Exempt? |
|---|---|---|---|---|---|---|
| arm.rs | 1469 | 355 | 1049 | 2.96:1 | mixed→rich | doc-comment-rich |
| census.rs | 910 | 302 | 520 | 1.72:1 | mixed | doc-comment-rich |
| insert.rs | 258 | 78 | 164 | 2.10:1 | mixed | doc-comment-rich |
| mod.rs | 44 | 25 | 17 | 0.68:1 | prose-heavy (by literal ratio) | **exempt — pure re-export index; purpose matches contents exactly** |
| node.rs | 230 | 32 | 175 | 5.47:1 | substance-rich | — |
| outcome.rs | 251 | 95 | 139 | 1.46:1 | mixed | doc-comment-rich (see Finding 1) |
| session.rs | 1848 | 397 | 1335 | 3.36:1 | substance-rich | — |
| stratify.rs | 1067 | 508 | 519 | 1.02:1 | mixed (borderline) | **doc-comment-rich — dominated by corrected-belief/design-rationale blocks, the house style the cast names explicitly** |
| fire/acc.rs | 582 | 105 | 448 | 4.27:1 | substance-rich | — |
| fire/delta.rs | 903 | 253 | 604 | 2.39:1 | mixed | doc-comment-rich |
| fire/mod.rs | 2334 | 480 | 1750 | 3.65:1 | substance-rich | — |
| fire/rules.rs | 894 | 316 | 532 | 1.68:1 | mixed | doc-comment-rich |
| fire/pass/accumulate.rs | 326 | 68 | 253 | 3.72:1 | substance-rich | — |
| fire/pass/alpha.rs | 422 | 140 | 269 | 1.92:1 | mixed | doc-comment-rich |
| fire/pass/filter.rs | 294 | 87 | 203 | 2.33:1 | mixed | doc-comment-rich |
| fire/pass/filter_after_join.rs | 307 | 30 | 274 | 9.13:1 | substance-rich | — |
| fire/pass/hash_join.rs | 615 | 164 | 434 | 2.65:1 | mixed | doc-comment-rich |
| fire/pass/join_after_filter.rs | 119 | 33 | 84 | 2.55:1 | mixed | doc-comment-rich |
| fire/pass/mod.rs | 185 | 74 | 103 | 1.39:1 | mixed | doc-comment-rich |
| fire/pass/production.rs | 151 | 57 | 89 | 1.56:1 | mixed | doc-comment-rich |
| fire/pass/root_join.rs | 100 | 36 | 61 | 1.69:1 | mixed | doc-comment-rich |
| fire/pass/round_census.rs | 142 | 45 | 95 | 2.11:1 | mixed | doc-comment-rich |

Recommendation on all 22: ship as-is. None is a wish; every "mixed"-band file I checked has real bodies behind its comments.

## NAMING ALIGNMENT

`stratify.wat`, `fire.wat`, `insert.wat`, `pass.wat`, `accum-pass.wat`, `explain.wat` each contain exactly the spec their name promises — no mismatch found. On the Rust side, `outcome.rs`, `arm.rs`, `census.rs`, `insert.rs`, `node.rs`, `session.rs`, `stratify.rs`, and the `fire/pass/*.rs` files all match name to content.

## CONVERGED

Diverges on two points (Findings 1 and 2 above), converges on everything else. Swept: 6 oracle `.wat` files in full (2,320 lines) and 22 kernel `.rs` files at header + targeted-body depth (13,451 lines), plus `tests/lint/rete_header_claims_are_asserted.rs` in full. Zero `TODO`/`FIXME`/`XXX`/`HACK`/`unimplemented!` (grep, independently reconfirmed). Zero `rune:probare` markers anywhere in target. No new described/hollow forms beyond the two already rowed by `purgare`. The dominant comment density across both halves is exactly the doc-comment-rich / corrected-belief-in-place house style the cast named as exempt, and I applied that exemption rather than flagging it. The one clean, ungated, checkably-wrong structural claim I found — `outcome.rs:22-25`'s caller count and line citation — is new, in-scope, and exactly probare's quarry.
