# CENSUS — the illegal-EDN form classes, and the true scale of the flip to `wat.is/a-clojure`

Measured 2026-08-15 at HEAD `69239937` over `wat/` + `wat-scripts/` + `tests/`, `--include=*.wat`.
**Each pattern was positive-controlled before its number was written down** — one of the three was
refuted and carries no number as a result.

> Context: the builder named the destination this session — *"once [the registry] is built... we
> destroy all 'colon-quoted symbols' at once... we are going to lay the foundations necessary to
> annihilate the ':rust::style::scheme' and move into `wat.is/a-clojure`."* This is the scale of
> that flip, as far as grep can honestly report it.

## CLASS 1 — the colon-quoted symbol (`:wat::core::+`) — **the main body**

| | |
|---|---|
| total occurrences | **79,253** |
| **distinct spellings** | **6,552** |
| `.wat` files carrying ≥1 | **1,263** |

✅ **Pattern validated.** Top-12 matches are all genuine colon-quoted symbols:
`:wat::core::i64` (4,280) · `:wat::core::None` (4,172) · `:wat::core::defn` (4,072) ·
`:wat::core::let` (2,172) · `:wat::core::String` (2,150) · `:wat::kernel::assertion-failed!` (1,913) ·
`:wat::holon::to-holon` (1,896) · `:wat::core::match` (1,541) · `:wat::core::if` (1,185) ·
`:wat::core::nil` (946) · `:wat::core::bool` (908) · `:wat::core::=` (903).

### ★ THE MIGRATION UNIT IS 6,552, NOT 79,253 — AND IT IS SHARPLY SKEWED

**Those top 12 spellings alone account for 26,138 occurrences — 33% of the corpus.** The distribution
is a steep head with a long tail, which is the single most useful fact here:

- A codemod that handles the **head** converts the majority of the corpus in one pass.
- The **tail** (thousands of once- or twice-used spellings) is where a per-name rhythm is affordable
  and where the registry's ability to hold two spellings at once actually earns its keep.

So *"one keyword-as-symbol at a time"* and *"all at once"* are **not** the two options. The real
shape is **head-by-codemod, tail-by-name**, and the registry is what makes the two compose instead
of conflicting.

## CLASS 2 — the angle-bracket parametric (`HashMap<K,V>`) — **ruled illegal post-migration (251.8a-bis)**

| | |
|---|---|
| occurrences of `Name<…>` | **2,945** |
| of which **comma-bearing** | **951** |

✅ **Pattern validated**, including a negative control: `(< a b)` and the `<-` binder arrow do **not**
match. Top matches are all genuine annotations — `wat::core::Vector<wat::core::String>` (460),
`Vector<wat::WatAST>` (251), `Vector<(i64,i64,String)>` (153), `PersistentVector<i64>` (103),
`Stream<T>` (72), `ThreadSelfPeer<i64,i64>` (42).

⚠ **951 here vs "965" on the 251 seam** — two patterns, same question, 14 apart. **Neither is
authoritative and this census does not pick a winner.** Recording the delta rather than laundering
one number into the record; the real enumeration comes from the checker, not grep (R65 `SCVTVM IDEM
INDEX`).

**Why the comma-bearing subset is the dangerous one:** after the symbol flip, `(f HashMap<K,V>)`
reads as valid EDN and *silently changes arity* 2→3 — no error, wrong program. That is the class
that decides 8b's scope.

## CLASS 3 — the double-slash symbol (`wat.core/Option/expect`) — ⛔ **NO VALIDATED COUNT**

**The corpus-wide pattern I wrote was REFUTED and its number is not recorded.** It returned 1,177 —
against the tracked figure of **59** (task #98) — because it matched **file paths inside strings and
comments**, not symbols: `wat-scripts/perf/grid` (69), `target/release/wat` (59), `docs/arc/2026`
(23), `usr/local/bin` (4).

**The tracked 59 stands. This census adds nothing to class 3** beyond the knowledge that a naive
slash-counting pattern cannot measure it — any real count must first exclude string and comment
context, which grep alone cannot do here.

*(This is `[[feedback_validate_a_search_pattern_before_trusting_its_count]]` firing in time: two
greps, one question, 59 vs 1,177. The positive control is what caught it, before the number reached
a document anyone would build from.)*

## What this census is NOT

It is **grep, over `.wat` only.** It does not count the Rust side — the 561 dispatch arms, the
`is_reserved_prefix` classifiers, the goldens, the test fixtures, or the error-message strings that
embed these spellings, all of which must also flip. It is not a census in the R65 sense: **when the
registry lands, the checker enumerates the real worklist.** These numbers exist to size the campaign
and to make "head vs tail" a decision made against data, not a feel.
