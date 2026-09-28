# BRIEF — STONE 255.64: MEASURE by wall, layer 5 — P1, the narrowed F1, `wat.type/AST`, to the floor

**Drawn 2026-09-27 against `main` @ `b46bb911e`.** **Executor: grok, via pulsare.** **MEASUREMENT BY WALL**, continuing
255.60–.63. All code changes in a **`git worktree` under the session scratchpad** (symlink `../holon-rs` beside it).
**Nothing lands on `main` but the SCORE** and records under `docs/arc/2026/06/255-builtin-registry/probes-255.64/`
(every diff applied). Commit those only; **do not push**. Then `pulsare_yield kind=scored`.

## The rulings (builder, 2026-09-27; recorded in `FINDING-the-shape-of-a-declared-signature.md` § "the cutover's type namespace")

- **F1, narrowed:** `wat.type/…` is for types the **language provides** (no wat declaration): the scalars, `Vector`,
  `HashMap`, `HashSet`, `PersistentVector`, `PersistentMap`, `List`, `Tuple`, `Value`, `Instant`, `Duration`, the syntax
  tree. **Declared types keep their declaring namespace:** `wat.core/Option`, `wat.core/Result`, every
  `defenum`/`defrecord`/`defstruct` name. `HolonAST` → `wat.holon/HolonAST`.
- **N-AST:** `:wat::WatAST` → **`wat.type/AST`** (1,802 corpus uses, plus Rust spellings the wall surfaces).
- **P1:** a `(Head :- [...])` form is a **type**, never a call. 255.63's STOP was the macro-body purity walk
  (`src/macros/eval.rs:458` `is_expand_time_legal`, default-deny F5) reading `(wat.type/Vector :- [wat/WatAST])` as an
  expand-time call. Under the old spelling it passed only because `:wat::core::Vector` is also the constructor's name.

## Steps (in the worktree)

1. **P1.** Wherever the macro-body purity walk (and any sibling walk you find that asks "is this head callable")
   meets a `(Head :- [...])` form, it treats it as a type and does not check the head as a call. Report each walk you
   changed.
2. **The codemod** (`probes-255.63/fix-f1.diff` as the base): the three-child type-position rule emits `wat.type/` **only
   for language-provided types**. A declared type in type position keeps its declaring namespace
   (`(wat.core/Option :- [wat.type/i64])`). `:wat::WatAST` → `wat.type/AST` everywhere (type position or not), and
   `:wat::holon::HolonAST` → `wat.holon/HolonAST`. Name the source of the "language-provided" list (the builtin
   registration, not a hand-list in the codemod), and say how the codemod reads it.
3. **Rust spellings the rulings rename:** the Rust side that looks up `:wat::WatAST` (e.g. `src/types.rs:3439`, `:4273`,
   `:9072`) keeps working under the new name. Report how (a canonical key change, or the denotation door), and every
   site touched.
4. **Convert and wall**, as 255.63 prescribed: `rest-param-head.diff`; convert the corpus with the no-wall binary,
   two-phase; the doc directives (`probes-255.61/convert-doc-fragments.wat`, a bare type token being a type position);
   then `probes-255.60/the-wall.diff`, `probes-255.62/link-cure-fqdn.diff`, `wall-f1-type-position.diff` (its refused list
   narrowed to language-provided types), and `link-cure-doc-symbol.diff` narrowed to **namespaced** symbols; stop
   `fqdn_of`'s enum branch (`crates/wat-macros/src/edn_doc.rs:281`) minting `::`.
5. **Cure pure respellings inline, and keep going** (255.63's definition: the only fault is spelling; each one listed).
6. **Link, then run with both walls up:** the release floor (`scripts/floor.sh`) and a census of every tracked `.wat`.
   Record the converted floor number.

## Class every remaining failure

**(a)** `::` not converted; **(b)** a name minted at run time with `::` (which of 255.60's listed sites fire; who calls
them); **(c)** `src/` reaching a name through a spelling that now fails; **(d)** a substrate break for another reason
(255.60's `ReservedPrefix` on `--check`; the 255.14 residue classes); **(e)** a meaning change (a namespaced keyword used
as a value, now a symbol); **(f)** a test pinning the old spelling that is not a pure respelling; **(h)** a
language-provided type still spelled `wat.core/`; **(g)** other. Per class: counts, file:line, the smallest cure set.
The converted floor number, against 78/6014 at 255.14.

## STOP triggers (checked against the work list: steps 1–5 are the rulings and pure respellings; STOP fires only on a non-spelling blocker the rulings do not cover)

- **STOP-1:** a single non-spelling blocker, not covered by P1, F1, N-AST or the declared-type rule, prevents the link or
  the floor from starting. Name it (the error verbatim, the site, why it is not a spelling), and STOP.
- A STOP means STOP.

## Doctrine

- A measurement. Nothing lands but the SCORE and the probes. There is no known flake; quote failing blocks verbatim.
- Capture `rc=$?` on the **next** statement.
- **If this brief contradicts the code, the code wins. Say so plainly.**
- Remove the worktree when done. Write `SCORE-STONE-255.64-the-wall-layer-five.md` beside this brief; commit it and
  `probes-255.64/` with `git add -- <paths>`. **Do not push.**
