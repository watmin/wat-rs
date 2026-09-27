# WEIGH — STONE 255.62: the wall, layer 3 — ACCEPTED (measurement)

**Executor: grok via pulsare, commit `a5ddd36ee`** (the SCORE and `probes-255.62/`: the two link cures and the F1 wall
as diffs; the worktree removed). Weighed by the orchestrator on 2026-09-27.

## What layer 3 found

- **Both link cures work.** `type_token_shape_ok` (`crates/wat-doc/src/lib.rs`, wired at `@arg`/`@ret` **and** the two
  copies in the special-form parser) and `fqdn_of`'s plain case returning the dotted spelling. 255.61's 501 errors are
  gone.
- **One token blocks the link:** `src/reflect/verbs.rs:1911`, the `@example`'s expected value `wat::cache::Lru.Hit`,
  one of the seven spans `fix-text` left unchanged (a symbol with `::` and a dot, which the call-head rule does not
  match).
- **F1's cost is two codemod positions.** A scan of the converted tree (not floor failures; the checker never ran):
  12,407 `wat.core/<type>` in type positions. **12,395** come from the head of `(Head :- [...])` (6,905) and the
  elements of its `:- [...]` bracket (5,490). The post-arrow rule already emits `wat.type/` (15,321 tokens) and missed
  12, in syntax-quote or skipped binders.
- The F1 wall's reach is recorded: `forbidden_core_type_symbol` in `parse_type_node` and `parse_type_form`, which every
  type position the scan listed goes through (checker slots, binders, field types, variant payloads, bounds).

## Two notes for the next layer

- The link cure accepts **any** symbol as a doc type token. The test
  `bare_symbol_without_colon_is_still_refused_by_the_colon_rule` guards against a bare `Bytes` standing in for a type.
  The cure should accept a **namespaced** symbol (`wat.type/…`, `u/Person`), and keep refusing a bare one.
- `fqdn_of`'s enum branch (`compose_variant`, `crates/wat-macros/src/edn_doc.rs:281`) still mints `::`. It did not
  fire here.

## The loop's cost

Each layer re-converts the corpus (~45 min) and has so far stopped on the first compile error. Layer 3's blocker is one
respelling. The next layer should cure **pure respellings in doc directives inline**, listing each one, and keep going,
so the floor finally runs.
