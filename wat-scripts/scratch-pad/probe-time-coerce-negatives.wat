;; Rows 3–4 of EXPECTATIONS-time-crosses-the-boundary.md.
;; Duration handed a String; Duration handed Integer(-1). Both via
;; :wat::edn::validate so the coerce arms, not a blanket-accept, are what fire.
(:wat::config::set-redef! true)

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [s (:wat::edn::validate "nope" :wat::time::Duration)
     n (:wat::edn::validate -1 :wat::time::Duration)
     z (:wat::edn::validate 0 :wat::time::NonZeroDuration)]
    (:wat::kernel::println
      (:wat::core::format "string={s};neg={n};zero-nzd={z}"
        :s (:wat::core::show s)
        :n (:wat::core::show n)
        :z (:wat::core::show z)))))
