# BRIEF — 198: a restriction governs MENTION, not head position

> Read `DESIGN-STONE-a-restriction-governs-mention-not-head-position.md` first. **Security stone —
> builder ruling: this takes precedence.** Baseline HEAD `60d0e77a`, tree clean, floor
> **4531 run / 4531 passed / 154 skipped**, clippy 0.

## THE WORK IN ONE PARAGRAPH

`walk_for_restricted_call` (`src/check.rs:1403`) enforces `:restricted-to` only when the restricted
FQDN is the **head of a List**. A restricted FQDN in any other position is a bare `WatAST::Keyword`,
which the walk passes over in silence — so a one-line `let` alias defeats every restriction in the
substrate. Make the walker check **every `WatAST::Keyword` node** instead of only `items.first()`.

## THE CHANGE

Current — the `List`/`first()` guard is the bug:

```rust
if let WatAST::List(items, _) = node {
    if let Some(WatAST::Keyword(head, head_span)) = items.first() {
        if let Some(meta) = env.get_binding_metadata(head) {
            if let Some(prefixes) = extract_prefix_list_from_metadata(meta) {
                if !caller_matches_prefix_list(enclosing_fn, &prefixes) { …push error… }
```

Target — **delete** the outer two `if let`s; check the node itself:

```rust
if let WatAST::Keyword(name, name_span) = node {
    if let Some(meta) = env.get_binding_metadata(name) { …same body, same error, same span source… }
}
```

The `for child in node.children()` recursion below stays exactly as it is. Same registry, same
`DefRestrictedCallerNotAllowed` variant, no new storage, nothing added to any builtin. **The code gets
smaller.**

## ⛔ QUESTIONS ALREADY SETTLED BY MEASUREMENT — DO NOT RE-OPEN

- **No declaration-site exemption is needed.** `src/check.rs:651` calls
  `walk_for_restricted_call(body, name, &env, &mut errors)` — it walks **function BODIES only**, per
  registered function, never declaration forms. The orchestrator initially flagged an exemption as a
  builder ruling and was wrong; one grep settled it.
- **Self- and mutual recursion pass naturally.** `str-double`'s body calls itself with enclosing fn
  `:wat::kernel::str-double`, which matches `[:wat::kernel::]`. `flood-stdout-raw` → `write-fd-raw`
  likewise. **Confirm both still pass** — but do not build machinery for them.
- **Registration is correct.** `src/runtime.rs:1453` keys the ctor whitelist on the type name. Do not
  touch it.

## THE GATE — three proofs, and the first must FAIL before you fix

1. **The value-position escape closes.** New fixture + test: a `:user::` fn that binds a restricted
   FQDN in value position and calls the local **must be refused**.
   ```clojure
   (:wat::core::defn :user::sneaky [] -> :wat::core::String
     (:wat::core::let [f :wat::kernel::str-double]
       (f "AB" 3)))
   ```
   **Run it BEFORE the change and confirm it passes clean (EXIT=0)** — that is the negative control
   proving the test can fail. Then fix, and confirm `DefRestrictedCallerNotAllowed` fires. Report both
   observations (`[[feedback_a_green_test_can_prove_nothing]]`).
2. **The constructor instance closes.** Un-ignore
   `tests/types/struct_restricted.rs::struct_restricted_ctor_restriction_fires_on_illegal_caller`.
   Its `expected startup failure; got Ok` panic must be **gone**. It may then fail on a stale inline
   literal — **that is EXPECTED and in scope here**: convert it to an `.edn` golden via
   `wat::assert_edn_matches_file!` and capture, since it is this stone's regression proof. Adjudicate
   the new face before capturing (same law as the recapture campaign: read the diff, do not bless it).
3. **The live seals refuse by the alias route.** A `:user::` fn aliasing `:wat::kernel::write-fd-raw`
   must be refused. **CHECK ONLY — never execute an arbitrary-fd write.**

## THE DOC IS ALSO WRONG — fix it in the same strike

`src/check.rs:1400-1402` currently claims:

> *"The walker recurses through every `List` and `Vector` child so a call buried inside **a let body**,
> match arm, or fn-literal argument is still caught."*

The let-body case is precisely what escapes. Rewrite that paragraph to state the new rule — a
restriction governs **mention in any position**, not head position — and say why, so the next reader
does not re-narrow it.

## EXPECT THE CORPUS TO SCREAM — that fire is the worklist

This check has been under-firing for months. Turning it on will surface real sites. **Every scream is
a finding: either a genuine escape (report it) or a site that needs a whitelist entry (report it).**
Do NOT widen a whitelist or weaken a restriction to reach green — that inverts the stone.

If the count is large, report the list and STOP rather than working through it; the orchestrator
re-plans.

## STOP TRIGGERS — rejections. Report and leave the site.

- **STOP-1 — the general rule cannot work and a form list looks necessary.** That is a finding about
  the AST, not a licence. Report the exact shape that defeats it. **Do NOT ship a special case for
  `kwargs-construct`/`aggregate-new`** — that was proposed and cut: *"this is a hack... we need it to
  be general."*
- **STOP-2 — more than a handful of legitimate sites need exemptions.** A rule needing many exemptions
  is the wrong rule. Report the list; do not start granting them.
- **STOP-3 — a `src/` change beyond `walk_for_restricted_call` + its doc looks necessary.** Report it.
- **STOP-4 — you are tempted to widen a whitelist or weaken a restriction to reach green.** Never.

## BLAST RADIUS

`src/check.rs` (`walk_for_restricted_call` + its doc), new fixtures/tests under `tests/types/` (or
alongside the existing `struct_restricted` family), and whatever the imposed check screams about. **No
`.wat` corpus rewrites without reporting first** — if the corpus needs migrating, that is a wat-fix
codemod and a separate strike (R21), not hand edits.

**Out of scope, affirmatively cut:** the startup wall (W1 — an unenforceable restriction fails at
registration) and the safety-claim sweep (W2). Both are in the stone; both are separate strikes. Do
not build them here.

## VERIFY

`cargo build --release --tests`, then `cargo clippy --workspace --all-targets --release -- -D warnings`
(expect 0), then `scripts/floor.sh` and read the **Summary line** — never a piped exit code.

Baseline `4531 run / 4531 passed / 154 skipped`. Expect `+N` run for your new tests, `−1` skipped for
the un-ignored ctor test. **Report the real arithmetic against that.**

**On any red you did not intend: do NOT re-run.** `scripts/floor.sh` keeps the untruncated log. Copy
the failing test's whole stdout+stderr block **verbatim** — never a `| head` window — name the exact
assertion, and report.

## HOW TO WORK

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Run
every build and test in the **FOREGROUND** and block on it. Anchor at `/home/watmin/work/holon/wat-rs`;
`pwd` first. **Leave the work uncommitted** — the orchestrator commits.

## REPORT

- the diff of the walker, and confirmation the code got smaller
- **the negative control both ways** — the escape passing before, refused after
- each of the three gate proofs, with the exact diagnostic
- every screaming site the imposed check surfaced, with its disposition
- the floor Summary line verbatim with the arithmetic
- every STOP that fired
- **the honest deltas — especially anywhere this brief did not match the disk.** Every rider on this
  arc has found a defect in the orchestrator's brief; this one already corrects two of its own.
