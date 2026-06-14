# EXPECTATIONS — Stone 251.5-4.2: `fix-text` comment-faithful codemod

Scorecard fixed BEFORE the strike. The Inquisitor scores against its OWN re-run, reads the diff,
credits nothing the disk doesn't show.

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | the gate: comment byte-identical + `-> :T` stripped | `cargo test --release -p wat --test probe_arc251_fix_text_comment_faithful` | `1 passed` |
| 2 | lib baseline unchanged | `cargo test --release -p wat --lib -- --test-threads=1` | `915 / 36` (zero new) |
| 3 | nursery baseline unchanged | `cargo test --release -p wat --test nursery -- --test-threads=1` | `895 / 4` (zero new) |
| 4 | full surface compiles | `cargo test --release --workspace --no-run` | exit 0 |
| 5 | pure-wat, no Rust | `git -C . diff --stat` | `wat/fix.wat` only (+ the probe is already on disk) |

## Inquisitor's own additional weigh (beyond the probe)

- **Idempotence** — run `fix-text` on its OWN output; the second pass must produce a byte-identical
  result (faithful forms yield zero edits). I'll add a quick in-session check.
- **Comment byte-identity on a richer fixture** — a multi-line fixture with several `;;` lines + a
  blank line between forms; confirm every comment + the blank line survive exactly (the design's
  real-corpus criterion, shrunk to a probe-sized input).
- **Read the diff** — confirm `fix-text` SPLICES original text (uses `string::subs`/`concat`), and is
  NOT a disguised `write-forms` round-trip (which would pass the single-comment probe by luck if the
  comment happened to round-trip, but fail richer inputs). The mechanism, not just the green.

## Runtime prediction

20–35 min. The walk-emitting-edits + line/col→offset + right-to-left splice is intricate (the design
calls it out), but every primitive exists and `fix-source`/`fix-seq` provide the decision logic to
adapt. Most risk is the strip-if deletion-span extents + char-offset math.

## Trap-doors named

- **AST-reprint instead of span-edit** — the single most likely wrong turn. If the build reaches for
  `write-forms`, comments die. The whole point is splicing ORIGINAL text at located spans. The probe's
  byte-identical assertion guards it; the diff-read confirms it.
- **char vs byte** — `:col`/`subs` must both be char-indexed; a byte version corrupts on multi-byte
  glyphs (STOP-3).
- **right-to-left** — applying edits left-to-right invalidates later offsets; must reverse-sort by
  offset (STOP-2).
- **strip-if deletion extent** — the `->`+type deletion must cover the exact leaf spans (and any
  inter-token space that must collapse); under-/over-reach corrupts.
- **idempotence** — a faithful (already-migrated) input must yield zero edits → byte-identical output.

## Out of scope (affirmatively cut)

The macro-param-type rule (the next strike — a NEW fix-form rule riding this same engine), the actual
corpus sweep (running `fix-text` over the ~16 macro files), the ENFORCE validator, and the 251 clojure
cutover (4.3/4.4). This stone is ONLY the comment-faithful engine + its proof. The engine is generic
in spirit (it applies fix-source's rules); making it parameterized over a chosen rule-set is the next
strike's concern when the param-type rule plugs in.
