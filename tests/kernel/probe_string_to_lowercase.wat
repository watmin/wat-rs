;; Co-located fixture for probe_string_to_lowercase.rs — slurped via startup_beside(file!()).

(wat.core/defn user/lower [s :- wat.type/String] :- wat.type/String
  (wat.string/to-lowercase s))

