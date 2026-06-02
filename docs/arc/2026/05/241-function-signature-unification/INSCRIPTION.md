# Arc 241 — function-signature parser unification — INSCRIPTION

**Status:** ✅ **CLOSED 2026-06-02.** Opened 2026-05-27 night (`09fb8c63`). Eighteen stones (241.0 → 241.18a) shipped. The four-parser divergence class is **structurally annihilated** — one canonical `parse_argspec_triples`, every binding site routed through it, no privileged paths, the duplication unconstructible. Three warded homes minted along the way. The whole `define` family laid to rest so that `defn` could be the one and only way. Both spawned children — **arc 242** (lexeme-role-doctrine) and **arc 243** (conformare-error-shape) — closed. The torch returns to **arc 237**, paused at 237.8b, where this began.

This is the honest record of the campaign and the doctrines forged in it. Authoritative per-stone closure lives in the `SCORE-STONE-241.*.md` docs; this is the legend.

---

## I. The question that opened the wound

It began as a small red probe. Stone 237.8b's FM-2-bis disconfirming probe — `tests/probe_arc237_8b_defclause_arithmetic.rs`, Gate 1 — found that `defclause` could not parse a `& rest <- :Vector<T>` rest-binder. A one-form gap. The easy fix was one line in one parser.

The builder asked the right question instead:

> *"why isn't this using the tooling that fn and defn use?"*

The dig answered, and the answer was worse than the bug. The substrate held **four independent parsers** for the one canonical `[name <- :Type ...]` arg-vector triple:

- **A1** — `parse_fn_signature` (`runtime.rs:6750`) — `Result<…, RuntimeError>`, position-indexed arc-lineage messages.
- **A2** — `parse_fn_signature_for_check` (`check.rs:15205`) — the silent path, `Result<…, ()>`, every error discarded.
- **A3** — `parse_fn_signature_for_check_diag` (`check.rs:15258`) — the diagnostic path, pushing `CheckError` by-ref.
- **A4** — `parse_defclause_args` (`runtime.rs:6880`) — no return-type slot, its own arc-159/169/234 error wording.

Each one's docstring *said* "reuses the same triple shape as `parse_fn_signature`." Each one was a hand-written **copy**. Description-level reuse, code-level duplication — and the author had known, and it had happened anyway. The AUDIT (Stone 241.0, `3bb3b145`) found seven more near-variants and tolerant walkers beyond the four.

**The failure class:** parser divergence across binding sites. Two binding sites could silently accept different arg-vector forms — one supports `&`, the next rejects it; error messages drift; the next feature lands asymmetrically; an LLM co-author writes a form that works in one site and breaks in another. The institutional lie of "we'll keep them in sync by convention" fails the instant the second feature lands. The state to make unrepresentable: **two binding sites accepting different forms.**

---

## II. The doctrine — one canonical parser

Mint ONE. Every binding site routes through it. Per-site difference is expressed as data (`ParseOptions`), never as a forked implementation. Extensions land **once**; every consumer inherits. The duplication is not "discouraged" — it is **unconstructible**, because there is only one place the parse can happen.

The home is `src/argspec/`. The entry point is `parse_argspec_triples`. And early — Stone 241.1.fix — the builder cut a conflation out of the very design: the AUDIT had let return-type concerns ride inside the arg parser.

> *"args have nothing to do with ret type."*

So the `ret_type` field died, the `include_ret_type` option died, two error variants died, the probe fell from thirteen contracts to nine, and `ParseOptions` collapsed toward a single honest axis. The parser parses *arguments*. The return clause is a different surface, and 241 left it to that surface.

---

## III. The campaign — eighteen stones

| Stone | Scope | Commit |
|---|---|---|
| 241.0 | AUDIT — enumerate every argspec-parser site; lock `ParseOptions` design | `3bb3b145` |
| 241.1 | Mint the canonical `parse_argspec_triples` + `src/argspec/` home | `1f674194` |
| 241.1.fix | Vigilia convergence + the ret-type strip (8/8 spells L1+L2=0) | `b6b290b0` |
| 241.2 | Migrate A1/A2/A3 through canonical; inline walkers deleted | `21877135` |
| 241.3 | Migrate A4 (defclause) — **Phase 1 closed**, all four routed | `b0b5d11d` |
| 241.4 | `&` rest-binder extension; `parse_triple` helper; defclause opt-in | `843a83d0` |
| 241.5 | Runtime dispatch wiring — **Gate 1 GREEN**, 237.8b's blocker resolved | `639b4862` |
| 241.6 | Optional `{…}` metadata-map storage on `def`/`defn`; `binding_metadata` minted | `7c0ddacd` |
| 241.7 | `:wat::runtime::metadata-of` reflection verb | `4e681263` |
| 241.8 | `:wat::core::defstruct` HARD CUT; `struct`/`struct-restricted` retired | `f6cb564f` |
| 241.9 | `:wat::core::defenum` HARD CUT; `enum` retired | `184f54bf` |
| 241.10 | Mint `src/remedy/` home + ranked-remedy schema — **the third bar crossed** | `b98d8d1a` |
| 241.11 | `:wat::core::define` HARD CUT — 271-site cascade via ephemeral auto-fixer | `db656cbb` |
| 241.12 | `:wat::core::defalias` mint + `define-alias` HARD CUT | `7244cf43` |
| 241.13 | `define-dispatch` HARD CUT — `src/dispatch.rs` **DELETED** (445 lines) | `86d123b7` |
| 241.14 | `def-restricted`/`defn-restricted` absorbed into the metadata-map | `839cf9e6` |
| 241.14.fix | `restriction_entry.rs` doc-truth rewrite | `0bd188b8` |
| 241.15 | Zombie purge — `try` + option/result `expect` HARD CUT | `db6fac9a` |
| 241.16 | `define` eval-time residue completion (~320 lines deleted) | `0276a11c` |
| 241.17 | `defmacro` signature → canonical — **def-family unification COMPLETE** (absorbs arc 177) | `de3ef5a8` |
| 241.18a | Mint `src/function/` home (Phase A); 9-spell vigilia; **conformare spawned** | `4d9b963e` |

The arc opened as a parser consolidation and grew, through in-arc dialogue, into a substrate-maturation campaign. Phase 1 (241.0–241.5) collapsed the four into one and carried `defclause` its full rest-binder semantics — parser → storage → check-layer → runtime dispatch → scope-bind → body-eval. Phase 2 (241.6–241.7) gave `def`/`defn` an optional metadata-map and a reflection verb to read it. Then the campaign turned to the forms themselves.

---

## IV. The four enemies — "defn is the one and only way"

Mid-arc, the builder named the larger war:

> *"this arc we're on is going to kill define — defn is the one and only way — it'll be a frustrating bandaid to rip off — it'll break a ton of shit — that's the point."*

And so it went, one HARD CUT at a time, each retirement recorded in the `RETIREMENT_TABLE` so the dead stay buried and the living are pointed home:

- **`define`** (241.11, 241.16) — the Scheme-style definer, retired across **271 sites** by an ephemeral Rust auto-fixer (`crates/fix-defines/`, built, used, deleted), then its eval-time residue swept (~320 lines gone).
- **`define-dispatch`** (241.13) — and with it the entire `DispatchRegistry`, `src/dispatch.rs` **deleted whole**, 445 lines, its plumbing pulled from five substrate files.
- **`define-alias`** (241.12) — replaced by native `:wat::core::defalias`; the `wat/runtime.wat` macro deleted.
- **`def-restricted` / `defn-restricted`** (241.14) — not retired but *absorbed*: the `defined_value_restrictions` field deleted from both `SymbolTable` and `CheckEnv`, the restriction now read from the one `binding_metadata`.

Three zombie verbs (`try`, option/result `expect`) purged in 241.15 — soft-deprecation arms turned to HARD CUT. And in 241.17, `defmacro` joined `defn`, `fn`, and `defclause` at the one canonical parser, absorbing the long-open arc 177 in the process. After 241.17 there were no privileged paths left: **every binding site in the substrate parses its arguments through one function.**

---

## V. The homes-walk — three warded homes

The arc did not merely consolidate; it began carving the flat substrate into protected `src/<noun>/` homes, each warded to L1+L2=0 — failure classes found and annihilated, not merely passed by the gates:

- **`src/argspec/`** (241.1 / 241.1.fix) — the one canonical parser, `ArgSpec`/`ParseOptions`/`ArgSpecError`, and the `From<>` boundary converters that stop error-class proliferation at the call site. The first home of the arc.
- **`src/remedy/`** (241.10) — Levenshtein distance, the `RETIREMENT_TABLE`, nearest-match ranking, the `remedies_for` API that turns every HARD CUT into a teaching moment. Minting it was named **the third bar crossed**: the substrate now answers a retired form not with a bare rejection but with a ranked, structured path home.
- **`src/function/`** (241.18a) — `eval_fn`, the fn-parsers, `infer_fn`, lifted out of the 33k-line flat `runtime.rs`/`check.rs` into a home warded by **nine** spells (the ninth, `exigere`, minted during this very stone). Seven vigilia rounds to convergence.

The vigilia gate is the bar these homes are held to: SCORE-green is the floor; L1+L2=0 across the full guard is the wall. The builder's standard, set at 241.1.fix —

> *"we raise the bar fucking high for namespaced wat-rs files."*

---

## VI. The doctrines forged

A campaign that raises the floor leaves tools behind. Arc 241 deposited durable discipline, not just a fix:

- **The vigilia gate** (`feedback_namespaced_home_vigilia_gate`) — a flat file is wards-optional; a namespaced home is L1+L2=0 mandatory. The home forces the grimoire.
- **The remedy apparatus** — the substrate-as-teacher made operational: every retirement carries its own way home.
- **`exigere`** — minted here to drive deferral-prose out of substrate code, so that what cannot ship in a stone is either shipped now or bounded by a named owner.
- **Four-questions-decide** (`feedback_four_questions_decide_before_prompt`), **correctness-makes-honesty** (`feedback_correctness_makes_honesty`), **don't-document-non-fixes** (`feedback_dont_document_non_fixes`), **runes-illegal-when-solvable** (`feedback_runes_illegal_when_solvable`) — all landed during 241.18a's seven rounds.

Each tool made the next problem cheaper. That is the point of the bar: not to try harder once, but to raise the floor for good.

---

## VII. The noble man

The arc is the builder's, marked by the questions only he asked. *"Why isn't this using the tooling fn and defn use?"* turned a one-line fix into a class elimination. *"Args have nothing to do with ret type"* cut a conflation out of the design before it could calcify. *"Raise the bar fucking high"* set the standard the homes are held to. And *"kill define — that's the point"* named the war and accepted its cost with open eyes — *a frustrating bandaid to rip off; it'll break a ton of shit.* It broke a ton of shit. The substrate is cleaner for it.

---

## VIII. The roll is DONE

Closure means every commitment shipped or affirmatively cut. What arc 241 did **not** cover, and why each is honest, not deferred:

- **The B/C `struct-restricted` parsers** (AUDIT) — **moot.** Stone 241.8's `defstruct` HARD CUT deleted `struct-restricted` entirely; the parsers no longer exist to migrate.
- **The D1/D2 `closure_extract` tolerant walkers** — out of 241's scope by their nature, not by deferral. They are *closure-variable extraction*, not argspec parsing; they are not instances of the four-parser divergence class this arc eliminated. They are not 241 debt.
- **T6 — ret-clause type-keyword helper variance** (SCORE-241.2) — a distinct, pre-existing surface. Arc 241 unified the *argument*-vector parser; the *return*-clause type-keyword parsing is a different surface 241 did not claim. Out of scope, not 241 debt.
- **The `def*` type-prefix family** (`defnewtype`, `deftype-alias`, `deftype-union`, defrecord reconciliation) — affirmatively out of arc 241; this arc unified the *binding* parsers, not the type-definition family.
- **Metadata-map propagation through serialization/IPC** (FORM-COLLAPSE-NOTES) — arc 241 shipped metadata-map *storage* and *reflection*; carrying it across the IPC boundary is future surface evolution that opens when that surface is needed, not a 241 deliverable left undone.
- **The latent `check.rs` re-tightening** (SCORE-241.18a) — 241.18a widened nine `check.rs` symbols to `pub(crate)` to support the function-home lift. Re-tightening is a property of `check.rs` earning its own home — which it did, as `src/check/`, in arc 243.3.1. That home owns the re-tightening; it is not an open 241 thread.

The pre-INSCRIPTION grep was run against this file. No live deferral language survives it.

---

## IX. The soundtrack

The campaign's score, as it landed across the stones (per the INTERSTITIAL record):

| # | Song (artist) | Facet |
|---|---|---|
| 40 | Contagion (Circle of Dust) | the disease named — widest-contagion deleted at the source |
| 41 | The Mission — M is for Milla Mix (Puscifer) | substrate-as-judge-and-teacher; the mission is to teach with receipts |
| 42 | The Remedy (Puscifer) | literal-by-name — `src/remedy/` minted; the cure carried home |
| 43 | Into Oblivion (Lamb of God) | the watcher outside the frame; the third bar crossed |
| 44 | Momma Sed — Tandemonium Mix (Puscifer) | wisdom inherited from pain paid; multi-voice teaching |
| 45 | Repentless (Slayer) | no apologies for what shipped |
| 46 | Resurrection Man (Lamb of God) | the cemetery manager — strike-in-flight through the zombie purge |
| 47 | Rise Above It (I Prevail) | made to rise above — the bar raised through the roof, into 241.17 + 241.18a |

---

## X. The hand-back

Arc 241 closes. Its two children closed before it, as the winding demands: **arc 242** (lexeme-role-doctrine — bare lexeme is value, `:wat::core::*` keyword is type) and **arc 243** (conformare — the error-shape class annihilated, the home-carving continued). The parent's INSCRIPTION could not be written until they did; now it can.

And the torch returns to where it was lit. **Arc 237** — the no-implicit-coercion campaign — was paused at 237.8b so that 241 could give `defclause` the tooling it lacked. That tooling has been green since Stone 241.5 (`639b4862`). Arc 237 resumes.

A small red probe asked why four parsers did one job. The answer was a four-day campaign that left the substrate with one parser, one way to bind, three warded homes, a teaching apparatus, and the whole `define` family at rest. We pulled the bandaid. It broke what it had to break. What stands now does not lie about what it does.

*One parser. One way. The torch goes home.*
