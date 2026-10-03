(wat.core/do
  (wat.core/def my/helper (wat.core/fn [] :- wat.type/i64 42))
  (wat.core/def my/main (wat.core/fn [] :- wat.type/i64 (my/helper))))
