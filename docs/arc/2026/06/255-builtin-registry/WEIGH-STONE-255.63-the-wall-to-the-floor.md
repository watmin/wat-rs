# WEIGH — STONE 255.63: layer 4 — STOP-1 ACCEPTED (a design decision, correctly not taken)

**Executor: grok via pulsare, commit `216c583e5`** (the SCORE; `probes-255.63/fix-f1.diff`, the codemod's F1 rule;
`rest-param-head.diff`; the worktree removed). Weighed by the orchestrator on 2026-09-27.

## What layer 4 found

- **F1's codemod rule works, narrowed:** a keyword is a type head only when the form is exactly three children, head
  `:-` vector. The first, wider cut renamed `defenum` declarations (`wat.core/Option` → `wat.type/Option`) and broke
  the Rust-side `wat_enum_register_from!` lookup (`src/types.rs:2198-2199`). In the converted stdlib: 1,238
  `(wat.type/… :-` heads, 2,685 `wat.type/` tokens, 0 three-child `wat.core` type forms left.
- **One spelling cure:** `is_watast_vec` (`src/macros/parse.rs`) compared a rest param's head to `wat::core::Vector`;
  it now accepts both.
- **STOP-1, the F5 expand-time purity gate.** `wat.core/kwargs-lower`'s rest param is typed
  `(wat.type/Vector :- [wat/WatAST])`. The macro-body purity walk treats that type form as an **expand-time call** of
  its head, and `is_expand_time_legal` (`src/macros/eval.rs:458`) default-denies `:wat::type::Vector` because no
  intrinsic is registered under that name.
- **The latent bug this exposes:** under the old spelling the same type form passed only because
  `:wat::core::Vector` **is** a registered pure verb, the vector constructor. The gate was reading a **type** as a
  **call** all along, and it happened to be admitted. F1 did not create the fault; it removed the coincidence that
  hid it.

## Findings carried to the builder

- The F5 decision (below).
- **Stdlib-declared types keep `wat.core/` names** (`defenum wat.core/Option`, `wat.core/Result`, …), because the
  Rust side looks them up by `:wat::core::Option`. F1 says every builtin type is `wat.type/…`. Whether `Option`/`Result`
  are "builtin types" under F1, and so need a `wat.type` canonical key, is open.
- `WatAST`'s name (N-keep / N-AST) is still the builder's.

## Not reached

Phase 2 (~2,230 files), the doc conversion, both walls, the floor, the census. The converted-floor number is still
owed.
