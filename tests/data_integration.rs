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

#[test]
fn rejects_orphan_location() {
    let json = r#"{
      "companies": [
        {"id":"acme","name":"a","description":"d","reactor_type":"t","funding_raised_usd":1,"funding_display":"$1","current_milestone":"m","source_url":"https://x"}
      ],
      "locations": [
        {"id":"ghost-hq","company_id":"ghost","name":"n","location_type":"hq","city":"c","country":"x","lat":0.0,"lng":0.0,"status":"active"}
      ]
    }"#;
    let err = load_dataset_from_str(json).unwrap_err();
    assert!(matches!(
        err,
        fusion_tracker::data::DataError::OrphanLocation { .. }
    ));
}

#[test]
fn rejects_company_without_locations() {
    let json = r#"{
      "companies": [
        {"id":"acme","name":"a","description":"d","reactor_type":"t","funding_raised_usd":1,"funding_display":"$1","current_milestone":"m","source_url":"https://x"},
        {"id":"lonely","name":"b","description":"d","reactor_type":"t","funding_raised_usd":1,"funding_display":"$1","current_milestone":"m","source_url":"https://x"}
      ],
      "locations": [
        {"id":"acme-hq","company_id":"acme","name":"n","location_type":"hq","city":"c","country":"x","lat":0.0,"lng":0.0,"status":"active"}
      ]
    }"#;
    let err = load_dataset_from_str(json).unwrap_err();
    assert!(matches!(
        err,
        fusion_tracker::data::DataError::CompanyWithoutLocations { .. }
    ));
}

#[test]
fn rejects_duplicate_company_id() {
    let json = r#"{
      "companies": [
        {"id":"dup","name":"a","description":"d","reactor_type":"t","funding_raised_usd":1,"funding_display":"$1","current_milestone":"m","source_url":"https://x"},
        {"id":"dup","name":"b","description":"d","reactor_type":"t","funding_raised_usd":1,"funding_display":"$1","current_milestone":"m","source_url":"https://x"}
      ],
      "locations": [
        {"id":"dup-hq","company_id":"dup","name":"n","location_type":"hq","city":"c","country":"x","lat":0.0,"lng":0.0,"status":"active"}
      ]
    }"#;
    let err = load_dataset_from_str(json).unwrap_err();
    assert!(matches!(
        err,
        fusion_tracker::data::DataError::DuplicateId { collection: "companies", .. }
    ));
}

#[test]
fn rejects_out_of_range_coord() {
    let json = r#"{
      "companies": [
        {"id":"acme","name":"a","description":"d","reactor_type":"t","funding_raised_usd":1,"funding_display":"$1","current_milestone":"m","source_url":"https://x"}
      ],
      "locations": [
        {"id":"acme-hq","company_id":"acme","name":"n","location_type":"hq","city":"c","country":"x","lat":95.0,"lng":0.0,"status":"active"}
      ]
    }"#;
    let err = load_dataset_from_str(json).unwrap_err();
    assert!(matches!(
        err,
        fusion_tracker::data::DataError::OutOfRangeCoord { .. }
    ));
}

#[test]
fn rejects_malformed_json() {
    let err = load_dataset_from_str("not json").unwrap_err();
    assert!(matches!(err, fusion_tracker::data::DataError::Parse(_)));
}

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn api_companies_returns_dataset() {
    let ds = load_dataset_from_str(VALID_JSON).unwrap();
    let app = fusion_tracker::routes::app(Arc::new(ds));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/companies")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["companies"][0]["id"], "acme");
}

#[test]
fn real_seed_data_is_valid() {
    let raw = std::fs::read_to_string("data/companies.json").expect("seed data missing");
    let ds = load_dataset_from_str(&raw).expect("seed data must validate");
    assert_eq!(ds.companies.len(), 15);
    assert!(ds.locations.len() >= 15);
}
