;; tests/reflection/wat_arc201_structured_signature_types_atomic_plus.wat
;; Fixture for test signature_of_defn_emits_atomic_for_monomorphic_path_types.
;; Probe: signature-of-defn :wat::core::i64::+ (all-Path types) stays atomic.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
              [sig
                (:wat::runtime::signature-of-defn :wat::i64::+)
               rendered
                sig]
              (:wat::kernel::println rendered)))
