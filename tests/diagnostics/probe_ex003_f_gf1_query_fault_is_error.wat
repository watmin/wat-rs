;; tests/diagnostics/probe_ex003_f_gf1_query_fault_is_error.wat — co-located fixture for
;; probe_ex003_f_gf1_query_fault_is_error.rs.
;;
;; Excursus 003 strike F, GF1: drives a REAL `:wat::query::Fault` producer through the
;; `lift-fault` narrowing path (wat/query/sqlite-store.wat) — a real sqlite failure (open a
;; nonexistent directory) becomes a `:wat::sqlite::Fault`, which `lift-fault` narrows into a
;; `:wat::query::Fault`, PROPAGATING the original `:location` rather than minting a new one
;; at the narrowing call site. Proves the value satisfies `:wat::core::Error`: accepted where
;; `[e <- :wat::core::Error]` is declared, round-trips through `edn::write`/`edn::read`.
;;
;; Strike F2 (GF2a): the ORIGINAL sqlite `classify` call site now derives a user-source
;; `:location` (via `:wat::kernel::error-site`) rather than minting `wat/sqlite.wat`'s own
;; line, so the location `lift-fault` propagates here is THIS FILE's own call site — never
;; `wat/sqlite.wat`, and never `lift-fault`'s own call site (wat/query/sqlite-store.wat).

(:wat::core::defn :probe::describe [e <- :wat::core::Error] -> :wat::core::String
  (:wat::core::Error/message e))

(:wat::core::defn :user::verify [] -> :wat::core::String
  (:wat::core::match (:wat::sqlite::open "/nonexistent-dir-arc278-strike-f-query/x.db")
    [:wat::core::Result.Err {:error [:wat::sqlite::Error.Fatal {:fault f}]}
      (:wat::core::let [qf   (:wat::query::lift-fault f)
                        msg  (:probe::describe qf)
                        s    (:wat::edn::write qf)
                        back (:wat::edn::read s)
                        loc  (:wat::query::Fault/location qf)]
        (:wat::string::interpolate "{file}:{line}:{col} :: {msg}"
          :file (:wat::core::Span/file loc)
          :line (:wat::i64::to-string (:wat::core::Span/line loc))
          :col  (:wat::i64::to-string (:wat::core::Span/col loc))
          :msg  msg))]
    [:wat::core::Result.Ok {:value _v}
      (:wat::kernel::assertion-failed! :message "open of a nonexistent directory must fail")]
    [_ (:wat::kernel::assertion-failed! :message "expected :wat::sqlite::Error.Fatal")]))
