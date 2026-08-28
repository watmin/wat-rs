# stone-o-delegate-census.awk — arc 255 Stone O. Companion to stone-o-shell-census.awk.
#
#   find src -name "*.rs" -print0 | xargs -0 awk -f wat-scripts/hunt/stone-o-delegate-census.awk
#
# The SHELL census is a LOWER BOUND: it calls a handler BINDING whenever env/sym reach a helper,
# even a helper that only evaluates arguments. This finds that second population — 187 handlers
# whose whole body hands (args…, env, sym) to ONE helper and returns it.
#
# ⚠ CANDIDATES, NOT PROVEN ALGEBRA. A member is algebra only if its HELPER is; the test reads body
# SHAPE, not behaviour. Read each helper before migrating. Two known: `eval_f64_max_of` delegates to
# `f64_variadic_reduce`, which is pure fold + eval_inner (algebra, but SPAN-CARRYING — see the
# design section "THE THIRD CATEGORY"); `eval_iowriter_new` passes env/sym while taking NO wat args.
# `[[feedback_a_census_without_attribution_is_not_a_census]]`
# A BINDING handler whose body does nothing but hand (args…, env, sym) to ONE helper is
# not binding — it has DELEGATED its arg-evaluation one level down. Same class as a SHELL,
# one indirection away.
/^#\[wat_intrinsic\(/ { pend=1; next }
pend && /^#\[/ { next }
pend && /fn [a-z_0-9]+\(/ { line=$0; sub(/^.*fn /,"",line); sub(/\(.*/,"",line); name=line; insig=1; pend=0; body=""
    if ($0 ~ /\{[ ]*$/ && $0 ~ /->/) { insig=0; inbody=1 } next }
insig { if ($0 ~ /\{[ ]*$/) { insig=0; inbody=1 } next }
inbody {
    if ($0 == "}") {
        b=body; gsub(/\/\/[^\n]*/,"",b); gsub(/\n/," ",b); gsub(/  +/," ",b); gsub(/^ | $/,"",b)
        # strip a leading `const OP: &str = "...";`
        sub(/^const OP: &str = "[^"]*"; /,"",b)
        if (b ~ /^[a-z_0-9:]+\([^;]*env, *sym[^;]*\)$/ && b !~ /;/) print name "\t" FILENAME
        inbody=0; body=""; next
    }
    body = body "\n" $0
}
