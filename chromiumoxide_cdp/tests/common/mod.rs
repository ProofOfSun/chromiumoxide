// Keep the fork's narrowly scoped overlay reproducible when refreshing pinned PDL.
pub const UPSTREAM_CLIENT_SECURITY_STATE: &str = "  experimental type ClientSecurityState extends object\n    properties\n      boolean initiatorIsSecureContext\n      IPAddressSpace initiatorIPAddressSpace\n      PrivateNetworkRequestPolicy privateNetworkRequestPolicy\n";

pub const COMPAT_CLIENT_SECURITY_STATE: &str = "  experimental type ClientSecurityState extends object\n    properties\n      boolean initiatorIsSecureContext\n      IPAddressSpace initiatorIPAddressSpace\n      # Legacy policy, absent in newer Chromium. Do not infer a policy when absent.\n      optional PrivateNetworkRequestPolicy privateNetworkRequestPolicy\n      # New policy, kept separately and verbatim for forward-compatible event decoding.\n      # This value is not interchangeable with privateNetworkRequestPolicy.\n      optional string localNetworkAccessRequestPolicy\n";

pub fn apply_network_compatibility_overlay(source: &str) -> Result<String, &'static str> {
    if source.matches(UPSTREAM_CLIENT_SECURITY_STATE).count() != 1 {
        return Err(
            "Pinned ClientSecurityState changed; review the compatibility overlay before updating PDL",
        );
    }
    Ok(source.replacen(
        UPSTREAM_CLIENT_SECURITY_STATE,
        COMPAT_CLIENT_SECURITY_STATE,
        1,
    ))
}
