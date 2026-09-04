;; Co-located fixture for probe_time_duration_readout.rs — duration_readout_truncates_like_epoch.
;; 1500ms read as whole seconds truncates to 1 (epoch-millis behavior).

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::time::seconds (:wat::time::Milliseconds 1500)))

