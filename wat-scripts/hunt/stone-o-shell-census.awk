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
# ⚠ KNOWN EDGE: this reads 380 handlers where the registry holds 381 names. The one it does not
# reach is unexplained and is Stone O-i's row 0 — do not quote 380 as a total without it.

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
