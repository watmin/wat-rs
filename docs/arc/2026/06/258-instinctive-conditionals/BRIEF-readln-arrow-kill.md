# BRIEF — `readln'` `-> :T` kill: the self-describing-wire mirror of recv' (arc 258, the LAST redundant arrow)

> **RULED (builder, 2026-07-22): Option A — the self-describing kill.** readln stops forcing the caller to
> ATTEST what they read; it reads what the self-describing wire tells it, exactly as `recv'`/`select'` do
> (258.5c). After this, `-> :T` is a located compile error EVERYWHERE except a fn/defn argspec return —
> the arc's end-state. STRIKE DRAWN + mechanism PROVEN + RED probe confirmed; NOT yet implemented.

## The ruling + the debate that settled it (four-questions)
readln's `-> :T` is **load-bearing** at runtime (`verbs.rs:418` decodes via `edn_to_typed_value(&target_ty)`,
which NEEDS the target). So it can't just be dropped. The fork was A (infer, self-describing) vs B (re-syntax
`(readln :i64)`, keep the type as an arg) vs C (exempt). The builder's cut: *"so we are forcing the caller to
attest what they are about to read?"* — that IS the crutch. Under wat's own wire discipline (records-are-EDN,
234.7: everything crossing a wire is self-describing/tagged), **stdin is a self-describing wire like any peer**
— there's no honest reason it's exempt. So the value already declares itself; forcing `(readln :i64)` is the
caller re-attesting what `5` already says. Four-questions FLIP on A once the premise is corrected (stdin
self-describes): Obvious YES (read what comes, like recv'), Simple YES (the PROVEN recv' rail, not new),
Honest YES (the type IS in the wire — EDN notation + tags), UX YES (no arrow, no attestation). **A wins;
B is the crutch; C overloads the fn-return glyph.** My earlier "A fails Honest" was the wrong premise
(raw-untyped stdin) — corrected by the builder's probe.

## The scalar contract (builder-ruled 2026-07-22) — ALREADY what decode_trusted_wire does
- **EDN int → `:i64`, EDN float → `:f64`** (grounded: `edn_shim.rs:1558-1559` `Edn::Integer→Value::i64`,
  `Edn::Float→Value::f64`; BigInt/BigDec → an explicit "wat numeric tower is i64+f64 only" error).
- Future rust primitives (`u8`, …) will DECLARE via a type tag on the wire — `#wat.type/u8 42` — *"we'll
  figure that out when we get there."* Not this strike.

## The rail to mirror (GROUNDED, file:line) — this is the recv'/select' kill (258.5b/258.5c), applied to IO
- **Checker** `infer_recv_prime` (`check.rs:11809`): rejects `-> :T` (args.len()>=2 → error), takes ONE arg,
  returns `let t = fresh.fresh(); CheckResult::partial_with(t, …)` — the type flows from the CONSUMER by
  unification; an unconstrained recv' stays a fresh var. **readln mirrors this** (return a fresh var).
- **Runtime** `decode_trusted_wire(types: Option<&TypeEnv>)` (`edn_shim.rs:3076`) — takes NO target type;
  reconstructs the exact `Value` from the EDN's own tags/notation (`read_edn_caps(s, types, true)`).
  This is what `eval_peer_recv_prime` already uses (`runtime.rs:26861` etc.).

## The strike — 5 moves (all mirror the recv' rail)
1. **`infer_kernel_readln_prime`** (`check.rs:9630`): currently REQUIRES exactly 3 args `[cap, ->, :T]`
   (`args.len() != 3`; arrow_idx=1, ty_idx=2) and returns `declared_ty`. Rewrite: **reject a stray `->`**
   with a migration hint (mirror the if/match/apply pattern); accept `(readln' <cap>)` — the cap arg only,
   no `->`/type; **return `fresh.fresh()`** (consumer-unify). Decide the cap arity: today cap is mandatory
   (the macro injects the default), so bare `(readln' <cap>)` = 1 arg. Confirm against the macro (below).
2. **`eval_kernel_readln_prime`** (`src/services/verbs.rs:304`): drop the `->`/`:T` parse (`:346-376`) and
   the `target_ty`; replace the decode `edn_to_typed_value(&target_ty, &edn, sym)` (`:418`) with
   **`decode_trusted_wire(&edn_str?, sym.types()…)`** — SAME as `eval_peer_recv_prime`. (Note the current
   path parses via `wat_edn::parse_owned(&line)` then coerces; decode_trusted_wire takes the raw wire string
   — check whether it wants the String or the parsed Edn and match the recv' call shape.)
3. **The `readln` macro** (`wat/kernel/services/stdin.wat:127`): today `(readln -> :T)` → `(readln' MAX -> :T)`
   and `(readln :max-buffer-bytes N -> :T)` → `(readln' N -> :T)` (it FORWARDS `-> :T`). Drop the forwarding:
   `(readln)` → `(readln' MAX)`, `(readln :max-buffer-bytes N)` → `(readln' N)`. (Model still the arg-handling
   macros; the `-> :T` forwarding was "THE wrinkle" per the max-buffer doc — now it just goes away.)
4. **Corpus strip — 88 sites** (67 `.wat` + 21 `.rs`): the readln `-> :T`. `.wat` via the robust span-based
   codemod (head-swap: `sed 's/":wat::core::match"/":wat::kernel::readln"/' /tmp/strip-match-robust.wat` — but
   NOTE readln's arrow sits AFTER the cap, and there's BOTH `readln` and `readln'` heads; do both heads, and
   verify the codemod deletes `-> :T` wherever the arrow sits, as it did for apply's leading arrow). `.rs`
   inline via the line-scoped sed (`/readln/ s/ -> :[A-Za-z0-9_:<>,]+//`). Also the `readln` MACRO callers vs
   `readln'` prime callers — both.
5. **Tests + the scheme note**: the `check.rs ~17291` readln-scheme (polymorphic-return-from-the-arrow) goes
   away — verify no dead code. Un-ignore/repoint any `readln -> :T` fixtures. Persist the RED probe as
   `wat-tests/core/readln-no-ascription.wat` (a deftest', mirror `match-no-ascription.wat`).

## The RED probe (verified RED at HEAD — exit 1, MalformedForm "requires -> :T")
```clojure
(:wat::core::defn :user::sum [] -> :wat::core::i64
  (:wat::core::let [xs (:wat::kernel::readln)]            ;; BARE — no -> :T
    (:wat::core::foldl
      (:wat::core::fn [a <- :wat::core::i64  b <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::+ a b))
      0 xs)))
```
`xs` is used in a `foldl` over `Vector<i64>` → the consumer constrains `readln` to `Vector<i64>`. GREEN when
readln infers from the consumer. **Negative control:** a `readln` whose result exits to Rust / has NO
constraining consumer → its check-time type is a fresh var with nothing to pin it; PROVE that case first
(same situation recv' faced; it stays a fresh var — confirm it doesn't wrongly error).

## STOP triggers
1. If `decode_trusted_wire` can't reconstruct a corpus target that `edn_to_typed_value` used to coerce (a
   genuinely-underspecified value with no consumer + no tag) — STOP, list the sites (they may need a `#wat.type`
   tag on the input, or a consumer). Do NOT re-introduce a target-type parse.
2. If the `readln` macro can't drop the `-> :T` forwarding cleanly (the max-buffer kwarg path breaks) — STOP.
3. If `readln'`'s cap arity is ambiguous after the arrow drop (is cap mandatory or optional?) — ground it
   against the macro's injection + the corpus; don't guess.

## Gate (weigh by own `--release` re-run)
The RED probe GREEN (bare readln infers `Vector<i64>`); a `readln -> :T` now a migration-hint error;
`every_wat_scripts_file_loads` green for the readln sites (the RecvOutcome red is the SEPARATE recv' sweep,
S3); no NEW failures. This is ONE atomic commit with match/if/apply + the recv' wall + the recv' sweep.
