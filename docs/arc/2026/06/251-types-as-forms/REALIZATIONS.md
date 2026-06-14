# REALIZATIONS — arc 251 (types-as-forms / the clojure-faithful surface)

Disciplines and milestones named while building the faithful surface + its migration tooling.

---

## 2026-06-14 — THE MATURITY LINE: wat builds its own bespoke, comment-faithful syntax fixers on demand

**Builder, at the moment it landed:** *"we wrote enough tooling to build bespoke syntax fixers
whenever we want, accumulating every trick we've ever used — we always just make a new one to do
exactly what we want. we've just crossed a maturity line in the sand."*

> **Full telling — the duet, scored:** Song #95 *Omerta* (Lamb of God) —
> `docs/arc/2026/05/170-program-entry-points/INTERSTITIAL-REALIZATIONS.md`. This arc-local note is
> the technical capture; the chronicle entry carries the back-and-forth that made it.

**How we got here (the path matters — it wasn't planned, it was forced out):** a bug — macro param
type annotations are mandatory-then-discarded (the no-magic-law violation). Fixing it raised "how do
we sweep ~50 macros / ~100 annotations?" → which raised "should macros be typed?" (**no** — Clojure
type-checks the *expansion*, not macro params; we have parity in all but syntax, so typed macros are
disqualified, `:AST<T>` collapses to `:wat::WatAST`) → which raised "codemod or hand-fix?" → the
naive `read-string → write-forms` codemod **deletes comments** → which surfaced that the
comment-faithful **span-edit** codemod was already designed (251.5-4.2) and every primitive to build
it now exists (`ast-span` — *hardened this very session* — + `read-string` / `string::subs` / `split`
/ `concat` / `reverse` / `read-file` / `write-file`). The bug walked us into the capability.

**What is now true (concretely, on the disk):** wat has the full homoiconic bridge (`read-string`,
`ast->children`, `with-children`, `ast-kind`, `ast-name`, `ast-span`, `write-forms`) + the span-edit
codemod engine (`fix-text`, STRIKE-READY at `60fc1257`) that splices the **original text** at located
spans, so comments + formatting survive **byte-identical**. So **any future syntax migration is a new
`fix-form` rule + a run of the engine** — not a manual slog, not a comment-destroying round-trip, not
a throwaway Rust harness. wat refactors wat, comments and all, **in wat**.

**The builder's design decision that makes it a *line*, not just a tool:** `fix-wat.wat` **accumulates
every migration rule indefinitely** — `strip-if`, `head-rule`, `arrow-rule`, `type-rule`, the
macro-param-type rule coming next, the 251 cutover rules after. It becomes the **recorded history of
every mass refactor wat has ever done** — each one left behind as both a reusable tool and provenance.
You don't write a one-off and delete it; you add a rule to the ledger and it stays.

**Why this is a maturity line.** A language that can mechanically, faithfully migrate its own corpus
on demand — preserving comments, accumulating its own transformation history — can **evolve its
syntax without fear**. Every future surface change becomes "write a rule, run the engine," reversible
behind dual-read. That is the capability that makes aggressive language evolution *safe and cheap* —
the thing most languages never get and pay for forever. The "wat writes wat" proving point is whole:
the language self-hosts not just its execution but its **own evolution**.

**Cross-references:** `DESIGN-STONE-251.5-4.2-comment-faithful-drive.md` (the span-edit codemod —
wat's `rewrite-clj`); `DESIGN-STRIKE4-corpus-drive.md` (the corpus drive + hard-cuts); `wat/fix.wat`
(the accreting rule library); the span-fidelity fix that hardened `ast-span` (arc-209 REALIZATIONS,
2026-06-14); `feedback_no_magic_that_lets_llm_fake_correctness` (the bug that started the walk);
`feedback_reach_stumble_is_the_signal` (the reach-stumble that redrew the codemod from Rust to wat).
