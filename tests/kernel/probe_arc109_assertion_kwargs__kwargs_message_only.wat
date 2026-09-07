;; The TARGET form — a plain fail carries NEITHER :actual NOR :expected (STOP-4).
;; RED at HEAD: the kwargs spelling does not exist.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::assertion-failed! :message "probe"))
