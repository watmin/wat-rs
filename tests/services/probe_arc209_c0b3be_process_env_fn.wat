(wat.core/defrecord app/Env [token :- wat.type/i64])
(wat.core/defn app/make-env [] :- wat.type/Record (app/Env :token 7))
