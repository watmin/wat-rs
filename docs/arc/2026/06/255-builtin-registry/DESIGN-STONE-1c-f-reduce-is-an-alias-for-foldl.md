# DESIGN — STONE 1c-f: `reduce` becomes what it already is

> **Builder, 2026-09-03:**
>
> *"sift is making illegal calls - that's the heretic - **reduce being named how it is... that's
> heresy... it is an alias to foldl** .... and foldl's name will be taken away.... reduce will take
> its spot....*
>
> *but not today..... **just make reduce a foldl alias**.... and then get the = and not= handled.....
> they are partial.... this has been known... for quite a while...."*

Governed by `[[RULING-the-registry-is-the-sole-authority]]`. This stone is a **step-3** move
(*the duplicate dies*) that turned out to require a **step-1** move first — see the probe.

## The heresy, stated exactly

`:wat::core::reduce` is a `defclause` (`wat/seq.wat:318`) whose 3-arity arm is the bare body
`(:wat::core::foldl f init coll)`. It is `foldl` wearing a second name — and that second name is
what `src/rete/purity.rs:652`'s placeholder hardcodes `total: true` for, without deriving it.

Its 2-arity arm is the only thing that is not `foldl`: it seeds from the first element and
**raises** `assertion-failed!` on an empty collection (`wat/seq.wat:328`). That raise is what makes
the hardcoded `total: true` a lie today.

## ★★★ THE PROBE — run before this design was written, and it REFUTED the obvious shape

Method: swap the `defclause` for `(:wat::core::defalias :wat::core::reduce :wat::core::foldl)`,
`cargo build --release --bin wat` (the stdlib is `include_str!`ed — `src/load/stdlib.rs:68` — so an
edit to `wat/seq.wat` is invisible to a stale binary), `--check` every call site, then **revert**.

⛔ **The first run of this probe was MIS-AIMED and every result was a false green.** It ran against
the stale binary; a deliberately undefined verb appended to `wat/seq.wat` also produced `exit=0`,
which is what exposed it. The sabotage canary is now part of the method:
`(:wat::core::reduce f coll)` — 2 args — **must** come back RED under a 3-arity alias, or the probe
is blind.

### Result 1 — a plain `defalias` breaks 4 of 7 files

```
:wat::core::reduce: parameter #3 expects (:wat::core::Vector :- [:wat::core::i64]);
                    got (:wat::stream::Stream :- [:wat::core::i64])
                    got (:wat::core::PersistentVector :- [:wat::core::i64])
```

**The corpus is not the blocker. `foldl`'s own registered scheme is.** `defalias` derives its
signature from `CheckEnv`'s retained `TypeScheme` (`src/declare/register.rs:2085`, Case 2), and that
scheme's third param is still `vec_of(T)` — pre-118.B6, before `foldl` was widened to walk any
seqable. Direct `(foldl …)` calls never see it: they are intercepted by `infer_foldl`
(`src/check.rs:2394`, a keyword-head arm). **An alias is the first consumer that reads the stale
copy** — which is precisely the RULING's shape, found by a probe rather than a census.

### Result 2 — widening the scheme clears every site but the intended one

Replacing that `vec_of(t_var())` with `(:wat::core::Seqable :- [T])` and rebuilding:

```
probe_arc278_0d_transform_dispatch_parity.wat   OK      (was RED ×6)
probe_arc278_0c_persistent_parity.wat           OK      (was RED)
probe-118B-dorun-retention-slope.wat            OK      (was RED)
census-three-call-stream-walks.wat              OK
bench-118B7-reduce-collapse.wat                 OK
census-defclause-arm-overlap.wat                OK
probe-118B2-rider-verification.wat              RED  ← ":wat::core::reduce: expected 3 argument(s); got 2"
```

★ **And it refutes a blocker note that has been sitting at that exact site.** `src/check.rs:20464`
says *"a static TypeScheme cannot express 'any Seqable'"*, and the three verbs cited around it
(`zip`/`window`/`remove-at`) were given custom `infer_*` arms on that basis. Measured today: it
**can** — `TypeExpr::Parametric { head: "wat::core::Seqable", args: [T] }` registers and unifies.
`[[feedback_a_blocker_note_is_a_claim_with_a_date_on_it]]`. This stone corrects the note at its
site; it does **not** revisit `zip`/`window`/`remove-at` (out of scope, affirmatively — they work,
and nothing in this stone touches them).

## The measured call-site census

Instrument (re-derivable — it is written down because the first one I wrote was WRONG, returning
impossible "7-arity" and "8-arity" rows before a corrected tokenizer and an eyeball agreed):

The instrument itself, so the number outlives the session that produced it
(`[[feedback_an_instrument_must_outlive_the_number_it_produced]]`) — save as `arity.pl`,
then `perl arity.pl $(grep -rl ":wat::core::reduce" wat/ wat-scripts/ tests/ | grep -v '^wat/seq.wat$')`:

```perl
use strict; use warnings;
my %t;
for my $f (@ARGV) {
  open my $fh,'<',$f or next; local $/; my $s=<$fh>; close $fh;
  $s =~ s/;;[^\n]*/ /g;                        # strip line comments
  $s =~ s/"(?:\\.|[^"\\])*"/STR/g;             # neutralize strings
  while ($s =~ /\(\s*:wat::core::reduce(?![\w:!?*\/><=+-])/g) {
    my $i = pos($s); my $depth = 1; my $args = 0;
    while ($i < length($s) && $depth > 0) {
      my $c = substr($s,$i,1);
      if    ($c eq '(') { $args++ if $depth==1; $depth++; $i++ }
      elsif ($c eq ')') { $depth--; $i++ }
      elsif ($c eq '[') { $args++ if $depth==1; $depth++; $i++ }
      elsif ($c eq ']') { $depth--; $i++ }
      elsif ($c =~ /\s/) { $i++ }
      else { # bare atom
        my ($tok) = substr($s,$i) =~ /^([^\s()\[\]]+)/;
        $args++ if $depth==1; $i += length($tok);
      }
    }
    $t{"$args-arity"}++; push @{$t{"sites-$args"}}, $f;
  }
}
for my $k (grep {/-arity$/} sort keys %t) {
  (my $n = $k) =~ s/-arity//;
  print "$k : $t{$k}\n";
  my %seen; print "    $_\n" for grep {!$seen{$_}++} @{$t{"sites-$n"}};
}
```

```
3-arity : 19 calls across 8 files      → survive, once the scheme is widened
2-arity :  1 call,  1 file             → wat-scripts/scratch-pad/probe-118B2-rider-verification.wat:67
```

**Exactly one caller of the arm this stone retires**, and it is a scratch-pad probe whose own
comment (`:59`) says it exists to exercise *"reduce — 3-arity and 2-arity Stream arms"*.
⛔ It is **augmented, not deleted** — builder, 2026-09-03: *"deletions must clear a high bar... we
augment as they need."*

## THE FOUR QUESTIONS — flat YES/NO

| | Obvious? | Simple? | Honest? | Good UX? |
|---|:---:|:---:|:---:|:---:|
| **the change** | YES | YES | YES | YES |

- **Obvious? YES** — the 3-arity body *is* `(foldl f init coll)`. A reader sees the alias and knows
  everything.
- **Simple? YES** — one form replaces twelve lines; one stale scheme param is corrected. No new
  entity, no new mechanism.
- **Honest? YES** — this is the whole point. Two names for one verb is the duplication the RULING
  exists to kill, and the 2-arity raise is what makes today's `total: true` false. After this stone
  `reduce` has exactly `foldl`'s properties because it **is** `foldl`.
- **Good UX? YES** — the one behavioural loss (2-arity, seed-from-first) fails at **check** time
  with a located `expected 3 argument(s); got 2`, not at runtime.

## Scope

**In:** `foldl`'s retained scheme widened to Seqable · the `defclause` → `defalias` · the one
2-arity caller augmented · the two now-shadowed `reduce` arms in `src/rete/purity.rs` measured and
removed if dead.

**Out of arc 255 Stone 1c-f's scope, affirmatively:**
- **The name swap** (`foldl`'s name retired, `reduce` taking its spot). Builder: *"but not today."*
  Not tracked as a deferral here — it is a **declared future arc** the builder named in the same
  breath, and this stone is its prerequisite, not its down-payment.
- **`:wat::core::=` / `:wat::core::not=`.** Their own stone, next, per the same ruling.
- **`zip`/`window`/`remove-at`'s custom infer arms.** They are correct; the note that justified them
  is what this stone corrects, and re-shaping working code on that basis is not this stone's job.

## What the placeholder does after this stone

`head_ok`'s door order (`src/rete/purity.rs:894`) is: `constructor_meta` → `accessor_meta` →
**`sym.has_function(head)` → `classify_fn`** → rete-vocabulary → runtime-closure → deny.
`register_defalias` registers its synthesized delegating `Function` into `sym.functions`
(`src/declare/register.rs:2071`), so door 3 fires for `:wat::core::reduce` and the two hand-list
arms below it may become **unreachable**:

```
src/rete/purity.rs:557   ":wat::core::reduce" in the pure_det list
src/rete/purity.rs:652   ":wat::core::reduce" in the matches! totality placeholder
```

⚠ **That is a hypothesis, not a finding.** The rider MEASURES it (delete-and-floor), and reports
the answer either way. If they are dead, they go — and the placeholder drops from three names to
two, leaving only `=`/`not=` for the next stone. If they are live, they **stay**, and the report
says why. This stone does not need them dead to be correct.
