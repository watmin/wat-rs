;; 255.8 — a namespace that is also a type. One file, both spellings, and a
;; real Type/method so the member join is not undone.
(wat.core/defrecord my/Journal [k :- wat.core/i64])
(wat.core/defrecord my.Journal/Req [k :- wat.core/i64])
(wat.core/defn user/clj [req :- my.Journal/Req] :- my.Journal/Req
  req)
(:wat::core::defn :user::colon [req <- :my::Journal::Req] -> :my::Journal::Req
  req)
(wat.core/defn user/member [] :- wat.core/i64
  (wat.core.Option/expect (:wat::core::Option.Some {:value 7}) "missing"))
