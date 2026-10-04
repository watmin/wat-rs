;; tests/diagnostics/probe_ex003_f_gf1_sqlite_fault_is_error.wat — co-located fixture for
;; probe_ex003_f_gf1_sqlite_fault_is_error.rs.
;;
;; Excursus 003 strike F, GF1: drives a REAL `:wat::sqlite::Fault` producer — opening a
;; nonexistent directory, which `:wat::sqlite::classify` (wat/sqlite.wat) lifts into
;; `Error.Fatal {fault}` — and proves the fault value satisfies `:wat::core::Error`:
;; accepted where `[e <- :wat::core::Error]` is declared, round-trips through
;; `edn::write`/`edn::read`, and carries a REAL `:location` (the `classify` call site in
;; wat/sqlite.wat), never a placeholder.

(:wat::core::defn :probe::describe [e <- :wat::core::Error] -> :wat::core::String
  (:wat::core::Error/message e))

(:wat::core::defn :user::verify [] -> :wat::core::String
  (:wat::core::match (:wat::sqlite::open "/nonexistent-dir-arc278-strike-f/x.db")
    [:wat::core::Result.Err {:error [:wat::sqlite::Error.Fatal {:fault f}]}
      (:wat::core::let [msg   (:probe::describe f)
                        s     (:wat::edn::write f)
                        back  (:wat::edn::read s)
                        loc   (:wat::sqlite::Fault/location f)]
        (:wat::string::interpolate "{file}:{line}:{col} :: {msg}"
          :file (:wat::core::Span/file loc)
          :line (:wat::i64::to-string (:wat::core::Span/line loc))
          :col  (:wat::i64::to-string (:wat::core::Span/col loc))
          :msg  msg))]
    [:wat::core::Result.Ok {:value _v}
      (:wat::kernel::assertion-failed! :message "open of a nonexistent directory must fail")]
    [_ (:wat::kernel::assertion-failed! :message "expected :wat::sqlite::Error.Fatal")]))
