;; ─── excursus 001 item 3, `an-outcome-arm-cannot-be-a-wildcard` — THE DRIVEN CONTROL ──────────
;;
;; Phase 1 of that stone is a CENSUS, and its scope predicate has two arms that the corpus does
;; not currently exercise:
;;
;;   ⚠ the `Status` arm. `defservice`'s generated `Status` has a FIXED six-variant shape but a
;;     PER-SERVICE path (`:<svc-ns>::Status`), so the rule reaches it by SHAPE, not by name. Item 1
;;     (`every-status-arm-is-named`, `4413fff0f`) drove the corpus to ZERO `Status` wildcards, so
;;     that arm of the predicate has nothing live to fire on — it would be an untested claim in a
;;     report. `:probe::census::Status` below is a hand-written enum with EXACTLY that shape and a
;;     wildcarded match, so the arm is measured end-to-end through the real checker instead of
;;     asserted from the unit test alone.
;;
;;   ⛔ the NEGATIVE arm (DESIGN trap-doors 1 and 2). A wildcard over a DOMAIN enum is ordinary
;;     code and must NOT be flagged. `:probe::census::Color` is that control, and
;;     `:probe::census::Statusish` is the sharper one: it is *named* `Status`'s last segment but
;;     carries a different variant set, so it proves the predicate keys on the SHAPE and not on the
;;     spelling. `:wat::core::Option` gets a wildcard too — `Option`/`Result` are their own
;;     `MatchShape`s, never `MatchShape::Enum`, so they cannot reach the census hook at all.
;;
;; ⛔ EVERY MATCH IN THIS FILE IS DELIBERATELY WILDCARDED. That is the point — this is a fixture
;; for a detector, not exemplary code. Do not "fix" the arms; the census test
;; (`tests/lint/outcome_wildcard_census.rs`) asserts on exactly this shape.

;; ── the FIXED six-variant `Status` shape, at a path the rule cannot have listed ────────────────
(:wat::core::defenum :probe::census::Status :wat::enum::Pure
  :Started      [addr     <- :wat::core::String]
  :Stopped      [resp     <- :wat::core::String]
  :Hibernated   [snapshot <- :wat::core::String]
  :PeersAllowed
  :PeersDenied
  :Faulted      [cause    <- :wat::core::String])

;; ── the negative controls ─────────────────────────────────────────────────────────────────────
;; A plain domain enum: a `_` here is ordinary code (trap-door 1).
(:wat::core::defenum :probe::census::Color :wat::enum::Pure
  :Red
  :Green
  :Blue)

;; Last segment `Status`, WRONG shape — must be declined, or the rule fires on a name.
(:wat::core::defenum :probe::census::Statusish::Status :wat::enum::Pure
  :Up
  :Down)

(:wat::core::defn :probe::census::a-status [] -> :probe::census::Status
  (:probe::census::Status::PeersAllowed))

(:wat::core::defn :probe::census::a-color [] -> :probe::census::Color
  (:probe::census::Color::Red))

(:wat::core::defn :probe::census::a-statusish [] -> :probe::census::Statusish::Status
  (:probe::census::Statusish::Status::Up))

;; IN SCOPE, by shape: one named arm + a `_` swallowing the other five.
(:wat::core::defn :probe::census::read-status [] -> :wat::core::String
  (:wat::core::match (:probe::census::a-status)
    (:probe::census::Status::PeersAllowed "allowed")
    (_ "swallowed")))

;; OUT: a domain enum.
(:wat::core::defn :probe::census::read-color [] -> :wat::core::String
  (:wat::core::match (:probe::census::a-color)
    (:probe::census::Color::Red "red")
    (_ "swallowed")))

;; OUT: named Status, wrong shape.
(:wat::core::defn :probe::census::read-statusish [] -> :wat::core::String
  (:wat::core::match (:probe::census::a-statusish)
    (:probe::census::Statusish::Status::Up "up")
    (_ "swallowed")))

;; OUT: `Option` is its own MatchShape and never reaches the hook (trap-door 2).
(:wat::core::defn :probe::census::read-option [] -> :wat::core::String
  (:wat::core::match (:wat::core::Some "x")
    ((:wat::core::Some s) s)
    (_ "none")))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::string::concat
      (:probe::census::read-status)
      (:wat::string::concat
        (:probe::census::read-color)
        (:wat::string::concat
          (:probe::census::read-statusish)
          (:probe::census::read-option))))))
