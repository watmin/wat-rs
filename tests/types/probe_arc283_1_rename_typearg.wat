;; tests/types/probe_arc283_1_rename_typearg.wat — co-located fixture
;;
;; Arc 283.1 — disconfirming probe: rename-keyword-prefix must reach TYPE ARGUMENTS.
;;
;; Arc 109 "annihilate the angle bracket" — the source string below was
;; `:wat::core::Vector<t::Old>`, and the angle form is gone from the language, so
;; the string a runtime `read-string` receives had to move with it. Note what the
;; `:-` spelling does to this probe's PREMISE: in `(:wat::core::Vector :- [:t::Old])`
;; the type argument is an ordinary keyword LEAF, not a name embedded inside another
;; keyword's text — so the rename reaches it by plain start-anchored matching. The
;; boundary-aware embedded rename arc 283.1 built exists to solve a problem the
;; angle bracket created. See NOTE-the-angle-bracket-also-cost-us-a-feature.md.

(:wat::core::defn :user::run [] -> :wat::core::String
  (:wat::fix::rename-keyword-prefix ":t::Old" ":t::New"
    "(:wat::core::defn :u::f [xs <- (:wat::core::Vector :- [:t::Old]) y <- :t::OldExtra] -> :t::Old (:t::Old/make xs))"))
