;; tests/types/probe_arc255_74_key_must_be_data__control.wat
;; The control: no HashSet/HashMap/PersistentMap/conj/assoc at all. Non-zero here means the
;; harness, the binary, or the fixture path is broken — not that the stone's subject is.
(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::kernel::println "control"))
