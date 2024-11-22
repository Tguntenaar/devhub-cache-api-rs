/**
 * Test Nearblocks mocked transactions
 */

/**
 * Search Proposals
 * Get Proposals
 * Get Proposal Snapshots
 * Search RFPs
 * Get RFPs
 * Get RFP Snapshots
 */

#[test]
fn test_index() {
    use rocket::local::blocking::Client;

    // Construct a client to use for dispatching requests.
    let client = Client::tracked(super::rocket()).expect("valid `Rocket`");

    // Dispatch a request to 'GET /' and validate the response.
    let response = client.get("/").dispatch();
    assert_eq!(response.into_string().unwrap(), "Welcome from fly.io!!!!!");
}
