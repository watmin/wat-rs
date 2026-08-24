# STONE — the heresy stops being TAUGHT

> Arc 109, "annihilate the angle bracket." The syntax is dead in every closed channel. This stone
> closes the one that is still open: **prose**. A reader of wat must never learn the retired form
> from our own comments.

## The census that scopes this (orchestrator's own, 2026-08-23, at `d78e148da`)

**Channel 1 — code — is exhaustively CLOSED.** Not a grep: `target/release/wat --check` imposed on
**all 1826** `.wat*` files (control-validated: fires on a known heretic, silent on a clean file).
15 refuse the angle wall, every one accounted:

```
 4  tests/**/*.wat.bad          negative controls — they exist to prove the wall fires   KEEP
 9  *.wat.disabled/.wat.intueri rotted through a loader gate scoped by EXTENSION          rot
 2  docs/arc/2026/05/130-…/complected-2026-05-02/{substrate,test}.wat                     rot
```

**Channel 6 — prose — is the work.** The wall's own predicate (`<` preceded by `[A-Za-z0-9_']`,
`crates/wat-reader/src/lexer.rs:854-866`) over the corpus:

```
232 comment lines / 100 files in .wat        A 44/13 · B 53/28 · C 70/47 · KEEP 33+28+3
113 PROSE occurrences in docs/ (non-arc)     E — token-safe
 61 IN-FENCE occurrences in docs/            NOT this stone — see "Out of scope", below
 61 wat-marked comment lines / 25 .rs files  D
```

## The ONE TRUE FORM — `:- [K V]`

```clojure
[n :- wat.type/i64]                          arg-spec
:- wat.type/i64                              ret-type
(wat.type/Vector :- [wat.type/i64])          type args   — a REFERENCE, in parens
(wat.type/Vector :- [wat.type/i64] 1 2 3)    constructor — the reference PLUS values
(wat.core/defn ns/f :- [T] [x :- T] :- T x)  declaration — a BINDER, siblings, NO parens
(:ns/f :- [:i64] 7)                          CALL-SITE application — and it BINDS
[A B :-> R]                                  function type
```

`:- []` ≡ absent. The role decides the parens, and the recorded codemod
`wat-scripts/fixes/angle-brackets-to-binder.wat:5-7` is the worked reference:

```
:wat::cache::Lru<K,V>                  (decl-name role)  -> :wat::cache::Lru :- [K V]
:wat::core::Vector<wat::core::i64>     (reference role)  -> (:wat::core::Vector :- [:wat::core::i64])
```

Most prose mentions are REFERENCES → parens. A name being DECLARED → siblings, no parens.

## KEEP — five classes that must NOT change

A rewrite here is worse than doing nothing. Each of these was ground-verified this session:

1. **Rust generics.** `Arc<Function>`, `HashMap<String, TypeDef>`, `PersistentVector<?>`,
   `Option<(Value,Value)>` where the comment says "the Rust shim", `collect::<Vec<_>>()`,
   `:rust::crossbeam_channel::Sender<T>`. wat's `:-` is not Rust's syntax. Leave every one.
2. **Not a type at all.** `n<=0` (`wat/seq.wat:426`), the SQL table template `index_<name>`
   (`wat/query.wat:67`, `wat/query/sqlite-store.wat:19,24`), `<test>__<case>.edn`.
3. **Comments that RECORD the retirement.** "pre-arc-109 this was accepted", "the OLD
   `map<I,O,W>`/`each<I,O,W>` fns" (`wat/bracket.wat:817`), "`uses` … is RETIRED"
   (`wat/spawn.wat:54`). History is not heresy — it is the epitaph, and it is why the next
   reader does not re-mint it.
4. **The migration record.** `wat-scripts/fixes/**` (33 lines / 9 files) and `wat-migrate/**`
   (3 lines). Their whole job is to name what they migrated FROM. Untouchable, same principle as
   an INSCRIPTION.
5. **`docs/arc/**`** (28 lines, 2044 files). Immutable record. What is inscribed is inscribed.

## STOP triggers — REJECTION criteria. Ship nothing on the row; report it.

1. **A QUOTED COMPILER DIAGNOSTIC.** A comment reproducing what the checker or renderer actually
   prints — e.g. `wat/seq.wat:600`, `;; clause 0 skipped (arg 2: expected :wat::core::Seqable<T>,
   got :wat::core::Vector)`. Hand-editing this INVENTS output the renderer may not emit. Do not
   touch it. Report the `file:line` and move on; the orchestrator recaptures it from the live
   renderer.
2. **The surrounding CLAIM is false, not just its spelling.** Worked example, already found:
   `wat/bracket.wat:285` says "the compound angle-bracket keyword strings **built below**" — the
   code below emits `(Peer :- [S R])` and builds no such string. Rewriting the token would leave
   a lie with better syntax. Report it; the orchestrator rules on the sentence.
3. **You cannot tell REFERENCE from BINDER role** from the comment and its surrounding code.
   Report the `file:line`. A guessed role is a wrong rewrite that reads correct.
4. **The comment is inside a fenced wat CODE BLOCK** (slice E only). Out of scope — see below.

## Out of scope — affirmatively cut, not deferred

**The 61 in-fence occurrences in `docs/`** (41 of them `docs/SERVICE-PROGRAMS.md`) are NOT in this
stone. Ground: `SERVICE-PROGRAMS.md:55` reads
`(((handle :wat::kernel::ProgramHandle<i64>)` — that block carries the retired `let` binding shape
**in addition to** the angle head, so swapping the angle head yields a block that still does not
run and now LOOKS repaired. Those belong to the sequenced executable-guides stone
(`NOTE-the-guides-are-not-executable.md`), where a fence is extracted and `--check`ed. Fixing them
here would manufacture the appearance of correctness — the precise failure this arc exists to end.

## The slices — TEXT ONLY

Every rider: comment/prose text only, no `.rs` or `.wat` **code** edits, no cargo, no commit, no
sub-agents. The orchestrator builds, floors, and clippies centrally once the tree is quiescent
(FM 18 + its 2026-08-18 amendment: it is a TIER rule, not a concurrency rule).

- **A — the stdlib.** 44 lines / 13 files. Most load-bearing prose in the language.
- **B — wat-scripts + the edn demo.** 53 lines / 28 files. `fixes/` EXCLUDED.
- **C — tests + wat-tests.** 70 lines / 47 files.
- **D — Rust comments, wat-marked.** 61 lines / 25 files. The discriminator is a wat marker
  (`:`, `::`, `/`) inside the brackets, so KEEP-class 1 is excluded by construction — if a token
  has no wat marker, it is Rust and it stays.
- **E — the guides, PROSE ONLY.** 113 occurrences / ~17 files. Nothing inside a ``` fence.

## Acceptance

- Re-run the wall's predicate per slice; every surviving line traces to a KEEP class or a
  reported STOP, named by `file:line`. **A count is not an acceptance row** — the classification is.
- `scripts/floor.sh` green at 4924, clippy 0, taken by the orchestrator on a quiescent tree
  (sample `git diff --numstat` twice, seconds apart; if it moves, the measurement is VOID).
- The 11 rot files (9 ungated + 2 arc artifacts) are **named here, not fixed here** — they are
  CODE, and the root is a loader gate that asks "does the name end in `.wat`" when it means "is
  this a wat program". That gate is the extirpare root and it is its own stone.
