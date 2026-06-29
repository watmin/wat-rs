;; Arc 293 item-2a — a surface's :holder bound takes the holder-root SYMBOL (:wat::core::Record),
;; NOT the magic shorthand :record. A 0-member :holder surface = "any aggregate of that holder"
;; (the portability shape behind program::Env's user.program — "must be ≥ a record").
;;
;; RED at HEAD: parse_defsurface (surface.rs:322) hand-matches :struct / :record / :holon-record,
;; so `:holder :wat::core::Record` is a MalformedDecl → this world fails to start.
;; GREEN once :holder routes through Holder::from_root_keyword (accepts the holder-root symbol).
(:wat::core::defrecord :env::Rec [host <- :wat::core::String])
(:wat::core::defsurface :env::Portable :holder :wat::core::Record :features [])
(:wat::core::defn :env::take [p <- :env::Portable] -> :wat::core::i64 42)
(:wat::core::defn :env::feed [] -> :wat::core::i64 (:env::take (:env::Rec "h")))
