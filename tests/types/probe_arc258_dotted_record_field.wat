;; tests/types/probe_arc258_dotted_record_field.wat — co-located fixture
;;
;; A2 de-risk probe — does a recordtype tolerate a DOTTED field name?
;; `wat.started-at` is a dotted field; its accessor is `:<class>/wat.started-at`.

(:wat::core::defrecord :user::Probe [wat.started-at <- :wat::time::Instant])
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::time::epoch-millis
    (:user::Probe/wat.started-at (:user::Probe (:wat::time::at-millis 1234)))))
