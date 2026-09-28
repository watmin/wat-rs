;; arc 255 Stone 255.21 (C-b1b) — MEASUREMENT. Expands a dialing kwargs `defn` (two Peer
;; fields and one data field) at the FORM level and prints the generated companions, so what the
;; kwargs macro emits for `::GrantHandles` / `::kwargs-check` / `::grant-worker` /
;; `::revoke-worker` / `::Coords` is READ, not argued. After 255.21 each service field `i` carries
;; its own declared transport param `T<i>`: `(TypedCapability :- [S R T<i>])`.
(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::kernel::println
    (:wat::core::write-forms
      (:wat::core::macroexpand
        (:wat::core::quote
          (:wat::core::defn :probe::work
            [item <- wat.type/String
             & [kv    <- (:wat::kernel::Peer :- [:probe::Kv::Op :probe::Kv::Reply])
                times <- wat.type/i64
                echo  <- (:wat::kernel::Peer :- [:probe::Echo::Op :probe::Echo::Reply])]]
            -> wat.type/String
            item))))))
