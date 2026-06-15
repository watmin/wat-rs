# Arc 270 (STUB) — naming revisit: the conversion-verb convention (`to-X` vs `->X`)

> **Status: STUB — a banked naming-convention audit, 2026-06-15.** Not blocking anything. Build when
> the list of flagged names is worth a single coherent sweep (rename via the `fix-wat` codemod, like
> the arc-269 spawn-coherence move). A catalog of reach-stumbles on names; each entry is a falsified
> prediction about what the substrate *should* be called.

## Why

A **reach-stumble on a name is the design signal** ([[feedback_reach_stumble_is_the_signal]]): when an
LLM instinctively reaches for a name and finds it ABSENT or PRESENT-BUT-DIFFERENT, that friction names
a convention the substrate hasn't settled. The instinct IS the spec. Collect them; when enough
accrue, settle the convention and sweep all sites at once (don't dr/ip-rename one-off — it churns).

## Flagged names (seed)

| reached-for | actual name | site | the convention question |
|---|---|---|---|
| `:wat::core::i64::->string` | `:wat::core::i64::to-string` | hit live, 2026-06-15 (stone 4a crawl) | Conversion verbs: `to-X` or `->X`? wat's arrow idiom (`->`/`->>` threading, `<-`/`->` annotation arrows) makes `->string` the instinctive *conversion-arrow* form; the substrate currently uses `to-string` (Rust-ish `to_string`). The instinct reached for `->`. |

## The convention question (the real decision)

wat already overloads `->` heavily (thread-first/last macros, type-annotation arrows). Does a
*value-conversion* verb want:
- **`->X`** — "arrow into X" — consistent with wat's arrow-as-transformation idiom (the instinct), OR
- **`to-X`** — Clojure/Rust-ish (`str`, `to_string`); avoids piling more meaning on `->`.

Decide ONE, then sweep every conversion fn (`to-string`, `to-uppercase`/`to-lowercase` [arc-209],
`kebab->pascal`/`pascal->kebab` [arc-209/265, already `->`!], `keyword/from-string`, …). Note the
substrate is ALREADY inconsistent: `pascal->kebab` uses `->` while `i64::to-string` uses `to-`. That
inconsistency is the bug this arc exists to settle.

## The task (when built)

1. Grep the conversion-verb surface across `wat/` + intrinsics: `to-string`, `to-*`, `*->*`,
   `from-string`, `from-*`.
2. Cast `intueri` on the candidates — which reads cleanest, which the substrate's other idioms favor.
3. Run the four-questions on the convention; decide `to-X` vs `->X` (flat).
4. Sweep via the `fix-wat` codemod (`:wat::fix::rename-keyword-prefix` or a sibling), gate grep-clean +
   zero-new (the arc-269 spawn-coherence move is the worked precedent).

Pairs [[feedback_reach_stumble_is_the_signal]] + [[feedback_does_a_macro_need_it_intrinsic_boundary]]
(naming as a taste/validation signal). Sibling of arc 264 (fmt) + the arc-209 naming-conversion tooling.
