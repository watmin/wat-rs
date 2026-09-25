;; Fixture for probe_ex003_construction_annotation_is_honoured.rs — the generic ENUM variant construction path honours `:- [..]` too: a unit
;; variant annotated to match the declared return, and a tagged variant annotated to its value.
(:wat::core::defenum :t::Opt :- [T] :wat::enum::Pure :None [] :Some [v <- T])
(:wat::core::defn :t::mk [] -> (:t::Opt :- [:wat::core::String])
  (:t::Opt.None :- [:wat::core::String] {}))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [_n (:t::mk)
                    s (:t::Opt.Some :- [:wat::core::i64] {:v 1})
                    _ (:wat::kernel::println (:t::Opt.Some/v s))] nil))
