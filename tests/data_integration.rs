use fusion_tracker::data::{load_projects_from_str, DataError};

const VALID_JSON: &str = r#"{
  "primary_projects": [
    {
      "id": "orion",
      "project_name": "Orion",
      "company_or_operator": "Helion Energy",
      "country": "United States",
      "city_or_region": "Malaga, WA",
      "latitude": 47.6137,
      "longitude": -119.9934,
      "credibility_bucket": "front_runner",
      "globe_modes": ["commercial_race"]
    }
  ],
  "program_roadmaps": [
    {
      "id": "iter",
      "project_name": "ITER",
      "company_or_operator": "ITER Organization",
      "country": "France",
      "city_or_region": "Saint-Paul-lès-Durance",
      "latitude": 43.7069,
      "longitude": 5.7641,
      "credibility_bucket": "credible",
      "globe_modes": ["important_unique"]
    },
    {
      "id": "eu_demo",
      "project_name": "EU DEMO",
      "company_or_operator": "EUROfusion",
      "country": "EU",
      "city_or_region": "TBD",
      "credibility_bucket": "conceptual",
      "globe_modes": []
    }
  ]
}"#;

#[test]
fn loads_visible_projects_only() {
    let projects = load_projects_from_str(VALID_JSON).expect("should load");
    assert_eq!(projects.len(), 2);
    let ids: Vec<&str> = projects.iter().map(|p| p.id.as_str()).collect();
    assert!(ids.contains(&"orion"));
    assert!(ids.contains(&"iter"));
    assert!(!ids.contains(&"eu_demo"));
}

#[test]
fn skips_visible_project_without_coords() {
    let json = r#"{
      "primary_projects": [
        {
          "id": "ok","project_name":"OK","company_or_operator":"C","country":"X","city_or_region":"x",
          "latitude":1.0,"longitude":2.0,"globe_modes":["commercial_race"]
        },
        {
          "id": "no_coords","project_name":"X","company_or_operator":"X Co","country":"X","city_or_region":"x",
          "latitude": null, "longitude": null, "globe_modes":["important_unique"]
        }
      ]
    }"#;
    let projects = load_projects_from_str(json).expect("should load");
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].id, "ok");
}

#[test]
fn rejects_duplicate_id() {
    let json = r#"{
      "primary_projects": [
        {
          "id": "dup","project_name":"A","company_or_operator":"C","country":"X","city_or_region":"x",
          "latitude":1.0,"longitude":2.0,"globe_modes":["commercial_race"]
        },
        {
          "id": "dup","project_name":"B","company_or_operator":"C","country":"X","city_or_region":"x",
          "latitude":3.0,"longitude":4.0,"globe_modes":["commercial_race"]
        }
      ]
    }"#;
    let err = load_projects_from_str(json).unwrap_err();
    assert!(matches!(err, DataError::DuplicateId { .. }));
}

#[test]
fn rejects_out_of_range_coords() {
    let json = r#"{
      "primary_projects": [
        {
          "id":"x","project_name":"X","company_or_operator":"C","country":"X","city_or_region":"x",
          "latitude":95.0,"longitude":0.0,"globe_modes":["commercial_race"]
        }
      ]
    }"#;
    let err = load_projects_from_str(json).unwrap_err();
    assert!(matches!(err, DataError::OutOfRangeCoord { .. }));
}

#[test]
fn rejects_empty_visible_set() {
    let json = r#"{
      "primary_projects": [
        {
          "id":"x","project_name":"X","company_or_operator":"C","country":"X","city_or_region":"x",
          "latitude":1.0,"longitude":2.0,"globe_modes":[]
        }
      ]
    }"#;
    let err = load_projects_from_str(json).unwrap_err();
    assert!(matches!(err, DataError::NoVisibleProjects));
}

#[test]
fn rejects_malformed_json() {
    let err = load_projects_from_str("not json").unwrap_err();
    assert!(matches!(err, DataError::Parse(_)));
}

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn api_projects_returns_visible_set() {
    let projects = load_projects_from_str(VALID_JSON).unwrap();
    let app = fusion_tracker::routes::app(Arc::new(projects));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/projects")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["projects"].as_array().unwrap().len(), 2);
}

#[test]
fn real_seed_data_is_valid() {
    let raw = std::fs::read_to_string("data/projects.json").expect("seed data missing");
    let projects = load_projects_from_str(&raw).expect("seed data must validate");
    let cr = projects
        .iter()
        .filter(|p| p.globe_modes.iter().any(|m| m == "commercial_race"))
        .count();
    let iu = projects
        .iter()
        .filter(|p| p.globe_modes.iter().any(|m| m == "important_unique"))
        .count();
    assert_eq!(cr, 7, "expected 7 commercial_race projects");
    assert_eq!(
        iu, 11,
        "expected 11 important_unique projects (fast_japan skipped for missing coords)"
    );
}
