# arc 278 — NEXT ANGLES (post-P11): perf + capability backlog, ordered

**State after P11:** native `fire-rules'` beats Clara across the entire measured grid — deep-cascade 5×5..30×10
(ours, 1.2–6.3×) + fan-out 16k/20k/40k (40k: 134ms → **89.8ms** < Clara ~96ms). The support chain is kept and
walkable (guiding-light substrate proven by the kernel probe). Differentials green; oracle untouched.

## Ordering (orchestrator's bias — guiding light first)

### ① EXPLAIN walk (P12) — THE GUIDING LIGHT. Top priority. Prove it IN WAT.
"How did this fact get derived" — the operator diagnostic, non-negotiable. Answers the open questions
(can-we-prove-the-diagnostic + prove-it-in-wat) together.
- Re-introduce the **fact→producing-token index** (the 4c cut) — the entry point to walk from a derived fact.
- The recursive backward walk: derived fact → producing token → `token.matches` (the `(fact, alpha_id)`
  condition-edges) → supporting facts → recurse to input facts.
- An **explain-retaining fire** (does not clear beta, or hooks native beta before the P11 clear) — the diagnostic
  path; the fast fire stays lean.
- A wat `explain` surface + a **wat demo** that fires cold-and-windy / a deep cascade, takes a derived fact, and
  prints the why-tree to its inputs. The property graph (nodes = facts, edges = conditions) navigated, in wat.

### ② Rule-count / node-sharing perf matrix — the last unmeasured Clara cell
The one regime where Clara's maturity could still hide an edge: **beta / join-prefix subtree sharing** at high
rule count with shared prefixes. We have alpha-sharing (`find-or-mint-alpha`); beta/join-prefix sharing is
unconfirmed. Add the rule-count × shared-prefix dimension to the matrix harness; bench vs Clara. Closes the
"anything left vs Clara" question. (Smaller than ①; can interleave.)

### ③ Structured Condition + Snapshot (DESIGN-STONE-S) — foundation
Durable structured `:lhs`/`:rhs` (Clara-parity, leaf exprs raw) + `parse-rule` + compile-consumes-structured; the
`{facts, rules}` EDN snapshot + revive/replay + the round-trip foundation probe. Enables EXPLAIN's
**"which gate MISFIRED"** overlay (needs the full rule structure to diff against the runtime edges) + the
S3-blob triage workflow + programmatic/ML. (DESIGN already drawn: `DESIGN-STONE-S-snapshot-and-data-tooling.md`.)

### ④ Capability stones 6–8 — close the Clara capability gap (KEEP-planned, not conceded)
`:test` (expr-joins / `where`) · negation (`:not` = a condition) · accumulators (incl the `acc/`-in-LHS
"minimum finding set to activate" — a DDoS primitive). The reduced-spec's KEEP set; building them closes the
honest capability gap vs Clara.

### ⑤ kernel.rs vigilia ward pass — hygiene before 278 closes
`kernel.rs` grew big across P4–P11 (+ P11 unused-import warnings). A vigilia ward pass before declaring 278
closed.

### ⑥ persistent rete Session in a process — the engine's first serious *service* app (now-buildable)
A `defservice` whose state IS a `Session`: iterative `insert` / `fire-rules` / `retract` / `query` / `explain`
messages over a LIVE working memory held in a process, proven over process/UDS. Proves **continuity** (the
gen_server threads `Session → Session'` across requests — pure engine within a fire, stateful actor across them)
and **retraction** live (`retract` + re-fire = pure replay; TM falls out — the hard textbook part, trivial by
purity). It's a deductive db (insert=write, retract=delete, query=read, fire-rules=infer). Zero new substrate —
composition of the engine + `defservice` + spawn; a probe spawns it and does iterative work (insert→fire→query→
retract→re-fire→query) observing the WM evolve. **NO HTTPS / TCP+TLS** (dropped — not mapped). v1 = pure replay;
a hot WM is where incremental insert (P4b delta) + incremental TM (the support-store cascade cut from the pure
oracle) would eventually earn their place. (Reclaimed from a too-greedily-created arc into this backlog.)
- **Read path = copy-on-write overlay** (the "Iron Man builds the armor in the cave, cave untouched" model):
  `query` reads through the immutable base; `:what-if`/`:try-rules` fire over a structurally-shared snapshot,
  read the derivation, discard the overlay — base byte-identical, near-free (persistence). Snapshot isolation,
  no MVCC, no locks. Folds into ⑥'s DESIGN-STONE when drawn. Full model + the **distributed-service HORIZON**
  (a NOT-NOW ddb-style leader/replica design + the evolvability constraint it puts on ⑥):
  `NOTE-overlay-read-path-and-distributed-horizon.md`.

## 278 close condition
perf (done — beat Clara) + tooling (② matrix) + capability (① EXPLAIN + ④ stones 6–8) + ⑤ vigilia → **THEN arc
280** (stdio bound to EdnRepresentable — the stamped #1 next arc; do not open before 278 closes).

## Banked / lower priority
- quote → `'` / `` ` `` lint rule (traditional now → flip to a rete rule once the engine closes); shares its
  sugaring pretty-printer with the structured→surface renderer (③).
- **accessor-idiom lint rule** (born from the first-bare cut): PROMOTE `(nth xs 0)` / `(Option/expect (get xs 0) m)`
  → `(first xs)` (1→`second`, 2→`third`); GUARD: `(match (get xs 0) Some/None)` STAYS `get` (empty-handling).
  Principle: `get` asks "is there one?", `first/second/third` assert "give me the one." Precise spec +
  equivalences: `DESIGN-STONE-first-bare-accessors.md` § Downstream lint rule. `fix-wat` autofix; build after
  the first-bare cascade closes; flip to rete with the rest.
- **lint suppression = mirror the proven `wat-test` sibling-annotation pattern (no new substrate).** Precedent:
  `wat/test.wat:407-427` — `:wat::test::ignore` / `:should-panic` / `:time-limit` are **no-op typed defns**
  (`String -> nil`, type-check but runtime-irrelevant) placed as a **sibling preceding** the target; the tooling
  (proc-macro scanner) reads them; *"attaches to the IMMEDIATELY NEXT [form]; intervening non-annotation forms
  clear the pending annotation."* Lint suppression reuses this exactly: a no-op `(:wat::lint::disable :rule)` /
  `(:wat::lint::ignore "reason")` annotation, sibling before the LIVE form, read by the linter's source-AST walk
  (`deporder.wat` — same walker shape as the test scanner). The code runs; only the lint is suppressed. **Scope
  answered by precedent: next-form, intervening-clears.** Supersedes the metadata-rune idea (`{:wat-lint.disable}`).
- **`(comment ...)` form — the dual of `(quote ...)`** (SEPARATE idea; NOT the suppression vehicle — the
  annotation above covers that). `quote` lifts a form INTO the program (→ AST value); `comment` lifts it OUT
  (→ `nil`, body parsed-but-dropped, never type-checked) yet leaves it in the SOURCE AST for parsers who care.
  The GENERAL homoiconic rich-comment: arbitrary dropped forms / kept-code-as-data. Impl: macro
  `(:wat::core::comment & body) → nil`. Caveat: body must be lex/parse-valid s-exprs (Clojure's `;` vs `(comment)`
  split; `;;` stays for free text). Decide on rich-comment merits, independent of suppression.
- tail-latency / GC-jitter measurement vs Clara (structural claim; lower priority).
- **The reborn linter** (lint rules as rete rules) — the engine's first serious app; the *why* of arc-278.
  The accumulating rule roster above (quote-sugar, accessor-idiom, positional-construction, nested-if-ladder,
  concat-abuse) is the first cohort to flip from if/cond wat to rete rules once the engine closes.
