# WEIGH — STONE 255.65: the cutover, sized — ACCEPTED (measurement)

**Executor: grok via pulsare, commit `eca239720`** (the SCORE only). Weighed by the orchestrator on 2026-09-27.

## The reading

- **255.60's 5,745 does not reproduce.** Remeasured two ways: **2,200** whole-string `:ident(::ident)+` literals in
  `src/` (189 files), and **23,538** keyword-shaped tokens in `src/` (comments and prose included), plus 597 in
  `crates/`. Of the 920 whole-string `:wat::core::…` literals: 481 are function/form heads (→ `wat.core/`), 295 are
  hard-primitive types (→ `wat.type/`), 20 are `Option`/`Result` (stay `wat.core/`), and 77 need question 1
  (`PersistentMap`/`PersistentVector`, `Uuid`/`Bytes`/`Record`). **A rewrite is exact only for a whole-string name:**
  3,256 Rust strings embed `wat::` inside a sentence, template or example. Those are the wall's last layer.
- **Identity lives in a dozen doors.** `canonical_identity` and a second copy in `crates/wat-source-derive/src/lib.rs:105`;
  `type_denotation` (dead after one spelling); `ns_to_wat_path`; `format_type` (319 calls: what users see in a type
  mismatch); `AggregateValue.class` / `type_name`. **The derive's copy must move with the first**, or registration
  keys and runtime keys diverge.
- **Constructors that are types:** on the unconverted tree, `Vector` has 4,083 `(Vector :- …)` forms and only 38 bare
  calls. The 4,107 in 255.62 counted the converted tree. `PersistentVector` has 1,210 bare calls and `Tuple` 541: the
  literal-vs-typed choice is real there. A splice (`~@`) cannot become a literal.
- **Homes:** `wat.type/AST`, `wat.time/Instant`/`Duration`, `wat.uuid/UUID` (24 `.wat` hits, 18 `src` literals, a case
  change), `wat.holon/HolonAST`. Core records (`Span`, `Error`, `Fault`, …) stay `wat.core/`, so the 398
  `#wat.core/Span` tags survive.
- **Bootstrap:** the position rule has to be in the checker **before** a converted stdlib loads. The codemod rules have
  to be in the **unconverted** `fix.wat`. Walls last.
- **Branches:** `grok-rete` 653, `sns-sqs` 551, `queue-promotion-blocked-on-startup-cost` 486 commits not on `main`, each
  touching `src/` and `wat/`. Inferred: rebase them after the cutover.

## For the builder

Two questions the rulings do not answer (the SCORE's own, verbatim in substance):

1. Which registered core names are hard primitives: `u8`, `bigint`, `rational`, `char`, `nil`, `Value`,
   `PersistentVector`, `PersistentMap`, `Bytes`?
2. May the sequence carry a **temporary** second key for one or two stones (so each stone lands green), with a named
   stone that deletes it?

The proposed sequence, if (2) is yes: **1** position (Rust) → **2** codemod, types only → **3** home renames (`Uuid`) →
**4** C1 for primitives, which deletes their door → **5** function/form heads → **6** printers and goldens → **7** walls.
