use fusion_tracker::data::load_dataset_from_str;

const VALID_JSON: &str = r#"{
  "companies": [
    {
      "id": "acme",
      "name": "Acme Fusion",
      "description": "Builds fusion.",
      "reactor_type": "Tokamak",
      "funding_raised_usd": 100000000,
      "funding_display": "$100M",
      "current_milestone": "Test milestone.",
      "source_url": "https://example.com"
    }
  ],
  "locations": [
    {
      "id": "acme-hq",
      "company_id": "acme",
      "name": "Acme HQ",
      "location_type": "hq",
      "city": "Boston",
      "country": "USA",
      "lat": 42.36,
      "lng": -71.05,
      "status": "active"
    }
  ]
}"#;

#[test]
fn loads_valid_dataset() {
    let ds = load_dataset_from_str(VALID_JSON).expect("should load");
    assert_eq!(ds.companies.len(), 1);
    assert_eq!(ds.locations.len(), 1);
    assert_eq!(ds.companies[0].id, "acme");
}
