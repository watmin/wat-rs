;; tests/reflection/probe_arc255_ivb1_structured_doc.wat
;; just-eval fixture for probe_arc255_ivb1_structured_doc.rs.
;;
;; Returns the full metadata-of(:wat::core::Bytes::to-hex) map; the Rust driver
;; asserts the structured-doc keys (:added / :ret) are present. metadata-of
;; carries no registered TypeScheme (runtime keyword-arg resolution), so the
;; declared Option<HashMap<keyword, HolonAST>> mirrors its documented shape.
(:wat::core::defn :user::to-hex-metadata []
  -> (:wat::core::Option :- [(:wat::core::HashMap :- [:wat::core::keyword :wat::holon::HolonAST])])
  (:wat::runtime::metadata-of :wat::core::Bytes::to-hex))
