# ClientSecurityState compatibility patch

## Base and scope

Fork: https://github.com/ProofOfSun/chromiumoxide

Patch branch: `fix/client-security-state-compat`, based on
`a7e2bb835b9643410f9e3dc044f0d947e96cbfa4`, the source commit recorded by the
published `chromiumoxide_cdp 0.9.1` crate. This intentionally excludes unrelated
changes from upstream main. Crate versions and `CURRENT_REVISION` are unchanged;
consumers must pin the exact fork commit rather than follow the branch head.

Chrome 156 emits `Network.requestWillBeSentExtraInfo.clientSecurityState`
without the formerly required `privateNetworkRequestPolicy`. Version 0.9.1
therefore rejects the event with `missing field privateNetworkRequestPolicy`.

This fork applies a narrow compatibility overlay:

- `privateNetworkRequestPolicy` is optional, retaining its existing enum.
- `localNetworkAccessRequestPolicy` is a separate optional string, preserving
  newer/unknown policy values verbatim, without mapping them to the legacy enum.
- Neither field is used to authorize requests. An absent policy means unknown,
  not Allow. No default policy is fabricated and no CDP parse error is swallowed.
- Other required fields remain required; invalid policy value types are rejected.

This is a mixed-version receive-side schema, not a wholesale protocol revision
upgrade. It does not guarantee compatibility with every newer Chrome event.

## Regeneration and API changes

The source of truth is `chromiumoxide_cdp/pdl/domains/Network.pdl`. The checked-in
`chromiumoxide_cdp/src/cdp.rs` is generated, not manually patched.

`tests/common/mod.rs` defines an exact-match overlay for the pinned upstream PDL.
The existing network-enabled `pdl_is_fresh` updater applies it only to
`domains/Network.pdl`, so refreshing the pinned source will not erase the fix.
If upstream changes this object, the overlay deliberately fails for review instead
of applying blindly. Offline tests check that the overlay exactly matches the
committed PDL and reject unexpected source shapes.

Generate/check using:

```sh
cargo test -p chromiumoxide_cdp --test generate generated_code_is_fresh
```

The first invocation after editing PDL rewrites generated code and intentionally
fails (existing upstream workflow); rerun to verify freshness. Avoid setting
`CDP_NO_EXPERIMENTAL`, `CDP_DEPRECATED` or alternate PDL paths for this check.

Public Rust API changes relative to upstream 0.9.1:

- `private_network_request_policy` is now `Option<PrivateNetworkRequestPolicy>`.
- `local_network_access_request_policy: Option<String>` is added.
- The generated `ClientSecurityState::new` now takes only the two required
  initiator arguments. To set either policy, use the builder or fields.
- The generator automatically boxes the now-larger
  `CdpEvent::NetworkRequestWillBeSentExtraInfo` variant; normal typed event
  listener APIs remain available. Direct enum pattern users must account for it.

## Validation

On macOS 26.6.2 arm64:

- `cargo fmt --all -- --check`: passed.
- `cargo check -p chromiumoxide --lib`: passed.
- `cargo test -p chromiumoxide --lib`: 4 passed.
- `cargo test -p chromiumoxide_cdp --test client_security_state`: 5 passed;
  covers legacy/new/future policy values, missing/both fields, malformed values,
  generated constructor/builder, full transport envelope, and PDL overlay.
- `cargo test -p chromiumoxide_cdp --test generate generated_code_is_fresh`:
  passed after regeneration.

Browser validation used the desktop repository's isolated local HTTP fixture
probe with this fork's **actual source**, not the earlier temporary text patch:

| Browser | Strict three-phase probe |
| --- | --- |
| Google Chrome 156.0.8078.80 | passed, zero protocol errors |
| Google Chrome for Testing 145.0.7632.6 | passed, zero protocol errors |

Phases: fresh headless → headed synthetic login → restarted headless. Each
checks JavaScript, responseReceived, loadingFinished, response JSON, synthetic
HttpOnly persistent cookie / Local Storage, graceful close and process exit.
Only newly created temporary profiles and loopback pages were used; profiles
were removed. Sandbox remained enabled; no elevated execution and no
`ignore-invalid` diagnostic mode were used.

The probe used registry `chromiumoxide 0.9.1` with path patches to the fork's
`chromiumoxide_cdp`, `chromiumoxide_types`, and `chromiumoxide_pdl`. Patch the
companion crates consistently when integrating this fork: otherwise Cargo may
resolve duplicate path/git and registry copies of `chromiumoxide_types`, which
have different Rust type identities even when version numbers match. A direct
git dependency on the root crate is another option; test whichever dependency
configuration is selected and commit its lockfile.

Not validated: Windows, Edge, Intel macOS, real platform authentication/scraping,
full desktop application compilation, all network/browser integration suites.
The network-enabled upstream `pdl_is_fresh` test was not run; overlay correctness
and generated freshness were tested offline. Initial offline workspace dependency
resolution failed due to an uncached `directories` package; fetching build
dependencies then allowed the offline checks above. The first full-event test
was corrected to account for the existing serializer's omission of empty arrays.

Keep original MIT/Apache licenses. When upstream resolves the protocol evolution,
review removal of this overlay and regenerate/retest rather than keeping duplicate
compatibility behavior indefinitely.
