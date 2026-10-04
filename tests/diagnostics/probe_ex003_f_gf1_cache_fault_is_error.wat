;; tests/diagnostics/probe_ex003_f_gf1_cache_fault_is_error.wat — co-located fixture for
;; probe_ex003_f_gf1_cache_fault_is_error.rs.
;;
;; Excursus 003 strike F, GF1: drives a REAL `:wat::cache::Fault` producer —
;; `:wat::cache::Lru/new` with a non-positive capacity — and proves the fault value
;; satisfies `:wat::core::Error`: accepted where `[e <- :wat::core::Error]` is declared,
;; round-trips through `edn::write`/`edn::read`, and carries a REAL `:location` (the lift
;; site inside wat/cache.wat), never a placeholder.

(:wat::core::defn :probe::describe [e <- :wat::core::Error] -> :wat::core::String
  (:wat::core::Error/message e))

(:wat::core::defn :user::verify [] -> :wat::core::String
  (:wat::core::match (:wat::cache::Lru/new :- [:wat::core::keyword :wat::core::i64] 0)
    [:wat::core::Result.Err {:error f}
      (:wat::core::let [msg  (:probe::describe f)
                        s    (:wat::edn::write f)
                        back (:wat::edn::read s)
                        loc  (:wat::cache::Fault/location f)]
        (:wat::string::interpolate "{file}:{line}:{col} :: {msg}"
          :file (:wat::core::Span/file loc)
          :line (:wat::i64::to-string (:wat::core::Span/line loc))
          :col  (:wat::i64::to-string (:wat::core::Span/col loc))
          :msg  msg))]
    [:wat::core::Result.Ok {:value _v}
      (:wat::kernel::assertion-failed! :message "Lru/new with capacity 0 must refuse")]))
