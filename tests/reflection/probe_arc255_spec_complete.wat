;; tests/reflection/probe_arc255_spec_complete.wat
;; just-eval fixture for probe_arc255_spec_complete.rs — the bare-world intrinsic
;; probes (variadic-args-measurement, render-doc, metadata-of). Each named entry
;; is a zero-arg fn the Rust driver invokes via call_beside; the Rust side
;; inspects the returned typed Value.

;; ─── Part A: variadic-args-measurement ───────────────────────────────────────
(:wat::core::defn :user::variadic-three [] -> :wat::core::i64
  (:wat::intrinsic::variadic-args-measurement 1 2 3))
(:wat::core::defn :user::variadic-zero [] -> :wat::core::i64
  (:wat::intrinsic::variadic-args-measurement))
(:wat::core::defn :user::variadic-one [] -> :wat::core::i64
  (:wat::intrinsic::variadic-args-measurement :x))

;; ─── Part B/C: render-doc goldens ─────────────────────────────────────────────
(:wat::core::defn :user::render-yields [] -> :wat::core::String
  (:wat::core::render-doc :wat::intrinsic::yields-witness))
(:wat::core::defn :user::render-to-hex [] -> :wat::core::String
  (:wat::core::render-doc :wat::core::Bytes::to-hex))
(:wat::core::defn :user::render-variadic [] -> :wat::core::String
  (:wat::core::render-doc :wat::intrinsic::variadic-args-measurement))

;; ─── metadata-of carries :category ───────────────────────────────────────────
;; metadata-of has no registered TypeScheme (runtime keyword-arg resolution); the
;; declared Option<HashMap<keyword, HolonAST>> mirrors its documented shape.
(:wat::core::defn :user::to-hex-metadata []
  -> (:wat::core::Option :- [(:wat::core::HashMap :- [:wat::core::keyword :wat::holon::HolonAST])])
  (:wat::runtime::metadata-of :wat::core::Bytes::to-hex))
