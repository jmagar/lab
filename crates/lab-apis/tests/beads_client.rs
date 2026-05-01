#![cfg(feature = "beads")]

use lab_apis::beads::BeadsClient;

#[test]
fn beads_contract_is_read_only() {
    let client = BeadsClient::default();
    let contract = client.contract_status();

    assert_eq!(contract.status, "cli_contract_implemented");
    assert!(contract.safe_v1_actions.contains(&"issue.ready"));
    assert!(contract.safe_v1_actions.contains(&"graph.show"));
    assert!(contract.deferred.contains(&"issue.create"));
    assert!(contract.deferred.contains(&"dolt.push"));
}
