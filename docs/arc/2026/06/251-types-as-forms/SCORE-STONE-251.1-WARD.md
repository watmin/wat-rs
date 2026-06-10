# SCORE — Stone 251.1 vigilatum ward (COMPLETE — stamp EARNED 2026-06-10T03:11:19Z)

**Status: full inward guard cast, every finding weighed against the disk, driven to
0 L1 / 0 L2 with L3 accepted-with-reason. `src/resolve/` is stamped.**

The vigilatum was earned by COMBAT against the whole applicable book (skip only the
genuinely inapplicable: `secare` — no parallelism; `mora` — no waits). The home is
correct, green→green, clippy-clean, bug-fixed, and now test-covered on the surface
the keystone touched.

## The keystone (the reason this ward mattered)

solvere — round 1 AND re-cast — surfaced the **quote-family boundary braid**: the
special-form argument-boundary table was encoded TWICE (`walk::check_form` carried 6
boundary heads, `normalize` only 4) and had **drifted** — `normalize` silently lacked
the `match`/`cond`/`matches?` boundaries, and its doc falsely claimed it "Mirrors
check_form's boundary logic exactly."

**Decomplected** (`97c206a3`): one `boundary::quote_boundary(head) -> Boundary`
classifier, the single source of the boundary-head set; both passes match it
**exhaustively**, so a new boundary variant is a compile error in every pass until
handled. Drift became unrepresentable. The per-pass traversal stays (walk borrows +
pushes errors; normalize consumes + rebuilds) — only the classification is shared.

## Casts (the full inward guard + circumspicere last)

| ward | verdict | disposition |
|------|---------|-------------|
| intueri | 0 L1 / 5 L2 / 2 L3 | F4/F5/F6/F7 fixed (rename `_list`→`_form`, cross-ref doc, `->` comment, `is_reserved_prefix` doc); F2 (`is_resolvable_call_head` name) accepted |
| purgare (round 1) | clean | dead-fallback bug fixed `4b0dd67a` |
| struere (round 1) | 2 L1 / 3 L2 | F2 (span fold) fixed; F1/F4/F5/F6 re-graded L3/accept (multi-error accumulator by design; located-error contract; pre-existing) |
| solvere | KEYSTONE + 2 | keystone fixed; F1 (unquote-escape braid) → `is_unquote_escape`; F2 (match positional grammar) accepted; F3 (`is_resolvable_call_head` home) accepted |
| sequi | 0 L1 / 0 L2 | clean; the `ambient-context` registry rune holds |
| conformare | 1 L2 / 2 L3 | F1 (span shape) accepted — substrate-wide `Span` convention |
| exigere | 1 L1 / 1 L2 | both were the orchestrator's OWN prose: "tracked, not urgent" reworded; 251.5 NOTE formalized as `rune:exigere(attested-arc)` |
| cernere | clean / 1 L3 | every form literal verified live or correctly-guarded-retired; `define` doc-drift fixed |
| temperare | 1 L1 | `format!`-per-iteration alloc in the `:rust::` coverage check → allocation-free slice check |
| circumspicere (last) | C1–C6 | C1/C3/C4/C6 fixed; C2 reworded; C5 accepted |

## circumspicere — the surround (cast last)

- **C4 (negative space, primary):** the rewritten normalize boundary handling had
  ZERO dedicated tests — the keystone's exact domain. Closed with 6 AST-inspecting
  tests (`normalize_skips_quoted_form_symbols`, `_skips_match_pattern_but_rewrites_body`,
  `_skips_quasiquote_template_but_rewrites_escapes`, `_rewrites_cond_arm_bodies_keeps_else_marker`,
  `_rewrites_matches_subject_keeps_pattern`) — these would have CAUGHT the original drift.
- **C3 (ordering contract):** `resolve_alone_cannot_see_symbol_heads_normalize_must_precede`
  pins that the resolver validates only keyword heads, so normalize must precede it.
- **C1:** freeze.rs step-7 pipeline doc now names `normalize_symbol_refs` + the order.
- **C2:** walk.rs's over-claiming "sole caller is enforcement" (false — `pub`-exported)
  reworded to an honest caller-contract.
- **C6:** the `resolve()` test helper now documents it deliberately skips normalize.
- **C5 (accepted):** the dangling `FOUNDATION.md` reference is pre-existing and
  codebase-wide (7+ sites in freeze.rs alone), not resolve-specific — a separate
  docs-hygiene concern, NOT silently dropped.

## L3 / accepted-with-reason (grounded against the disk)

- **`match` positional grammar duplicated** (solvere F2): the spec is single-sourced in
  the `Boundary::Match` doc; the two traversals implement it under the irreducible
  per-pass ownership split solvere itself flagged. Index-consts don't fit the owned
  sequential rebuild. Accept.
- **`is_resolvable_call_head` in walk.rs, used by normalize** (solvere F3 / intueri F2):
  a documented `pub(super)` shared predicate, correctly homed with the call-head-
  resolution pass (walk.rs's module doc declares it). Documented reuse, not a lie.
- **`UnresolvedReference.span: Span`** (conformare F1): `Span` (with its `unknown()`
  sentinel) is the substrate-wide span representation; all 6 construction sites pass
  real AST spans; spanless-uncompilable is a substrate-wide conformare-arc concern.

## Gates

- `cargo build --release` — clean (0 warnings citing `src/resolve/`).
- lib **950/0/1**; `resolve::tests` **23/23**; `probe_arc251_stone0_symbol_head` **2/2**.
- `cargo clippy --release --lib` — **0** notes citing `src/resolve/`.
- Full-workspace integration reds are NONDETERMINISTIC and PRE-EXISTING (49 at the
  session-start base `26f8805f`, 25 now, set shifts per run) — collateral from the
  live arc-213 fork-IPC deadlock; every suspicious name passes in isolation. **Zero
  regressions from this ward** (verified by base-comparison + isolation).

## Commits

`97c206a3` (keystone) · `a645a27f` (inward guard fixes + DESIGN move-6) · this commit
(circumspicere fixes + 6 boundary tests + the stamp).
