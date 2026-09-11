;; Arc 255 / 296 — the twin of `__dot_spelling_is_the_constructor.wat`, and the half that makes the
;; pair a discrimination rather than a single green.
;;
;; `Option::Some` was the canonical constructor spelling until the dot flip. It is now the one that
;; does NOT resolve — refused at RESOLVE, before the keyword-accessor fall-through is consulted, so
;; `main` never runs and nothing reaches stdout. That is the exact inversion of what this probe's
;; parent fixture used to assert, and asserting only the dot half would leave the flip unproven:
;; a test that says "the new spelling works" passes just as well in a world where BOTH spellings do.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::core::Option::Some {:value 7})))
