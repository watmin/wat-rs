# The retired prime survives in prose, and the lint that polices it cannot see there

**Measured 2026-09-19, HEAD `de06f5ccd`.** 16 sites fixed (comments only); the class left open,
deliberately, with its size measured.

## What started it

Four exchanges were spent establishing that `:wat::spawn::spawn-thread` (a bypass wrapper, since
deleted) was **not** a duplicate of `:wat::kernel::spawn-thread`. The builder's question —
*"is there a second flavor of spawn-program?"* — was the right one to ask, and the answer is **no**:
one `defclause` at `wat/spawn.wat:362`, two clauses (ThreadOpts → `spawn-thread`, ProcessOpts →
`spawn-process`), each tier primitive registered exactly once (`#[wat_intrinsic]` across **446**
intrinsics has **no** duplicate name; the only repeated token is the literal `<fqdn>` placeholder in
macro documentation).

⛔ **But the prose said yes.** `spawn-program'` — a name arc 278 retired — appeared **16 times** in
`.wat` comments, including the section header a reader lands on first:

```
;; ── The Keymaker's masterwork (the spawn-program' defclause) ──
```

Someone reading that goes looking for a primed variant and finds none. That is very close to the
confusion this excursus just spent four exchanges unwinding, sitting in the file's own prose.

Fixed: 16 sites across `wat/spawn.wat` (8), `wat/test.wat` (6), `wat/bracket.wat` (2). Comments only
— verified: **every changed line is a `;;` line**, so the form tree is untouched and no codemod was
required (`fix.wat` walks forms and does not rewrite comments by design).

## ⛔ The class is NOT closed, and here is why — measured, not guessed

`tests/lint/retired_name_justified.rs` exists for exactly this defect. Its own reasoning applies
verbatim to a comment: *"a user can only ever type the UNPRIMED form, so a message naming the primed
form points at a verb that does not exist."* It cannot see these sites:

- it collects `.rs` only (`p.extension() == Some("rs")`), and
- its predicate is **shape-based, not list-based** — any `word'` **inside a Rust string literal**.

⚠ **Widening it to `.wat` comments is not a small change.** Census of primed identifiers in `.wat`
comments:

> **1518 occurrences, 440 distinct names.**

And the top of that distribution splits two ways that a shape predicate cannot separate in prose:

| real retired primes | English possessives |
|---|---|
| `recv'` 100 · `send'` 44 · `connect'` 30 · `select'` 22 | `wat'` 56 · `surface'` 38 · `service'` 33 · `op'` 21 · `child'` 21 · `owner'` 18 · `macro'` 18 · `caller'` 18 |

The Rust lint survives this because string literals rarely carry possessives — it even has
`tests::ignores_english_possessive_and_contraction`. In free prose the ratio **inverts**: the noise
dominates the signal, and *"the surface's"* written as `surface'` is not mechanically
distinguishable from a primed name.

⭐ **So the honest disposition is: fix the unambiguous sites, measure the class, and do not ship a
detector that would need 440 exemptions on day one.** A list-based predicate over the 24 names arc
278 actually retired would be tractable; the shape-based one is not. That is a stone, not a
side-effect of this one.

## Also open, same cause, different language

**22** `spawn-program'` sites in `.rs` **comments** (`src/process/mod.rs:29`, `:72`,
`src/channel/mod.rs:56`, …). The lint walks those files but scans only string literals, so a Rust
*comment* is as invisible to it as a wat comment. Not fixed here — naming it so the count is on the
record rather than discovered again in four exchanges' time.

## Verification

`(:wat::deporder::verify-stdlib)` → `[]` · floor `Summary [ 125.008s] 5317 tests run: 5317 passed,
22 skipped` (`.floor/2026-09-19T05-59-12Z/`, no `ARM.txt`) · zero residual `spawn-program'` in
`wat/`.
