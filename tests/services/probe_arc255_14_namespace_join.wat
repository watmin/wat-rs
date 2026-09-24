;; Arc 255 Stone 255.14 — "a namespace is not a type". The POSITIVE fixture.
;;
;; `process` is a NAMESPACE, not a type, so its members join with `::`. The ruling's whole
;; point is that the FAITHFUL spelling did not change: both entries below name the SAME
;; function, one in each surface, and both must answer 3.
(:wat::core::defn :user::kw-spelled [] -> :wat::core::i64
  (:wat::spawn::ProcessOpts/runner-count (:wat::spawn::process::runner-count 3)))

;; The faithful-Clojure surface. `wat.spawn.process/runner-count` is the image of the
;; keyword name above — byte-identical to what it was BEFORE the respelling, which is why
;; this stone is not a rename.
(:wat::core::defn :user::faithful-spelled [] -> :wat::core::i64
  (wat.spawn.ProcessOpts/runner-count (wat.spawn.process/runner-count 3)))

;; ⛔ A LEGITIMATE `Type/member` JOIN — `ProcessOpts` IS a type, so its accessor keeps the
;; `/` join (255.4: a member join is `/`, always). This stone must not move it.
(:wat::core::defn :user::type-parent-join [] -> :wat::core::i64
  (:wat::spawn::ProcessOpts/max-message-bytes (:wat::spawn::process::max-message-bytes 77)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::core::i64/to-string (:user::kw-spelled))))
