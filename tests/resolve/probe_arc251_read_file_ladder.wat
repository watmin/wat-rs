(wat.core/defn user/run [path :- wat.type/String c :- wat.type/String] :- wat.type/String
  (wat.core/do
    (wat.io/write-file path c)
    (wat.io/read-file path)))
(wat.core/defn user/run2 [path :- wat.type/String] :- wat.type/String
  (wat.io.IOReader/read-all-string (wat.io.IOReader/open-file path)))
