# What complected looks like — arc 130 sonnet sweep, 2026-05-02

These two files are preserved as the calibration set for the
*complectēns* spell. They are the failed state of the arc 130
slice 1 sonnet sweep: a substrate reshape + monolithic test
rewrite that one-shot a hard problem and could not be diagnosed.

The user's instruction (verbatim, 2026-05-02):

> we need to know what bad looks like to make good - keep it
> here - we'll rebuild from it. we know the pattern... [...]
>
> put this in the arc.... next to the realizations - we must
> not forget what bad looks like... and the test procedure is
> our proof that we attacked it.. we beat complected forms
> with cascading simplicity as complexity..

## What's in this directory

- `substrate.wat` — sonnet's WIP at `crates/wat-lru/wat/lru/CacheService.wat`.
  The arc 130 substrate reshape attempt: `Reply<V>` enum, `Handle<K,V>`
  + `DriverPair<K,V>` typealiases, paired-by-index spawn + driver
  loop. The shape is partially correct; the bugs are in the details.
  - `PutAck` written as a bare symbol (line 64 of the original)
    instead of the keyword `:PutAck` for unit variants — the
    substrate's `MalformedVariant` error fired without span /
    enum-name / hint, leading sonnet to chase parens for ~10
    minutes. (The substrate diagnostic was fixed in commit
    `db4ecc7` adding span + offending text + migration hint to
    `TypeError::MalformedVariant`.)
  - Driver-loop indexing logic broken: even after fixing the enum,
    the runtime test failed `expected "hit", actual "<missing>"`.
    No diagnostic surface; could be in any of the binding chains.

- `test.wat` — sonnet's WIP at `crates/wat-lru/wat-tests/lru/CacheService.wat`.
  Monolithic deftest body. ~30 sequential let* bindings inside one
  `(:wat::test::run-hermetic-ast (:wat::test::program ...))`. When
  the runtime assertion failed, the panic message named no unit
  of work; the failure could have been in any of:
  - The driver loop indexing wrong DriverPair
  - The Reply variant dispatch mismatch
  - The Put not actually persisting
  - The Get's match arm wrong
  - A typo in any of 30 anonymous bindings

  This test cannot answer YES to the four questions:
  - **Obvious?** No — 30 bindings hide structure.
  - **Simple?** No — accidental complexity (anonymous binding
    sequencing) is conflated with inherent complexity (must spawn,
    pop, send, recv, drop, join).
  - **Honest?** No — names like `state`, `pair`, `_finish` lie
    about what's being constructed.
  - **Good UX?** No — failure narrows nothing; localizes nothing.

## What the rebuild produced

The compositional rewrite landed in commit `98fa7c9` at
`crates/wat-lru/wat-tests/lru/CacheService.wat` (the live test).

The rebuild applied the *complectēns* discipline:

1. **One file, top-down dependency graph** — Layer 0 (lifecycle),
   Layer 1a (put), Layer 1b (get), Layer 2 (composition); each layer
   defined as a named helper above the deftest section.
2. **Each helper carries its own deftest** — five deftests total,
   one per layer plus the final scenario.
3. **The final deftest body is short** — six lines, composing the
   topmost named helper. All complexity is named in the helpers above.

The user's framing of the rebuild (verbatim):

> we beat complected forms with cascading simplicity as complexity..

Cascading simplicity AS complexity — the inherent complexity of the
scenario is preserved (it MUST spawn, pop, send, recv, drop, join),
but the accidental complexity (anonymous binding sequencing) is
extracted into the namespace via named helpers. The test body becomes
the smallest expression that COULD have produced the scenario, while
still carrying the inherent complexity in the helpers it composes.

## Why preserved here

Failure engineering says: failures aren't recovered from; they are
read. These artifacts ARE the failure, frozen at the moment the
discipline broke. Future readers — sonnet, future Claude, a human
six months out — read this directory + REALIZATIONS.md + the
compositional rewrite and learn the discipline through its
opposites. The bad surface IS the lesson.

The `complected/` antonym to *complectēns*: from the same Latin
root (*plectere* — to weave) but the past participle *complexus*,
"woven together to the point of entanglement." English "complex"
descends here. Wat-rs grimoire convention: *complectēns* (active,
weaving-into-comprehension) cures *complected* (passive, woven
into entanglement).

## Cross-references

- `../REALIZATIONS.md` — the discipline named.
- `../../../../../crates/wat-lru/wat-tests/lru/CacheService.wat`
  (live, post-rebuild) — the demonstration.
- `holon-lab-trading/.claude/skills/complectens/SKILL.md` — the
  spell that audits for this discipline.
- Commit `db4ecc7` — substrate diagnostic improvement
  (`MalformedVariant` carries span + offending + hint), surfaced
  by sonnet chasing parens on the bare-symbol mistake recorded
  here at the original line 64 of `substrate.wat`.
- Commit `98fa7c9` — REALIZATIONS + compositional rewrite of the
  live test file.
