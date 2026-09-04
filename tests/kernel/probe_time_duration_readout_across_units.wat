;; Co-located fixture for probe_time_duration_readout.rs — duration_reads_across_units.
;; Cross-unit conversion: 1ms = 1_000_000ns.

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::time::nanoseconds (:wat::time::Milliseconds 1)))

