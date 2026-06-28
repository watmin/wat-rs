;; Negative fixture: nil body against i64 signature → ReturnTypeMismatch.
;; Used by test: value_position_nil_against_i64_recipient_fires_type_mismatch

(:wat::core::defn :t::probe [] -> :wat::core::i64 nil)
