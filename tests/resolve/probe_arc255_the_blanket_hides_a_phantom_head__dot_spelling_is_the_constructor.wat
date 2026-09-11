;; Arc 255 / 296 — ⛔ THIS FIXTURE'S PREMISE WAS INVERTED BY THE DOT FLIP. Rewritten 2026-09-10.
;;
;; It was `__dot_spelling_reaches_the_accessor.wat`, and that name was a CLAIM that is now false.
;; Its whole point was that `Option.Some` — spelled with a DOT — was a PHANTOM head: it did not
;; resolve as the `Option::Some` constructor, so it fell through to the keyword-as-accessor path
;; (total: a miss is `None`, never an error) and silently printed `#wat.core/Option.None {}` for
;; what looked like a real constructor. The `:wat::*` blanket is what let it reach that door.
;;
;; ★ THE DOT FLIP MADE THAT SPELLING CANONICAL. `Option.Some` IS the constructor now; `Option::Some`
;; is the spelling that no longer resolves (see `__colon_spelling_is_refused.wat`, its twin). The
;; asymmetry this file was built to expose is gone in the only way that retires it honestly: the
;; phantom became the real name.
;;
;; ⛔ ONE LINE DID NOT SURVIVE, and it is worth saying why rather than deleting it quietly. The old
;; third line was `(:wat::core::Option.Some {:wat::core::Option.Some 42})` — a map whose KEY was a
;; ctor name, there to prove the accessor finds a key when the key is present. Post-flip that head
;; resolves, so its `{}` is a ctor PAYLOAD, not a map literal, and a payload key must be a bare
;; field keyword — it is now `MalformedForm`, not a clever lookup. The property it tested is proven
;; by the last line here instead, which needs no phantom to make its point.
(:wat::core::defn :user::main [] -> :wat::core::nil
  ;; ⛔ ONE println, not three. stdout must be a SINGLE EDN value so the assertion can be a
  ;; structural golden (`tests/lint/no_inlined_edn.rs` bans an inline EDN string literal, and
  ;; three values on three lines cannot be one golden). The vector carries all three reads:
  ;;   [0] the DOT spelling IS the constructor — it RESOLVES, and the map is its payload
  ;;   [1] the bare keyword accessor, untouched by the flip and still TOTAL: a miss is None
  ;;   [2] ...and it still finds the key when the key is actually there
  (:wat::kernel::println
    [(:wat::core::Option.Some {:value 7})
     (:anything-at-all {:value 7})
     (:value {:value 7})]))
