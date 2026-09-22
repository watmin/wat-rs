;; Stone 255.12, site 2 — the MACRO registry's cross-spelling arm, measured.
;;
;; The 255.12 brief's lead was that `macros/registry.rs::ast_same_identity`'s
;; cross-spelling arm is blocked by its `id.is_reference()` requirement, and that
;; this is what makes `wat/holon/Ngram.wat` a duplicate under conversion.
;; ⛔ REFUTED. This file is the positive control: the SAME macro, once in each
;; dialect — every head and every annotation re-spelled — and it registers as a
;; NO-OP on the pre-255.12 binary already. `canonical_identity` was doing its job.
;; The Ngram failure is the `.wat.bad` sibling, and it is not an identity question.
(:wat::core::defmacro :p255_12::M [x <- :wat::WatAST] -> :wat::WatAST
  `(:wat::i64::+ ~x 1))
(:wat::core::defmacro :p255_12::M [x <- wat/WatAST] :- wat/WatAST
  `(wat.i64/+ ~x 1))
