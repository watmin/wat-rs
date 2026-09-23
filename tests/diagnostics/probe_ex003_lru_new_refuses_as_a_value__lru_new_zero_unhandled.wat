;; Fixture for probe_ex003_lru_new_refuses_as_a_value.rs — the UNHANDLED refusal.
;;
;; The-little-wat's F-084 program passed a non-positive capacity and did nothing about it. This is
;; that program at the cured HEAD: `Result/expect` is the corpus's disposition for a site whose
;; capacity is a literal (`wat/query/sqlite-store.wat`'s `:init` does the same with
;; `:wat::sqlite::open`), so the refusal still ENDS the program — what changed is the face it
;; wears. Before the cure this died with a Rust `panic!`: a `RUST_BACKTRACE` note, the internal
;; `:rust::cache::Lru/new` name, and no span at all. The probe pins the whole stderr.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [cache (:wat::core::Result/expect
             (:wat::cache::Lru/new :- [:wat::core::keyword :wat::core::i64] 0)
             ":wat::cache::Lru/new refused the capacity: it must be positive")]
    (:wat::kernel::println (:wat::cache::Lru/len cache))))
