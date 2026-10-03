;; tests/reflection/wat_arc201_holon_ast_accessors_children_parametric.wat
;; Fixture for test bundle_children_walks_parametric_type_slot.
;; Probe: Bundle/children on sig of (Vector :- [i64])-typed fn shows standalone :wat::core::Vector.
(wat.core/defn user/sum-list [init :- wat.type/i64 & xs :- (wat.type/Vector :- [wat.type/i64])] :- wat.type/i64
  (wat.core/foldl
              (wat.core/fn [acc :- wat.type/i64 x :- wat.type/i64] :- wat.type/i64
                (wat.i64/+ acc x))
              init
              xs))

(wat.core/defn user/main [] :- wat.type/nil
  (wat.core/let
              [sig-opt (wat.runtime/signature-of-defn user/sum-list)
               sig     (wat.core/match sig-opt 
                         [wat.core/Option.Some {:value s} s]
                         [wat.core/Option.None {}     (wat.kernel/assertion-failed! :message "signature-of-defn returned None")])
               kids    (wat.core/ast->children sig)
               rendered kids]
              (wat.kernel/println rendered)))
