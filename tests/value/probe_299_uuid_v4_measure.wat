;; tests/value/probe_299_uuid_v4_measure.wat — entropic v4 conformance measure.
;; Rust generates the entropy; wat judges conformance via the new accessors.
;; Arc 299 slice 1.
(:wat::core::defn :probe::measure [s <- :wat::core::String] -> :wat::core::bool
  (:wat::core::match (:wat::uuid::from-string s) 
    ((:wat::core::Some u) (:wat::core::and (:wat::core::= (:wat::uuid::version u) 4)
                                            (:wat::uuid::rfc4122-variant? u)))
    (:wat::core::None false)))
