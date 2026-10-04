;; Was `probe_arc251_8d_macro_member_join_wrong_join.wat.bad`.
;;
;; 255.8's direction control refused `:user::helper::of` against a defmacro
;; stored as `:user::helper/of`. Amend 255.92 (R-a, 2026-10-02; the builder's
;; pair ruling 2026-10-04): the two joins are one name, `{user.helper, of}`.
;; Both spellings reach that one macro. The parent stays lower-case so this
;; is not the retired PascalCase Type::member spelling `one_member_join` refuses.
(:wat::core::defmacro :user::helper/of [n <- wat.type/AST] -> wat.type/AST
  `(:wat::core::+ ~n 1))
(:wat::core::defn :user::slash [] -> wat.type/i64
  (:user::helper/of 7))
(:wat::core::defn :user::colon [] -> wat.type/i64
  (:user::helper::of 7))
