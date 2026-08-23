;; tests/reflection/probe_arc255_reflection_parity.wat
;; just-eval fixture for probe_arc255_reflection_parity.rs — the bare-world
;; metadata-of cases (reflection parity between rust builtins and user forms).
;;
;; metadata-of carries no registered TypeScheme (its arg is a binding-name
;; keyword resolved at runtime), so its inferred return is permissive; the
;; declared Option<HashMap<keyword, HolonAST>> here mirrors its documented shape
;; (runtime.rs eval_metadata_of). The Rust driver inspects the returned Value.

;; metadata-of on a rust builtin (:wat::core::i64::+) — RED at HEAD returns None
;; (builtins registered nowhere); GREEN after arc 255.1 returns Some(baseline).
(:wat::core::defn :user::builtin-metadata []
  -> (:wat::core::Option :- [(:wat::core::HashMap :- [:wat::core::keyword :wat::holon::HolonAST])])
  (:wat::runtime::metadata-of :wat::core::i64::+))

;; metadata-of on the Bytes::to-hex intrinsic — the full map, for the diagnostic dump.
(:wat::core::defn :user::to-hex-metadata []
  -> (:wat::core::Option :- [(:wat::core::HashMap :- [:wat::core::keyword :wat::holon::HolonAST])])
  (:wat::runtime::metadata-of :wat::core::Bytes::to-hex))
