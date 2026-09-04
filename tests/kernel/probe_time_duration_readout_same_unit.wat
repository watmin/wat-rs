;; Co-located fixture for probe_time_duration_readout.rs — duration_reads_back_in_same_unit.
;; Identity round-trip: 1500ms built as Millisecond reads back as 1500ms.

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::time::milliseconds (:wat::time::Milliseconds 1500)))

