;; wat/doctest.wat — doctest types and verifier surface (arc 255.1b-iv-b2).
;;
;; Arc 255 Stone iv-b2-a — defines :wat::intrinsic::Example, the typed record
;; returned by the `:wat::intrinsic::examples` reflection seam. Records (not
;; heterogeneous tuples) so `verify-examples` (iv-b2-b) can field-access typed
;; values and pass `expr`/`expected` to `:wat::eval-ast!` without a down-cast.
;;
;; Load order: after Record.wat (uses :wat::Record::def), core.wat (keyword/bool),
;; and the holon/*.wat files (no additional deps beyond those). The seam that
;; RETURNS these records (:wat::intrinsic::examples) is a Rust intrinsic and does
;; not need the record type at registration time — only at call time.

(:wat::Record::def :wat::intrinsic::Example
  [fqdn          <- :wat::core::keyword
   expr          <- :wat::WatAST
   expected      <- :wat::core::Option<:wat::WatAST>
   run           <- :wat::core::bool
   pure          <- :wat::core::bool
   deterministic <- :wat::core::bool])
