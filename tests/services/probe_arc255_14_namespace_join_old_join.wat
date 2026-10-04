;; Was `probe_arc255_14_namespace_join_old_join.wat.bad`.
;; Arc 255 stone 255.14 refused `:wat::spawn::process/runner-count`.
;; Amend 255.92: that `/` join and `:wat::spawn::process::runner-count` are one
;; name, so this call resolves. A file that freezes is a `.wat` (the `.wat.bad`
;; gate requires a startup failure).
(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::do (:wat::spawn::process/runner-count 3) nil))
