;; tests/rete/probe_arc251_8d_unquote_marker_identity.wat — co-located fixture for the sibling
;; .rs. See that file for the whole argument.
;;
;; The accumulator fence (`wat/rete/compile.wat:579`) builds its four-axis probe with a
;; quasiquote whose ONE hole is an explicitly-spelled `unquote`:
;;
;;     fence-call (:wat::core::quasiquote ((:wat::core::unquote acc-hd) __acc__))
;;
;; Converted, that is `(wat.core/quasiquote ((wat.core/unquote acc-hd) __acc__))`. These entries
;; write BOTH spellings of that exact template by hand, on an unconverted tree.

(:wat::core::defn :user::qq-kw-unquote [] -> :wat::core::String
  (:wat::core::write-forms
    (:wat::core::quasiquote ((:wat::core::unquote :foo::bar) __acc__))))

(:wat::core::defn :user::qq-sym-unquote [] -> :wat::core::String
  (:wat::core::write-forms
    (:wat::core::quasiquote ((wat.core/unquote :foo::bar) __acc__))))

(:wat::core::defn :user::qq-kw-splice [] -> :wat::core::String
  (:wat::core::write-forms
    (:wat::core::quasiquote (:head (:wat::core::unquote-splicing [1 2])))))

(:wat::core::defn :user::qq-sym-splice [] -> :wat::core::String
  (:wat::core::write-forms
    (:wat::core::quasiquote (:head (wat.core/unquote-splicing [1 2])))))

;; ⛔ THE NON-VACUITY OF THE TWO ABOVE — a form that is NOT one of the markers must stay
;; VERBATIM inside the template, in either spelling. If the cure had taught the walker to fire
;; on "anything symbol-headed", these two would come back evaluated.
(:wat::core::defn :user::qq-kw-not-a-marker [] -> :wat::core::String
  (:wat::core::write-forms
    (:wat::core::quasiquote ((:wat::core::unquotex :foo::bar) __acc__))))

(:wat::core::defn :user::qq-sym-not-a-marker [] -> :wat::core::String
  (:wat::core::write-forms
    (:wat::core::quasiquote ((wat.core/unquotex :foo::bar) __acc__))))

;; ── THE PROPERTY ROWS ────────────────────────────────────────────────────────────────────────
;;
;; ⚠ The four renderings above are returned as STRINGS so the `.rs` can compare the two
;; spellings to EACH OTHER — but the `.rs` may not hold the expected text as a literal
;; (`no_inlined_wat_in_tests` / `no_inlined_edn`: a rendered wat form in a Rust string literal is
;; inlined wat, and the lint's own rubric says restructure the CODE rather than rune it). So the
;; ABSOLUTE claim — "the marker FIRED" / "the near-miss did NOT" — is computed here, in wat,
;; where the form is a form and not a string in someone else's language.
(:wat::core::defn :user::kw-unquote-fired [] -> :wat::core::bool
  (:wat::core::not (:wat::string::contains? (:user::qq-kw-unquote) "unquote")))
(:wat::core::defn :user::sym-unquote-fired [] -> :wat::core::bool
  (:wat::core::not (:wat::string::contains? (:user::qq-sym-unquote) "unquote")))
(:wat::core::defn :user::kw-splice-fired [] -> :wat::core::bool
  (:wat::core::not (:wat::string::contains? (:user::qq-kw-splice) "unquote")))
(:wat::core::defn :user::sym-splice-fired [] -> :wat::core::bool
  (:wat::core::not (:wat::string::contains? (:user::qq-sym-splice) "unquote")))
(:wat::core::defn :user::kw-not-a-marker-stayed-data [] -> :wat::core::bool
  (:wat::string::contains? (:user::qq-kw-not-a-marker) "unquotex"))
(:wat::core::defn :user::sym-not-a-marker-stayed-data [] -> :wat::core::bool
  (:wat::string::contains? (:user::qq-sym-not-a-marker) "unquotex"))
