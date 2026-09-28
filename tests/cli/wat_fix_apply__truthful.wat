;; tests/cli/wat_fix_apply__truthful.wat — arc 282 POSITIVE control, byte-for-byte the liar
;; with ONE difference: the claim is TRUE. Without this pair the negative control proves only
;; that something raised, never that a correct edit still lands.
(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
    [src   "hello world"
     edits (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])]
             (:wat::core::Tuple 6 "world" "there"))]
    (:wat::kernel::println (:wat::fix::fix-text-apply src edits))))
