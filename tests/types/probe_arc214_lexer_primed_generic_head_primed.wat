;; Fixture: PRIMED two-param generic head — must pass the LEXER (CommaInKeywordBody
;; must NOT fire). Twin of the control; the apostrophe is the ONLY variable.
(wat.core/defn user/take [m :- (wat.core/HashMap' :- [wat.type/String wat.type/String])] :- wat.type/nil nil)
