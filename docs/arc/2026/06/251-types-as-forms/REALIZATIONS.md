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

---

## 2026-06-14 — THE RUNNER SELF-HOSTS: the engine got a runner, in wat, through the CLI (Song #96 *Again We Rise*)

The companion to the maturity-line entry above: that one named the capability (wat can *express* a
comment-faithful codemod); this one records the line crossed **again, one layer deeper** — wat now
*executes* its codemods on its own corpus, **through its own CLI, with no Rust in the loop**.

**The first rule rode the engine.** `fix-macro-param-types` shipped (`73113b9d`) — the first real
entry in `fix-wat.wat`'s permanent ledger, rewriting a defmacro's discarded param/return types to the
honest `:wat::WatAST` / `:wat::core::Vector<wat::WatAST>` (fixed / rest). Comment-faithful, idempotent,
defmacro-scoped (sibling `defn`/`fn` real types untouched).

**The read ladder was missing — the entry above was optimistic.** The maturity-line note listed
`read-file` among the primitives that "now exist." It did **not** — `io.wat` shipped the whole *write*
ladder (`write-file` / `with-open-file` / `IOWriter/open-file`) and **nothing on the read side**;
`IOReader` could only wrap in-memory bytes. The runner exposed it: the builder caught a Rust harness
doing the corpus run and asked *"why can the wat CLI not do this?"* — the reach-stumble. So the rung
was built, not routed around (`3dc0f48d`):
- `IOReader/open-file` — fd-backed read handle (Rust, exact mirror of `IOWriter/open-file` via `PipeReader`)
- `IOReader/read-all-string` — byte-faithful UTF-8 slurp (Rust, mirror of `IOWriter/to-string`)
- `:wat::io::read-file` — the one-shot (wat defn, mirror of `write-file`)
warded by `tests/probe_arc251_read_file_ladder.rs` (write→read round-trip, byte-identical incl. unicode + trailing newline).

**The runner.** `wat-scripts/fix-macro-param-types.wat` — `main []` reads an EDN vector of paths via
`(:wat::kernel::readln -> :Vector<String>)`, recurses `first`/`rest`, `read-file` →
`fix-macro-param-types` → `write-file`. Migrated all 15 stdlib files carrying `:AST<...>` /
`:wat::holon::HolonAST` lies → the honest forms, **100/100 balanced replacements, line counts
preserved**, idempotent — run through `./target/release/wat`, exit 0.

**Two findings the runner forced into the open:**
1. **The `wat-scripts/` graveyard.** Writing one current program hit *four* retired-form walls —
   `:()`→`:wat::core::nil` (153), `:wat::core::define`→`defn` (241), `main`-with-stdio-args→`main []`
   + stdio services (170), and the CLI takes no trailing args (stdin only). The existing examples
   (`count-logs`/`seed-fixture`) don't run today. Migration is itself a future fix-wat job — stubbed.
2. **The native-stack recursion ceiling, live (arc 261).** The runner SIGSEGV'd on the largest file
   (`test.wat`, ~960 lines) — eval recurses on the native stack, overflowing the forked child's 8MB
   `RLIMIT_STACK`. Papered with a `setrlimit`-before-fork stopgap in `wat-cli` under
   `rune:exigere(attested-arc)` → arc 261 (`21ee6807`, the CEK stub, which owns the delete-on-landing
   condition). RAISES the ceiling to 1 GiB; CEK removes the class. The debt is marked, not hidden.

**What is now true (on the disk):** wat refactors wat — comments and all, **in wat, run by wat's own
CLI**. Every future mass-refactor is a new rule in `fix-wat.wat` + a run of the self-hosted runner.
The language self-hosts its execution *and* its evolution *and* the act of carrying that evolution out.

**Cross-references:** Song #96 *Again We Rise* (arc-170 INTERSTITIAL-REALIZATIONS, 2026-06-14);
`wat-scripts/fix-macro-param-types.wat` (the runner); `tests/probe_arc251_read_file_ladder.rs` (the
read ladder warded); `docs/arc/2026/06/261-eval-stack-safety-cek/STUB.md` (the recursion ceiling + the
stopgap it owns); `feedback_reach_stumble_is_the_signal` (the "why not the CLI?" stumble that built
the read rung). ENFORCE (the macro-param-type validator) + its corpus cascade remain a separate
deferred thread.
