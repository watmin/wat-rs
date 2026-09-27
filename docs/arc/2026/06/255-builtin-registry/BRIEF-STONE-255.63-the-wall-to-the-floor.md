# BRIEF — STONE 255.63: MEASURE by wall, layer 4 — to the floor

**Drawn 2026-09-27 against `main` @ `99ec39cf9`.** **Executor: grok, via pulsare.** **MEASUREMENT BY WALL**, continuing
255.60–.62. All code changes in a **`git worktree` under the session scratchpad** (symlink `../holon-rs` beside it).
**Nothing lands on `main` but the SCORE** and records under `docs/arc/2026/06/255-builtin-registry/probes-255.63/`
(every diff you apply, the codemod change included). Commit those only; **do not push**. Then `pulsare_yield kind=scored`.

## Why now (builder, 2026-09-27)

*"we are preparing for a serious perf upgrade — we need the clojure cutover to happen."* The loop has cost ~45 min of
re-conversion per layer and stopped each time on the first compile error. This layer goes **to the floor**: it cures
**pure respellings inline**, applies F1's two codemod positions, and does not stop until the floor and the census have
run with both walls up, or a blocker needs a design decision.

## Standing rulings

- **The flip** (251.8d): `::` keywords become faithful-Clojure symbols; the `::` wall (255.60) makes the old spelling
  unspeakable.
- **F1:** every builtin type is `wat.type/…` in every type position; `wat.core/<type>` in a type position is illegal.
  Container only; names unchanged (the builder's 2026-09-20 ruling).

## Steps (in the worktree)

1. **The codemod change for F1, first**, so the conversion emits `wat.type/` where types are. 255.62 measured the two
   positions that produce 12,395 of the 12,407 `wat.core/<type>`: **the head of a `(Head :- [...])` form**, and **the
   elements of its `:- [...]` bracket**. Change the faithful-Clojure conversion (`wat-scripts/fixes/to-faithful-clojure.wat`
   → `:wat::fix::fix-text`, `wat/fix.wat`) so a builtin type in those positions becomes `wat.type/<name>` (the 255.5
   position signal). A **standalone type token** (a doc `@arg`/`@ret` type) is a type position too: the doc converter
   emits `wat.type/` for it. The remaining 12 (syntax-quote, skipped binders) are listed if they still scream.
2. **Convert:** apply `probes-255.60/the-wall.diff` only after the conversion (a walled binary cannot read the old
   corpus). The corpus is converted with the no-wall binary, two-phase, then the doc directives with
   `probes-255.61/convert-doc-fragments.wat` (with step 1's type-position behaviour).
3. **The link cures and the second wall:** `probes-255.62/link-cure-fqdn.diff`, `wall-f1-type-position.diff`, and
   `link-cure-doc-symbol.diff`, **narrowed**: a doc type token may be a keyword, a `(…)`/`[…]` form, or a **namespaced**
   symbol (`wat.type/…`, `u/Person`). A **bare** symbol like `Bytes` stays refused, so
   `bare_symbol_without_colon_is_still_refused_by_the_colon_rule` keeps its meaning. Also stop `fqdn_of`'s enum branch
   (`compose_variant`, `crates/wat-macros/src/edn_doc.rs:281`) from minting `::`.
4. **Cure pure respellings inline, and keep going.** A **pure respelling** is a site whose only fault is spelling: a doc
   `@example` token like `src/reflect/verbs.rs:1911`'s `wat::cache::Lru.Hit` → `wat.cache/Lru.Hit`; a string example
   `"wat::core::i64"` that the runtime wall now refuses → its dotted spelling; a test whose contract is the old spelling
   (the lexer's `keyword_double_colon_path` and its neighbours) → respelled or retired, listed. Each one goes into the
   diff and the SCORE's list. **A site whose fix needs more than a spelling change is not cured: record it and continue
   past it.** If it blocks everything behind it, STOP (below).
5. **Link, then run with both walls up:** the release floor (`scripts/floor.sh`) and a census of every tracked `.wat`.
   Record the converted floor number.

## Class every remaining failure

**(a)** a `::` not converted; **(b)** a name minted at run time with `::` (which of 255.60's listed mint sites fire, and
who calls them); **(c)** `src/` reaching a name through a spelling that now fails; **(d)** a substrate break for another
reason (255.60's `ReservedPrefix` on `--check`; the 255.14 residue classes: `MalformedForm`, `CheckErrors`,
`LociDiedError`, `DeclarationInExpressionPosition`, `ProgramBodyEvalFailed`); **(e)** a meaning change (a namespaced
keyword used as a value, now a symbol); **(f)** a test pinning the old spelling that is **not** a pure respelling;
**(h)** a `wat.core/<type>` in a type position left after step 1; **(g)** other.

Per class: counts, file:line, and **the smallest set of cure sites that clears the most failures**. The converted floor
number, against 78/6014 at 255.14.

## STOP triggers (checked against the work list: steps 1–4 cure only spelling; STOP fires only on a non-spelling blocker)

- **STOP-1:** a single **non-spelling** blocker prevents the link, or prevents the floor from starting. Name it (the
  error verbatim, the site, why it is not a spelling), and STOP.
- A STOP means STOP: report, do not work around it.

## Doctrine

- A measurement. Nothing lands but the SCORE and the probes. There is no known flake; quote failing blocks verbatim.
- Capture `rc=$?` on the **next** statement.
- **If this brief contradicts the code, the code wins. Say so plainly.**
- Remove the worktree when done. Write `SCORE-STONE-255.63-the-wall-to-the-floor.md` beside this brief; commit it and
  `probes-255.63/` with `git add -- <paths>`. **Do not push.**
