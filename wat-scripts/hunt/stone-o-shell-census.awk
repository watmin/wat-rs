# stone-o-shell-census.awk — arc 255 Stone O.
#
#   find src -name '*.rs' -print0 | xargs -0 awk -f wat-scripts/hunt/stone-o-shell-census.awk
#
# Classifies every `#[wat_intrinsic]`-annotated handler as SHELL or BINDING, and records
# whether it already carries a `value = <path>` slot (arc 255 Stone N). Output is TSV:
#   <SHELL|BINDING>\t<HASVAL|->\t<file>\t<fn name>
#
# SHELL := after deleting the argument-eval calls `eval_inner(<expr>, env, sym)` and all line
#          comments, the BODY names neither `env` nor `sym`. Such a handler is
#          (eval each arg) -> value-fn, so ONE declaration can generate BOTH doors.
# BINDING := everything else. It needs the environment, so it has no value-level twin and
#          `:wat::core::apply` genuinely cannot splat into it.
#
# ⚠ THE SIGNATURE IS EXCLUDED ON PURPOSE. Every handler's signature names `env` and `sym` —
# the generated shim forces that shape — so a classifier that reads the signature calls
# EVERYTHING binding. The first version of this instrument did exactly that and reported
# 14 shells where there are 137. Controlled both ways before its numbers were quoted:
#   POSITIVE  eval_persistentvector_length_home  must classify SHELL   (hand-read: eval_inner + _inner)
#   NEGATIVE  eval_seq_zip_intrinsic             must classify BINDING (passes env/sym to a helper)
# `[[feedback_validate_a_search_pattern_before_trusting_its_count]]`
#
# ⛔⛔ KNOWN BLIND SPOT — SHELL DOES NOT MEAN MIGRATABLE. Added 2026-08-28 after Stone O-iv-c-1.
# This script asks ONE question: does the body name `env` or `sym` after the arg-eval is stripped?
# That is not the whole of "can this become ALGEBRA", because there is a THIRD binding-only
# capability it cannot see: **`<arg>.span()`** — a `&WatAST` parameter's own source location.
# `Value`, the ALGEBRA parameter type, carries NO span, so a handler that builds
# `RuntimeError::new(<arg>.span().clone(), …)` cannot reproduce that location after migration.
# The best substitute is the call-list span (Stone Q), which is a DIFFERENT location.
#
# Measured cost of not knowing this: O-iv-c-1's brief said 32 SHELL verbs were migratable. FIVE
# of them read an argument's span — reckoner's new-discrete/new-continuous/resolve, hologram's
# put/remove — and the rider refused them under STOP-3 rather than silently downgrade a
# diagnostic. 27 migrated, not 32; 8 stayed BINDING, not 3.
#
# SO: treat SHELL as a CANDIDATE LIST. Before migrating, also check:
#     grep -nE '[a-z_]+\.span\(\)' <the handler's body>      -> ARG-SPAN, stays BINDING
#     grep -n 'require_encoding_ctx'  <the handler's body>     -> takes &SymbolTable, stays BINDING
# ⚠ Do NOT extend this script to do that check by pattern. Three span classifiers were written
# for this exact question in one afternoon and all three were retracted, the last after failing a
# control the orchestrator wrote himself. THE COMPILER AND A READ ARE THE INSTRUMENTS.
# `[[feedback_a_pass_answers_only_the_question_the_instrument_asks]]`
#
# ⛔ 380 IS THE TOTAL, AND THIS INSTRUMENT WAS RIGHT — the "381" it was once measured against
# was the WRONG number, corrected 2026-08-28 by the Stone O-iii rider. That baseline came from a
# text grep run over whole files:
#     grep -oP '#\[wat_intrinsic\(\s*"\K[^"]+' | grep -v '<fqdn>'
# which counted a DOC COMMENT as a registration. `src/intrinsic/holon/mod.rs:9` reads:
#     //! `#[wat_intrinsic(":wat::holon::…")]` handlers under the SAME names, here.
# Prose about a migration, using `…` as a placeholder — and the `-v '<fqdn>'` filter knew only
# ONE spelling of "this is a placeholder" and let the other through. Counting comments as code,
# for the third time in one day. `[[feedback_a_file_count_is_not_an_item_count]]`
#
# The honest one-liner ANCHORS to attribute position instead of matching text anywhere in a file:
#     grep -rhoP '^\s*#\[wat_intrinsic\(\s*"\K[^"]+' src/ --include=*.rs | sort -u | wc -l
# It returns 380, and its list is IDENTICAL to this awk's — verified name-for-name, both ways.

/^#\[wat_intrinsic\(/ { pend=1; hasval = ($0 ~ /value[ ]*=/) ? 1 : 0; next }
pend && /^#\[/ { next }                                    # stacked attributes
pend && /fn [a-z_0-9]+\(/ {
    line=$0; sub(/^.*fn /,"",line); sub(/\(.*/,"",line); name=line
    insig=1; pend=0; body=""
    if ($0 ~ /\{[ ]*$/ && $0 ~ /->/) { insig=0; inbody=1 } # one-line signature
    next
}
insig { if ($0 ~ /\{[ ]*$/) { insig=0; inbody=1 } next }   # skip to the end of the signature
inbody {
    if ($0 == "}") {                                       # top-level fn; rustfmt closes at col 0
        b = body
        gsub(/\/\/[^\n]*/, "", b)                          # comments first: prose names env/sym
        gsub(/eval_inner\([^;]*, *env, *sym\)/, "", b)      # the argument-eval call
        kind = (b ~ /(^|[^A-Za-z_0-9])(env|sym)([^A-Za-z_0-9]|$)/) ? "BINDING" : "SHELL"
        printf "%s\t%s\t%s\t%s\n", kind, (hasval?"HASVAL":"-"), FILENAME, name
        inbody=0; body=""
        next
    }
    body = body "\n" $0
}
