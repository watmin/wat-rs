;; Scratch — arc 255 Stone P3, row 3 re-diagnosis. Dumps metadata-of(:wat::core::Bytes::to-hex)
;; to confirm the SHIPPED contract: :purity/:determinism are Value::Enum
;; (:wat::runtime::Purity/Pure, :wat::runtime::Determinism/Deterministic), not plain
;; :pure/:deterministic bools. See tests/reflection/probe_arc255_ivc_metadata_plain_values.rs.
(:wat::core::defn :user::dump-to-hex-metadata []
  -> (:wat::core::Option :- [(:wat::core::HashMap :- [:wat::core::keyword :wat::holon::HolonAST])])
  (:wat::runtime::metadata-of :wat::core::Bytes::to-hex))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:user::dump-to-hex-metadata)))
