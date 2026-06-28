;; Co-located fixture for probe_time_duration_readout.rs — instant_delta_reads_as_a_number.
;; (now - 5s-ago) is ~5s; read as whole seconds it is >= 4 (truncation slack).

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::time::seconds
    (:wat::time::- (:wat::time::now) (:wat::time::seconds-ago 5))))

