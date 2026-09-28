# WEIGH — STONE 255.64: layer 5 — ACCEPTED (measurement; the converted stdlib does not load)

**Executor: grok via pulsare, commit `313075d53`** (the SCORE; `probes-255.64/fix-f1.diff`, the codemod reading the
builtin registration via `:wat::keyword::language-provided-spelling`; `probes-255.64/src.diff`, P1 + the narrowed F1
wall + `wat.type/AST` through `type_denotation` + the doc grammar narrowed to namespaced symbols + `fqdn_of`'s enum
branch; the worktree removed). Weighed by the orchestrator on 2026-09-27.

## Built and measured

- **The narrowed F1 rule reads the registration, not a list:** language-provided means `classify` is Builtin under
  `:wat::core::` / `:wat::time::` / `:wat::WatAST`; declared types (`Option`, `Result`, every `defenum`) and subsystem
  builtins (`HolonAST`, `Peer`, `Stream`) keep their namespace. `:wat::core::Tuple` is not registered, so it stays
  `wat.core/Tuple`. That is a registration gap, not the ruling.
- **P1 is in the macro-purity walk** (`validate_pure_total`, `src/macros/eval.rs`). No second walk needed it.
- **`wat.type/AST`** reaches the canonical `:wat::WatAST` through `type_denotation`, as do `wat.type/Instant`/`Duration`.

## The two blockers

1. **`extend-type`'s type arguments are type positions the codemod did not mark.** `wat/class.wat:16`
   `(wat.core/extend-type wat.core/i64 wat.core/Orderable)` is refused by the F1 wall. That is a spelling. The rule
   grok added lives in `wat/fix.wat`, which is **itself stdlib**: it has to be applied to the unconverted `fix.wat`
   *before* the conversion. Bootstrap order, not a design question.
2. **With the wall paused, the checker reports 534 errors**, starting at `conj` of the empty constructor
   `(wat.core/Vector :- [wat.type/AST])` (`wat/bracket.wat:356`, then `select`/`send`/`nth`/`foldl`/`mapv` in
   `bracket.wat` and `cache.wat:203`). `unify`'s Path arm already equates spellings through `type_denotation`
   (orchestrator read), so **the fault is elsewhere**. The orchestrator suspects the recurring class (CLAUDE.md's
   corollary: a comparison with one side normalized and the other not); `type_denotation` is consulted at 31 sites,
   which means every other comparison site is a place two spellings can disagree. **Suspected, not measured.**

## A refinement to P1 the measurement forces

The stdlib uses `(wat.core/Vector :- [wat.type/AST])` in **value** position as the empty **constructor**. The **head's
namespace already disambiguates**: `wat.type/Vector` is the type, and `wat.core/Vector` is the constructor function. P1
("a `(Head :- [...])` form is a type, never a call") holds for `wat.type/` heads. A `wat.core/` constructor head with
`:-` is a call. For the builder to confirm.
