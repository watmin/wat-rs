;; Arc 255 Stone "the three special-form tables" — the 3 names special_forms.rs still
;; carries (no registration site: defstruct is a stdlib macro / the FOURTH-registry fork;
;; unquote and unquote-splicing are punctuation, not verbs). Confirms `signature-of-defn`
;; still answers for them post-deletion, via `Binding::SpecialForm` (lookup.rs:264-266),
;; a route entirely separate from the deleted `Binding::Registered` arm 247.
(:wat::core::def :user::main
  (:wat::core::fn [] -> :wat::core::nil
    (:wat::kernel::println (:wat::runtime::signature-of-defn :wat::core::defstruct))
    (:wat::kernel::println (:wat::runtime::signature-of-defn :wat::core::unquote))
    (:wat::kernel::println (:wat::runtime::signature-of-defn :wat::core::unquote-splicing))))
