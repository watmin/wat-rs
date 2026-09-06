;; POSITIVE control — Alpha into an Alpha field must still compile. A check that
;; rejects correct work is worse than the hole.
(:wat::core::defenum :fit::Alpha :wat::enum::Pure :A [])
(:wat::core::defrecord :fit::Box [k <- :fit::Alpha])
(:wat::core::defrecord :fit::Src [k <- :fit::Alpha])

(:wat::rete::defrule :fit::same
  :when [(:fit::Src (?k <- :k))]
  :then [(:fit::Box :k ?k)])
