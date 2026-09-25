;; Fixture for probe_ex003_construction_annotation_is_honoured.rs — annotated constructions that
;; are RIGHT check and run: a struct, the empty binder `:- []` (expressed, legal everywhere), a
;; record with two params and kwargs out of declared order, and the `kwargs-construct` verb
;; written directly. The runtime erases the spec (types are the checker's); these prove it is
;; peeled, not read as a `:-` kwarg.
(:wat::core::defstruct :t::Box :- [T] [x <- T])
(:wat::core::defrecord :t::Pair :- [A B] [a <- A b <- B])
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [b (:t::Box :- [:wat::core::i64] :x 7)
                    e (:t::Box :- [] :x "empty-binder")
                    p (:t::Pair :- [:wat::core::String :wat::core::i64] :b 2 :a "one")
                    d (:wat::core::kwargs-construct :t::Box :- [:wat::core::i64] :x 9)
                    _ (:wat::kernel::println (:t::Box/x b))
                    _ (:wat::kernel::println (:t::Box/x e))
                    _ (:wat::kernel::println (:t::Pair/a p))
                    _ (:wat::kernel::println (:t::Pair/b p))
                    _ (:wat::kernel::println (:t::Box/x d))] nil))
