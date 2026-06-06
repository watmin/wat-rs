# wat/core.wat — updated-guard re-earn (249.N ward-close, spec/DSL set)

> The file's 2026-06-04T04:01:56Z stamp (4-spell) drifted: anchor `6b6def52`,
> diff since = +53 lines (the 249.3b threading macros + 249.4a keyword/of).
> Re-earned against the spec/DSL guard: cernere + probare + conferre + exigere
> + circumspicere LAST. Spell texts fetched fresh from the signed channel,
> embedded verbatim.

## R1 — the 4 inward casts (on `330e4a6c`)

| spell | verdict |
|---|---|
| cernere | **CONVERGED 0+0+0** — full vocabulary table: every form traces to a live registration (declaration forms → check.rs; per-Type leaves + HOFs + keyword/string ops → runtime.rs dispatch; every expand-time head in the ->/->>/keyword-of bodies confirmed on the is_pure_total allow-list, incl. the 5-arg if and 4-arg Option/expect shapes) |
| probare | 1 L2 + 1 L3 — 166:68 form:comment (2.44:1, mixed-leaning-rich; commentary is architecture signal); ALL 16 top-level forms Expressed; L2 = dissoc/keys/values short-name aliases alive but ZERO corpus callers (hollow-by-caller-absence); L3 = alias block lacks the section banner |
| conferre | **4 L1 + 2 L2 — ALL spec-side (docs/USER-GUIDE.md stale); core.wat's own 17 header claims ALL verified clean** against bodies + witnesses. DIV-1 "mixed-numeric promotes to f64" (retired 237.8b → NoMatchingClause); DIV-2 typed leaves shown variadic (strictly 2-ary, ArityMismatch); DIV-3 Tier-3 comma syntax (lexer-rejected arc 171; '2 suffix dropped 237.8b; mixed leaves deleted); DIV-4 `:wat::runtime::define-alias` named as the mechanism (hard-cut 241.12 → `:wat::core::defalias`); DIV-5 ->/->> spec-silent; DIV-6 cheatsheet row conflates =/ordering cross-numeric behavior |
| exigere | **CONVERGED 0+0+0** — every comment present-tense; the one arc cite (249.4a) verified on disk |

## Orchestrator weighing + the fights

- **conferre's 6 → FOUGHT BY THE ORCHESTRATOR in docs/USER-GUIDE.md** (docs
  lane): the three-tier arithmetic block rewritten to present truth (same-type
  only; 2-ary leaves; Tier-3 retirement note); DIV-4 fixed as a CLASS at three
  sites (~1391, ~2956 [which presented the retired macro as live in a
  wat/runtime.wat that no longer exists — live aliases are defalias calls in
  wat/list.wat], ~3004); the cheatsheet row split (=/not= cross-numeric OK;
  ordering same-type only); a Threading-macros section added (with the
  FQDN-vs-return-arrow note).
- **probare L2 → FOUGHT BY WITNESS, not deletion, not rune**: the aliases are
  stdlib surface; tests-are-the-demos — corpus witnesses added
  (wat-tests/core/core-collection-aliases.wat) so the short names are
  exercised AND taught. list-fold-aliases.wat is the precedent.
- **THE NAMED THREADING DEFTEST** (owed by the close protocol):
  wat-tests/core/core-threading.wat — ->/->> corpus demos mirroring
  core-arithmetic.wat's shape.
- probare L3 → the section banner added.

## R1 fight LANDED (`96861342`): 6 spec fixes (orchestrator) + 17 corpus
deftests (corpus 217→234) + banner. Gates re-run first-hand: lib 920/0/1,
corpus 234/0/53.

## circumspicere — the perimeter (cast LAST, on `96861342`)

Verdict: **1 L1 + 3 L2 + 2 L3** + 11 claims verified CLEAN (0-ary identities +
0-ary -// rejections + cross-type arithmetic AND ordering rejections all
corpus-witnessed; register_stdlib bypass pub(super)-bounded with one caller;
missing core.wat impossible — include_str! bakes it into the binary, corruption
= loud StartupError; defn expansion claim; keyword/of promotion cite; the
template-position witness live).

| # | surround | finding | weighing |
|---|---|---|---|
| F1 | claim+egress | **L1**: "Loads early in the stdlib" (core.wat:6-7) + "Loads BEFORE collection forms" (stdlib.rs:212-214) are FALSE — core.wat is position ~24/25; the two-pass registration (freeze.rs register_stdlib_defmacros pre-expansion pass + defclause stub pre-registration) makes position irrelevant. Orchestrator re-grounded both sites. | FIGHT: both comments rewritten to the position-independence truth |
| F2 | invariant | L2: the OLD stamp's parenthetical names only core-arithmetic as the witness corpus | DIES IN THE RE-STAMP (full guard description replaces it) |
| F3 | negative space | L2: zero-steps identity `(-> x)` / `(->> x)` unwitnessed | FIGHT: 2 corpus deftests |
| F4 | negative space | L2: `(-> x ())` fires Option/expect panic ("-> step has no head") — unstructured, undocumented; `(->> x ())` splices to `(x)` — a different, asymmetric failure | FIGHT: document BOTH behaviors truthfully + witness the -> rejection at its real failure shape (verified by experiment first); behavior redesign NOT in ward-close scope |
| F5 | negative space | L3: `(keyword/of :Head)` zero-args → `:Head<>` unspecified | FIGHT: doc line (behavior verified first) |
| F6 | negative space | L3: defn metadata-map restriction has a Rust witness but no corpus demo | **DECLINED → task #181** (the test-surface ward owns corpus pedagogy completeness; named tracked task, not a vague deferral) |

**Perimeter closure sweep dispatched** (4 items). On green: gates → commit →
re-stamp wat/core.wat (spec/DSL guard, full witness-corpus list, ISO8601-UTC-
seconds computed at convergence) → **the 249.N board all-green** → INSCRIPTION
(FM-11 pre-grep) → arc 249 CLOSED.
