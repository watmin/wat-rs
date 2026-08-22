;; tests/resolve/arc109_type_reference_must_resolve_row5_field.wat — row 5.
;;
;; A phantom type in a `defrecord` FIELD. `register_types_impl` consumes this form entirely
;; at freeze step 5 — it never reaches step 7's residue walk, which is why the sweep must
;; read the TypeEnv registry rather than re-walk forms.
(:wat::core::defrecord :user::R
  [n <- :user::NoSuchType])
