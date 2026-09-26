;; Arc 293 item-2a — a surface's :nature bound takes the nature-root SYMBOL (:wat::core::Record),
;; NOT the magic shorthand :record. :env::Rec joins the featureless :env::Portable by extend-type
;; (stone 255.48: an empty member list is not ambient satisfaction).
;;
;; RED at HEAD: parse_defsurface (surface.rs:322) hand-matches :struct / :record / :holon-record,
;; so `:nature :wat::core::Record` is a MalformedDecl → this world fails to start.
;; GREEN once :nature routes through Nature::from_root_keyword (accepts the nature-root symbol).
(:wat::core::defrecord :env::Rec [host <- :wat::core::String])
(:wat::core::defsurface :env::Portable :nature :wat::core::Record :features [])
(:wat::core::extend-type :env::Rec :env::Portable)
(:wat::core::defn :env::take [p <- :env::Portable] -> :wat::core::i64 42)
(:wat::core::defn :env::feed [] -> :wat::core::i64 (:env::take (:env::Rec :host "h")))
