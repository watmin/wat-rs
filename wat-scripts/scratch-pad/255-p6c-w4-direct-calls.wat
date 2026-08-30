;; Scratch probe — arc 255 Stone P6-c-W4, acceptance row 5.
;;
;; Direct, statically-checked calls to the three W4 verbs at legal arity. Output
;; must be byte-identical before and after homing — only the DISPATCH mechanism
;; changes, never the returned value.
;;
;; `metadata-of` is called on `:wat::i64::+`, the CORPUS shape used by the real
;; fixture `tests/reflection/probe_arc255_reflection_parity.wat` (a rust-builtin
;; FQDN, not a synthetic name) — chosen because 167 corpus call sites make its
;; ordinary shape the one that matters most. `field-names-of`/`field-types-of`
;; are called on a defstruct, mirroring `wat-scripts/probes/arc-170/probe-strikeB-fields.wat`.

(:wat::core::defstruct :probe::W4Bag [n <- :wat::core::i64  s <- :wat::core::String])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::core::match (:wat::runtime::metadata-of :wat::i64::+)
      ((:wat::core::Some hm) (:wat::kernel::println (:wat::string::concat "metadata-of-i64+ :name= " (:wat::edn::write (:wat::hashmap::get hm :name))
        " :arity= " (:wat::edn::write (:wat::hashmap::get hm :arity)))))
      (:None (:wat::kernel::println "metadata-of-i64+ UNEXPECTED NONE")))
    (:wat::kernel::println (:wat::string::concat "field-names-of-W4Bag= " (:wat::edn::write (:wat::runtime::field-names-of :probe::W4Bag))))
    (:wat::kernel::println (:wat::string::concat "field-types-of-W4Bag= " (:wat::edn::write (:wat::runtime::field-types-of :probe::W4Bag))))))
