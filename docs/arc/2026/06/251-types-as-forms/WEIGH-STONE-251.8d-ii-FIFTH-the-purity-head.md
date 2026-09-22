# WEIGH — 251.8d-ii (FIFTH): cure ACCEPTED, conversion STOP. ⭐ **416 → 283, and the purity class is GONE.**

Weighed against `b4d032352` by independent re-run, **building both binaries**. Tree clean, `wat/`
byte-identical to baseline, nothing pushed.

## ⭐⭐ THE CURE, VERIFIED ON TWO BINARIES — EXACTLY ONE CELL MOVES

| probe | pre-cure | post-cure |
|---|---|---|
| `(:wat::rete::pure? '(:wat::core::< 1 2))` | true | true |
| `(:wat::rete::pure? '(wat.core/< 1 2))` | ⛔ **false** | ⭐ **true** |
| `(:wat::rete::pure? '(wat.kernel/println "x"))` — **impure** | false | ⭐ **false — unchanged** |

⭐ **That is the permissive direction bounded correctly**: one verb stops having two answers, and the
default-deny wall does not move. The cure is the **Symbol arm through `canonical_identity`**, the
Keyword arm left semantically byte-identical (255.13's dual-arm rule). **One door; no second spelling
test per table.**

## ⛔⛔ MY BRIEF QUOTED DEAD CODE — the twenty-third correction

I starred `purity.rs:1232`'s `_ => "<structural form>"` as *"a SYMBOL head becomes a literal string."*
⭐ **Verified here: the arm's match guard (`:1223`) is
`matches!(items.first(), Some(WatAST::Keyword(k, _)) if …)` — no symbol head can reach it.** That
fallback fires only on an empty list.

⛔⛔ **And the two stories imply DIFFERENT CURES.** *"The name is lost"* argues for teaching the
structural guard a second spelling — **the false-green direction**. *"The name is mis-keyed"* argues
for a door on the read, 113 lines later. ⭐ **The rider took the second and was right. My brief
pointed at the first.** Twice now I have convicted the wrong address; last time it was a file with no
keyword comparison in it at all.

## Conversion: 416 → 283, and I re-derived the subset myself

| | |
|---|---|
| converted floor | **283 / 6002**, captured, `git checkout -- wat/` (fifth proof) |
| landable floor | ⭐ **6002 / 6002 GREEN** |
| ⭐ purity class in the ARM logs | **234 → 0** — ⭐⭐ **GONE, not reduced** |
| ⭐ **FIXED / NEW** | ⭐⭐ **133 / 0 — a STRICT SUBSET.** No regressions. |

⚠ **That last row cost me three attempts and two broken extractors.** My first pass mis-parsed
`ARM.txt` (which repeats each failure: 1 248 lines for 416 failures). My second reported **138 NEW** —
⛔ **an artifact of my own regex**, which required a digit straight after `(` and so failed on padded
indices `( 167/6002)`, leaving timings in the compared lines. **Third instrument error I have caught
in myself today, and the third time checking before accusing was what saved it.**

## Ledger 232 → 229 — and ⭐⭐ THE RATCHET ITSELF WAS DEFECTIVE

`intrinsic_meta` **3 [Bx3] → 0**; `effectful_by_prefix` **8 [Bx8] → 8 [Ax8]**.
Crate-wide: **B 14 → 3**, shape B in `purity.rs` **12 → 1**. A 93→101, E 125→125.

⛔⛔ **`Bx8 → Ax8` WAS INVISIBLE TO THE RATCHET — it froze COUNTS, not SHAPES.** ⭐ **So would
`Ax8 → Bx8` have been: a site silently acquiring the DANGEROUS class at constant count.** The rider
found this **in the instrument it had shipped one stone earlier**, fixed it in the same commit
(a `⇄ … [fmix] → [mix] (count unchanged)` arm), **made it fail once by construction**, and
⭐ **re-anchored the shape-B calibration onto `arm.rs::compile_acc_fold` rather than deleting it** —
the alternative would have been a calibration that passed because its subject was cured.

⭐ **A ratchet that only counts is a ratchet you can walk past sideways.** Verified present here.

## The adversarial search — three refuted, one found, and it is STRICT

Four attacks on the permissive direction: re-spell impure→pure, re-spell unknown→member, collapse the
axes (`wat.uuid/v4` admitted on Pure, ⭐ **still refused on Deterministic**), and law A still refuses
`wat.core/<` in a `where` in both spellings.

⭐ It did find one: `classify_expr`'s quote/quasiquote **DATA guard is keyword-only** and the new door
sits downstream, so `'(wat.core/quote <impure>)` is walked as a call and **refused** while the keyword
spelling is admitted as data. ⛔ **STRICT, not loose — it cannot forge a green.** Reported, not cured.

## The finding the brief could not have known, and it decided every fixture

⭐⭐ **The wat surface reaches this door through a QUOTE BOUNDARY only.** `resolve::normalize`
rewrites a namespaced symbol call head everywhere else, so **the obvious `sort`-shaped repro returns
rc 0 on the pre-cure binary** and a symbol head inside `(:wat::rete::where …)` is normalized before
the fence sees it. ⛔ **A fixture written the obvious way would have been VACUOUS IN BOTH
DIRECTIONS** — green before and after, proving nothing.

## Also corrected

- **"B should fall by ~12"** → **3 in COUNT, 11 of 14 in SHAPE.** My arithmetic conflated the two.
- **"fixed for free"** → true for `intrinsic_meta`, ⛔ **false for `effectful_by_prefix`**: reached via
  `is_effectful_op`, whose second caller `runtime.rs:13199` hands it a keyword-only head.
  ⭐ **The census my brief ordered is what found it** — the one thing the brief got right here.

## What remains, classified not diagnosed

283 = CheckErrors 123 · TypeMismatch 59 · bare panic 54 · UnresolvedReferences 33 · MalformedForm 6 ·
StartupError 3 · a TEXT-grepping lint 1 · unclassified 4. **Two leads, explicitly not causes:** an
`ArityMismatch` of exactly ONE argument across four sibling `:wat::rete::acc::` verbs — ⭐ **that is
`arm.rs`'s neighbourhood, where 255.13 put the two remaining shape-B sites** — and
`:wat::spawn::thread/init` unresolved, a `Type/method` surface join where `canonical_identity` and
`reconstruct_call_path` disagree on the leaf separator. ⭐ **The recurring class, again.**

## VERDICT

**CURE ACCEPTED, CONVERSION STOP.** Floor 6002/6002 landable, clippy 0 (**not cached — touched
first, 14.21 s**), census `no STOP-8`, delta **3 / RECOVERY 0**, ledger 4/4 at 229, 0 live `.wat`.

⭐ **Trajectory: 3 747 → 416 → 283.** **Twenty-three corrections across twenty-one stones.**

**Next:** `arm.rs`'s 2 shape-B sites (the acc-verb arity lead points there) · the `spawn::thread/init`
leaf-join · then shape **E** — ⛔ **125 sites, 78 in `check.rs`, and the terminal cut needs A+E at
zero, not just heads.**
