;; Negative fixture: empty (do) is MalformedForm.
;; Used by test: do_empty_form_is_malformed

(:wat::core::defn :t::compute [] -> :wat::core::i64 (:wat::core::do))
