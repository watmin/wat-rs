# EXPECTATIONS — negative-fixture falsifiability

> ⚠ **This strike buys the CORPUS, not the call sites.** Migrating the 200 `assert!(is_err())` tests
> to a kind-checking helper is a named successor and stays out. A report claiming C18 fully closed
> must say which half.

## ⛔ NO PINNED TEST COUNT

**The floor must be ≥ 5,376 plus every arm you drive.** An equality caps coverage downward while
looking like rigour.

## The scorecard — every pre-value driven at HEAD `545771b2f`

| # | what | state AT HEAD (driven) | required after |
|---|---|---|---|
| 1 | ★ no fixture dies at startup for a missing entry point | **17 of 281 fail with `MainSignatureError`** (stable ×3) | **0**, except rune-carrying fixtures whose subject IS the main signature |
| 2 | ★ the 17 resolved | 6 expired-premise (`arc237_8a/8b/8c/8d`), 2 legitimate (`arc170_slice_1e_*`), 9 others | each given a real `main`, or resolved with driven evidence |
| 3 | ★ the class becomes falsifiable | `probe_arc294_9a_kwargs_ctor_bad` with its construct made VALID **still fails** | mutation 3: the same edit on a RESOLVED fixture turns its test RED — **both halves shown** |
| 4 | the gate is discovered, not listed | — | walks `tests/`+`wat-scripts/`+`docs/`; population **281** or STOP-3 |
| 5 | the exemption is rune-driven | — | mutation 2: removing a rune REDs |
| 6 | the gate REDs on a regression | — | mutation 1: deleting an added `main` REDs, naming the file |
| 7 | no fixture silently passes | **0 `DID NOT FAIL`** | still 0 — STOP-4 |
| 8 | expired premises resolved with evidence | 6 files say *"retired"* / *"formerly"* in their own headers | each disposition backed by a drive against the current binary, **never a guess** |
| 9 | floor / lints / clippy | **`5376 tests run: 5376 passed, 21 skipped`** (425.7 s, 0 FAIL rows), lints **228**, clippy rc=0 | **≥ 5376** + arms, 0 FAIL, lints ≥ 228, rc=0 |
| 10 | `src/` | — | **zero diff, index AND worktree** — or STOP-2 fired and was reported |

## Runtime prediction

**80–110 minutes.** The gate is an afternoon; the 17 fixtures are the work, and the six expired
premises need a driven answer each.

## Trap doors named in advance

- **⛔ DO NOT TRUNCATE THE FIXTURE OUTPUT.** The orchestrator's first classifier sliced
  stdout+stderr to 400 chars and reported **16** where the truth is **17** —
  `typed_if_match_bare_symbol_variant.wat.bad` carries its `MainSignature` at char **4441**. A
  truncated sweep is how absence becomes unfalsifiable.
- **A resolved fixture must still fail for its OWN reason.** Adding a `main` that itself errors just
  moves the wrong reason. After each fix, confirm the error kind is the wall's, not the main's.
- **The two `arc170_slice_1e_*` files are not bugs.** A gate that hard-codes them is a list; the rune
  is what makes it a rule.
- **`git checkout <sha> -- <path>` STAGES**, so `git diff --stat` shows nothing after a real
  mutation. Verify restores by hash.

## What would make this strike a failure even if every test passes

**Giving the 17 fixtures a `main` without proving the class is fixed.** Row 3 is the strike: the same
"make the construct valid" edit must turn a resolved fixture's test RED where it leaves an unresolved
one green. Without both halves, this is 17 cosmetic edits and a gate nobody has shown to bite.

**And batch-deciding the six expired premises.** Each says its wall was retired. That is a claim in a
comment — the same kind of claim that has been the ★ of the last two strikes. Drive each one.
