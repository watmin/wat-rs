# DESIGN-STONE — a citation in prose is a claim, and nothing checks these two kinds

> **Origin (2026-09-01).** Class **F1**, rows 1 and 2, found by `intueri` and `conferre`. Driven at
> HEAD `de8f3f6a0`. Drawn as **one** strike because both are "a name written in a comment that
> nothing resolves", and one walker answers both.

## Why — two kinds of citation, neither checked

**Kind 1 — a backticked identifier.** `` `head_is_boolean_rete_predicate` `` guards a silent
`_ => None` on the fix-list F path. If it names nothing, the reader cannot find the thing the
comment is vouching for.

**Kind 2 — a bare `*.rs` filename.** `src/rete/kernel/mod.rs:4` says *"Tests are `tests.rs`."*
Driven: **`src/rete/kernel/tests.rs` does not exist** — it became `tests/` in the 2026-08-30
`partire` split, so that sentence was stale the day the split landed.

**And the existing gate cannot see kind 2 by construction.** `no_stale_path_in_doc.rs:47` requires
`tok.contains('/')`. A bare filename has no slash, so it is invisible — the sibling hole filed
during E5 and left open by E3's rustdoc ratchet, which sees `[links]`, not prose tokens.

## ⛔ MY FIRST INSTRUMENT SAID ZERO, AND IT WAS SELF-VOUCHING

A naive scan reported **0 unresolved** out of 732 identifier-shaped backticked tokens — because its
resolution universe was raw text including comments, so **a name appearing only in prose resolved
against itself.** That is exactly the failure the last strike taught (*a resolver whose halves
overlap proves nothing*), committed one strike later by the hand that recorded it.

Restricted to **code positions only**, and widened to `src` + `crates` + `tests` + `benches` +
`examples`: **33 unresolved.** All seven names the work-list row predicted are among them.

**The universe is the whole design.** Too narrow and every test-only name is a false finding; too
wide — or unstripped — and the gate vouches for prose with prose.

## ★★ TWO OF THE 33 ARE CITATIONS MY OWN STRIKES ROTTED

- `check_field_at` — renamed to `check_field_kw` by the **E1+E2** strike (`1efb42fc7`).
- `keyword_constant_segment` — superseded by `classify_keyword_constant` in **D1's residual**
  (`f22704f1f`).

Both shipped with a **green floor and clean clippy**. This is the third time this session a strike
of mine has rotted a citation nothing could see — E4 broke two intra-doc links the same way. The
gate is not hypothetical hygiene; it is the thing that would have caught me, three times.

## ★ THE ONE CONTRACT DECISION

**A name written in a rete doc comment resolves — an identifier to a code position, a bare `*.rs`
filename to a file — or it is declared.** Prose may still discuss a name that does not exist; it
must say so, the way this tree's other citation gates already require.

## ⚠ THE CLASSIFIER IS THE STRIKE, AND ITS NOISE MUST BE DECLARED, NOT GUESSED

Of the 33, several are legitimately unresolvable and each belongs to a **named vocabulary**, not to
a judgement call:

- **clippy lint names** — `needless_borrow`, `type_complexity`, `unused_variable`, backticked inside
  `#[allow]` explanations;
- **memory slugs** — `feedback_a_gate_that_discovers_beats_one_that_lists`;
- **underscore placeholders** — `_pass`, `_raw`.

Declare each vocabulary in the gate, or rune the site. **Do not hand-filter**: an unexplained
exclusion is the shape this whole class exists to remove.

## Blast radius

`tests/lint/` (one gate, extending or sitting beside `no_stale_path_in_doc`), plus the comment sites
it flags. **No `src/` behaviour change** — every edit is prose or a rename in a comment.

## Out of scope — AFFIRMATIVELY CUT

- **Widening beyond `src/rete`.** The arc's surface. The gate may be written to scale, but the
  population it enforces is rete's, and a tree-wide sweep is not this strike.
- **Backticked wat keywords** (`` `:wat::rete::…` ``). The strike that just landed
  (`rete_names_in_wat_scripts_resolve.rs`) covers that family under `wat-scripts/`; the `src/` side
  is its own row.
- **F1's other two lints** — the rune vocabularies and `MINIMUM of`. Their own strikes.
