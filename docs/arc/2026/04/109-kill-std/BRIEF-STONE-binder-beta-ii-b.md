# BRIEF — arc 109, β-ii-b: the ~20 generated FUNCTION names drop their `{p}` suffix

`defservice` appends the service's bracketed type params to every name it generates. For the
**function** names that suffix contributes nothing — it is parsed and then unioned away. This stone
deletes it from those ~20 sites. **Nothing becomes a form here**; that is β-ii-c.

Design: `DESIGN-STONE-binder-beta-ii.md` — read BOTH amendments at the end, especially the second,
which carries the census and the proof.

## Why this is a deletion and not a port

`split_name_and_type_params` (`src/runtime.rs:4156`) parses a name's `<…>`, and Stone 251.7
(`src/runtime.rs:~3614`) then **unions those params with the free bare type-vars found in the
signature**:

> *"Stone 251.7 — union raw_type_params with free bare type-vars in the signature."*

So `(defn :user::id<ZZZ> [x <- T] -> T x)` runs: `ZZZ` is added, `T` is *also* found free, both are
quantified. **A generated function name's `{p}` is additive, never authoritative** — provided every
param it names appears in that function's signature.

## The sites — FUNCTION names only

```
:580  {b}::init{p}              :824  {b}/start{p}            :2337 {b}/start$impl{p}
:602  {b}::stop-project{p}      :978  {b}::dispatch-admin{p}  :2341 {b}/start$impl-thread{p}
:630  {b}::hibernate-project{p} :980  {b}::extract-addr{p}    :2345 {b}/start$impl-process{p}
:818  {b}::serve{p}             :1936 {b}/stop{p}             :2351 {b}/resume$impl{p}
:1982 {b}/hibernate{p}          :2025 {b}/grant{p}            :2355 {b}/resume$impl-thread{p}
:2078 {b}/revoke{p}             :2511 {b}/resume{p}           :2359 {b}/resume$impl-process{p}
```

Each is a `string::interpolate "…{p}"` with `:p fqdn-tp`. **Drop the `{p}` from the template and the
`:p fqdn-tp` kwarg** — leaving `"{b}::init"`, `"{b}/start"`, and so on.

⛔ **DO NOT TOUCH these — they are TYPE names and belong to β-ii-c:**
`:525 {b}::State{p}` · `:528 {b}::Record{p}` · `:855 {b}::Handle{p}` · `:915 {b}::Admin{p}` ·
`:1080 {b}::Op{p}`

⛔ **DO NOT TOUCH the substring cluster (β-ii-d):** `:829`–`:831` (`contains? fqdn-tp "<T>"` /
`"<T,"` / `",T>"`), `:837`, `:844`, `:850` (`subs fqdn-tp 0 …`), and `:1795`.

★ **Verify before you delete each one.** The licence above holds only if every param the name
carries appears in that function's SIGNATURE. For each site, read the generated `defn`'s param and
return types in the surrounding quasiquote and confirm `K`/`V` (or whatever the service declares)
appear there. **If one does not — a function whose type param appears only in its BODY — STOP and
report it.** That is the one shape where dropping `{p}` loses information.

## STOP triggers

1. **STOP-1** — a site whose params do not appear in its own signature. Report it; do not drop it.
2. **STOP-2** — if a name is used as a KEY somewhere that expects the suffixed spelling (a lookup
   table, a `keyword/from-string` round-trip compared against a suffixed literal), STOP. The runtime
   registers under the base, but a *macro-local* comparison could still expect the suffix.
3. **STOP-3** — no new primitive, no helper `defn`, no `mapv` over a bare primitive keyword. A macro
   body may not call user-defined functions at all: F5 is default-deny and refuses AT DEFINITION,
   taking the whole stdlib down. Read
   `NOTE-the-F5-allow-list-and-what-a-macro-body-may-call.md` before writing anything.
4. **STOP-4** — edit `wat/service.wat` ONLY.

## Blast radius

`wat/service.wat` — ~20 interpolation templates and their `:p` kwargs. No type names. No forms. No
new vocabulary.

## How this lands

You are a rider. **Text edits only.** Do not run cargo, build, commit, stash, or revert.

⚠ **You cannot test this edit.** `wat/service.wat` is baked into the binary by `include_str!` at
RUST-compile time, so `--check` reflects the LAST BUILD and prints a staleness warning. Trace by
reading; report separately what you verified by reading and what you could not verify at all.

Report: the diff; the per-site verification that each dropped `{p}`'s params appear in that
function's signature; any site you did NOT drop and why; and anything on disk contradicting this
brief. My size estimates for this macro have been wrong four times — treat the brief as my claim.
