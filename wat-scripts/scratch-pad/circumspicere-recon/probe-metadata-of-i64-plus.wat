;; Scratch probe (circumspicere recon, read-only) — mirrors the fixture used by
;; the #[ignore]d test `metadata_of_answers_for_a_rust_builtin`
;; (tests/reflection/probe_arc255_reflection_parity.rs / .wat). Its ignore
;; reason claims "arc-255 metadata-of reflection (builtin-registry) not yet
;; built" — checking whether that premise still holds.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [m (:wat::runtime::metadata-of :wat::i64::+)]
    (:wat::core::match m
      ((:wat::core::Some hm) (:wat::kernel::println "SOME (builtin metadata-of answered)"))
      (:None (:wat::kernel::println "NONE (ignore reason still true)")))))
