mod common;

use chromiumoxide_cdp::cdp::browser_protocol::network::{
    ClientSecurityState, EventRequestWillBeSentExtraInfo, IpAddressSpace,
    PrivateNetworkRequestPolicy,
};
use serde_json::{Value, json};

fn state(fields: Value) -> Value {
    let mut state =
        json!({"initiatorIsSecureContext": true, "initiatorIPAddressSpace": "Loopback"});
    state
        .as_object_mut()
        .unwrap()
        .extend(fields.as_object().unwrap().clone());
    state
}

#[test]
fn old_and_new_policies_are_preserved_separately() {
    for fields in [
        json!({"privateNetworkRequestPolicy": "Allow"}),
        json!({"localNetworkAccessRequestPolicy": "PermissionBlock"}),
        json!({"localNetworkAccessRequestPolicy": "FuturePolicy"}),
        json!({"privateNetworkRequestPolicy": "Allow", "localNetworkAccessRequestPolicy": "PermissionWarn"}),
        json!({}),
    ] {
        let input = state(fields);
        let decoded: ClientSecurityState = serde_json::from_value(input.clone()).unwrap();
        assert_eq!(serde_json::to_value(decoded).unwrap(), input);
    }
}

#[test]
fn malformed_fields_and_missing_required_state_are_rejected() {
    for fields in [
        json!({"privateNetworkRequestPolicy": 42}),
        json!({"localNetworkAccessRequestPolicy": 42}),
        json!({"initiatorIsSecureContext": "true"}),
    ] {
        assert!(serde_json::from_value::<ClientSecurityState>(state(fields)).is_err());
    }
    assert!(serde_json::from_value::<ClientSecurityState>(json!({})).is_err());
}

#[test]
fn full_extra_info_event_accepts_both_chrome_shapes() {
    for fields in [
        json!({"privateNetworkRequestPolicy": "Allow"}),
        json!({"localNetworkAccessRequestPolicy": "PermissionBlock"}),
    ] {
        let event = json!({
            "requestId": "fixture-request", "associatedCookies": [],
            "headers": {}, "connectTiming": {"requestTime": 1.0},
            "clientSecurityState": state(fields), "siteHasCookieInOtherPartition": false
        });
        let decoded: EventRequestWillBeSentExtraInfo =
            serde_json::from_value(event.clone()).unwrap();
        // Exercise the same untagged transport envelope as the WebSocket connection.
        let wire = format!(
            r#"{{"method":"Network.requestWillBeSentExtraInfo","params":{event},"sessionId":"fixture-session"}}"#
        );
        let message: chromiumoxide_types::Message<chromiumoxide_cdp::cdp::events::CdpEventMessage> =
            serde_json::from_str(&wire).unwrap();
        assert!(matches!(message, chromiumoxide_types::Message::Event(_)));
        // The generator intentionally omits empty vectors on serialization.
        assert!(decoded.associated_cookies.is_empty());
        let serialized = serde_json::to_value(decoded).unwrap();
        assert_eq!(
            serialized["clientSecurityState"],
            event["clientSecurityState"]
        );
        assert_eq!(serialized["requestId"], event["requestId"]);
    }
}

#[test]
fn generated_constructor_and_builder_do_not_invent_a_policy() {
    let empty = ClientSecurityState::new(true, IpAddressSpace::Loopback);
    assert!(empty.private_network_request_policy.is_none());
    assert!(empty.local_network_access_request_policy.is_none());
    let legacy = ClientSecurityState::builder()
        .initiator_is_secure_context(true)
        .initiator_ip_address_space(IpAddressSpace::Loopback)
        .private_network_request_policy(PrivateNetworkRequestPolicy::Allow)
        .build()
        .unwrap();
    assert_eq!(
        legacy.private_network_request_policy,
        Some(PrivateNetworkRequestPolicy::Allow)
    );
    assert!(legacy.local_network_access_request_policy.is_none());
    let modern = ClientSecurityState::builder()
        .initiator_is_secure_context(true)
        .initiator_ip_address_space(IpAddressSpace::Loopback)
        .local_network_access_request_policy("PermissionBlock")
        .build()
        .unwrap();
    assert!(modern.private_network_request_policy.is_none());
    assert_eq!(
        modern.local_network_access_request_policy.as_deref(),
        Some("PermissionBlock")
    );
}

#[test]
fn pinned_pdl_overlay_is_exact_and_rejects_unknown_upstream_shapes() {
    let network = include_str!("../pdl/domains/Network.pdl");
    assert_eq!(
        network
            .matches(common::COMPAT_CLIENT_SECURITY_STATE)
            .count(),
        1
    );
    let upstream = network.replacen(
        common::COMPAT_CLIENT_SECURITY_STATE,
        common::UPSTREAM_CLIENT_SECURITY_STATE,
        1,
    );
    assert_eq!(
        common::apply_network_compatibility_overlay(&upstream).unwrap(),
        network
    );
    assert!(common::apply_network_compatibility_overlay(network).is_err());
    assert!(common::apply_network_compatibility_overlay("unexpected source").is_err());
}
