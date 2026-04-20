# Fusion Tracker Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship a dark, globe-first website where users explore 15 major fusion companies via an interactive 3D globe, clicking markers to open a slide-in sidebar with company detail.

**Architecture:** Axum (Rust) serves `static/` assets plus one JSON endpoint (`/api/companies`) backed by `data/companies.json` loaded into memory at startup. Frontend is vanilla HTML/CSS/JS using globe.gl from a CDN. No database, no bundler, no framework. Deployed to Railway via Dockerfile; GitHub Actions runs fmt/clippy/tests on push.

**Tech Stack:**
- Rust 1.75+, Axum 0.7, tokio, serde, serde_json, tower-http (for static files)
- Vanilla JS, globe.gl (CDN), CSS (no preprocessor)
- Docker, Railway, GitHub Actions

**Spec:** [docs/superpowers/specs/2026-04-19-fusion-tracker-design.md](../specs/2026-04-19-fusion-tracker-design.md)

---

## File Structure

**Created by this plan:**

```
fusiontracker/
├── Cargo.toml                    # Task 1
├── Cargo.lock                    # Task 1 (auto)
├── .gitignore                    # Task 1
├── README.md                     # Task 12
├── Dockerfile                    # Task 10
├── railway.json                  # Task 10
├── .github/workflows/ci.yml      # Task 11
├── src/
│   ├── main.rs                   # Task 2, expanded Task 5
│   ├── models.rs                 # Task 3
│   ├── data.rs                   # Task 4
│   └── routes.rs                 # Task 5
├── tests/
│   └── data_integration.rs       # Task 4
├── data/
│   └── companies.json            # Task 6
└── static/
    ├── index.html                # Task 7
    ├── styles.css                # Task 7, expanded Task 9
    └── globe.js                  # Task 8, expanded Task 9
```

**Responsibility per file:**
- `main.rs`: Axum bootstrap, port binding, passes state to routes.
- `models.rs`: `Dataset`, `Company`, `Location`, enums. Pure data types + serde.
- `data.rs`: Load `companies.json` from disk, validate, return `Dataset`. Panics on bad data.
- `routes.rs`: Three handlers — index, static files, `/api/companies`.
- `companies.json`: Seed data from the TechCrunch roundup.
- `index.html`: Minimal shell with `#globe`, `#sidebar`, script/style tags.
- `globe.js`: Fetch data, init globe.gl, handle hover/click/drag/keyboard, manage sidebar.
- `styles.css`: Dark theme, sidebar glassy drawer, marker label tooltip, mobile bottom-sheet.

---

## Task 1: Project scaffolding

**Files:**
- Create: `Cargo.toml`
- Create: `.gitignore`

- [ ] **Step 1: Initialize Cargo project**

Run: `cargo init --name fusion-tracker`

Expected: creates `src/main.rs` (hello world), `Cargo.toml`, and `.gitignore`.

- [ ] **Step 2: Replace `Cargo.toml` with project dependencies**

Replace contents of `Cargo.toml` with:

```toml
[package]
name = "fusion-tracker"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
tower-http = { version = "0.5", features = ["fs"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[dev-dependencies]
tower = { version = "0.4", features = ["util"] }
http-body-util = "0.1"
```

- [ ] **Step 3: Replace `.gitignore`**

Replace contents with:

```
/target
.superpowers/
.DS_Store
*.swp
```

- [ ] **Step 4: Verify it builds**

Run: `cargo build`

Expected: "Compiling fusion-tracker ... Finished `dev` profile" with no errors.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml Cargo.lock .gitignore src/
git commit -m "chore: scaffold Cargo project with axum/tokio/serde deps"
```

---

## Task 2: Minimal Axum server that boots

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Replace `src/main.rs` with minimal Axum bootstrap**

```rust
use axum::{routing::get, Router};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(|| async { "fusion tracker boot ok" }));

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3000);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    println!("listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

- [ ] **Step 2: Run it**

Run: `cargo run`

Expected: prints `listening on 0.0.0.0:3000`. Open `http://localhost:3000/` in a browser → see "fusion tracker boot ok". Kill with Ctrl+C.

- [ ] **Step 3: Commit**

```bash
git add src/main.rs
git commit -m "feat: boot axum server on PORT or 3000"
```

---

## Task 3: Data models

**Files:**
- Create: `src/models.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Create `src/models.rs`**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LocationType {
    Hq,
    Facility,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LocationStatus {
    Active,
    Planned,
    Announced,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Company {
    pub id: String,
    pub name: String,
    pub description: String,
    pub reactor_type: String,
    pub funding_raised_usd: u64,
    pub funding_display: String,
    pub current_milestone: String,
    pub source_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub id: String,
    pub company_id: String,
    pub name: String,
    pub location_type: LocationType,
    pub city: String,
    pub country: String,
    pub lat: f64,
    pub lng: f64,
    pub status: LocationStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dataset {
    pub companies: Vec<Company>,
    pub locations: Vec<Location>,
}
```

- [ ] **Step 2: Wire module into `src/main.rs`**

At the top of `src/main.rs`, add:

```rust
mod models;
```

- [ ] **Step 3: Verify build**

Run: `cargo build`

Expected: compiles cleanly (there may be `dead_code` warnings — ignore them; they go away in Task 4).

- [ ] **Step 4: Commit**

```bash
git add src/main.rs src/models.rs
git commit -m "feat: add Dataset/Company/Location models"
```

---

## Task 4: Data loader with validation

**Files:**
- Create: `src/data.rs`
- Create: `tests/data_integration.rs`
- Modify: `src/main.rs`

### Step group A — loader skeleton (TDD)

- [ ] **Step 1: Write failing test — valid JSON loads successfully**

Create `tests/data_integration.rs`:

```rust
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
```

Also create `src/data.rs` (empty for now):

```rust
// placeholder
```

And expose modules as a library. Modify `src/main.rs` — replace `mod models;` with nothing and create `src/lib.rs`:

Create `src/lib.rs`:

```rust
pub mod data;
pub mod models;
```

Modify `src/main.rs` — replace the `mod models;` line with:

```rust
use fusion_tracker::models as _models;
```

Actually simpler: just remove the `mod models;` line from `main.rs` and use `fusion_tracker::` paths. Final `main.rs` top looks like:

```rust
use axum::{routing::get, Router};
use std::net::SocketAddr;
```

(no `mod` declarations — everything lives in the library crate now).

- [ ] **Step 2: Run test — expect failure**

Run: `cargo test --test data_integration loads_valid_dataset`

Expected: FAIL with "cannot find function `load_dataset_from_str`".

- [ ] **Step 3: Implement minimal `load_dataset_from_str`**

Replace `src/data.rs` with:

```rust
use crate::models::Dataset;

#[derive(Debug)]
pub enum DataError {
    Parse(serde_json::Error),
    OrphanLocation { location_id: String, company_id: String },
    CompanyWithoutLocations { company_id: String },
    DuplicateId { collection: &'static str, id: String },
    OutOfRangeCoord { location_id: String, lat: f64, lng: f64 },
}

impl std::fmt::Display for DataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataError::Parse(e) => write!(f, "JSON parse error: {e}"),
            DataError::OrphanLocation { location_id, company_id } => write!(
                f,
                "location '{location_id}' references unknown company_id '{company_id}'"
            ),
            DataError::CompanyWithoutLocations { company_id } => {
                write!(f, "company '{company_id}' has no locations")
            }
            DataError::DuplicateId { collection, id } => {
                write!(f, "duplicate id '{id}' in {collection}")
            }
            DataError::OutOfRangeCoord { location_id, lat, lng } => write!(
                f,
                "location '{location_id}' has out-of-range coords: lat={lat}, lng={lng}"
            ),
        }
    }
}

impl std::error::Error for DataError {}

pub fn load_dataset_from_str(raw: &str) -> Result<Dataset, DataError> {
    let ds: Dataset = serde_json::from_str(raw).map_err(DataError::Parse)?;
    Ok(ds)
}
```

- [ ] **Step 4: Run test — expect pass**

Run: `cargo test --test data_integration loads_valid_dataset`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/lib.rs src/main.rs src/data.rs tests/data_integration.rs
git commit -m "feat: add load_dataset_from_str with DataError type"
```

### Step group B — validation rules (TDD, one rule at a time)

- [ ] **Step 6: Failing test — orphan company_id rejected**

Append to `tests/data_integration.rs`:

```rust
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
```

Run: `cargo test --test data_integration rejects_orphan_location`
Expected: FAIL (current loader doesn't validate).

- [ ] **Step 7: Implement orphan + company-without-locations + duplicate-id + lat/lng validation**

Replace the `load_dataset_from_str` function in `src/data.rs` with:

```rust
pub fn load_dataset_from_str(raw: &str) -> Result<Dataset, DataError> {
    let ds: Dataset = serde_json::from_str(raw).map_err(DataError::Parse)?;
    validate(&ds)?;
    Ok(ds)
}

fn validate(ds: &Dataset) -> Result<(), DataError> {
    use std::collections::HashSet;

    let mut company_ids = HashSet::new();
    for c in &ds.companies {
        if !company_ids.insert(c.id.clone()) {
            return Err(DataError::DuplicateId {
                collection: "companies",
                id: c.id.clone(),
            });
        }
    }

    let mut location_ids = HashSet::new();
    for l in &ds.locations {
        if !location_ids.insert(l.id.clone()) {
            return Err(DataError::DuplicateId {
                collection: "locations",
                id: l.id.clone(),
            });
        }
        if !company_ids.contains(&l.company_id) {
            return Err(DataError::OrphanLocation {
                location_id: l.id.clone(),
                company_id: l.company_id.clone(),
            });
        }
        if !(-90.0..=90.0).contains(&l.lat) || !(-180.0..=180.0).contains(&l.lng) {
            return Err(DataError::OutOfRangeCoord {
                location_id: l.id.clone(),
                lat: l.lat,
                lng: l.lng,
            });
        }
    }

    let companies_with_location: HashSet<&String> =
        ds.locations.iter().map(|l| &l.company_id).collect();
    for c in &ds.companies {
        if !companies_with_location.contains(&c.id) {
            return Err(DataError::CompanyWithoutLocations {
                company_id: c.id.clone(),
            });
        }
    }

    Ok(())
}
```

Run: `cargo test --test data_integration rejects_orphan_location`
Expected: PASS.

- [ ] **Step 8: Add tests for remaining validation rules**

Append to `tests/data_integration.rs`:

```rust
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
```

Run: `cargo test --test data_integration`
Expected: all 6 tests pass.

- [ ] **Step 9: Commit**

```bash
git add src/data.rs tests/data_integration.rs
git commit -m "feat: validate dataset integrity (orphans, duplicates, coords)"
```

### Step group C — load from file path

- [ ] **Step 10: Add `load_dataset_from_path` function**

Append to `src/data.rs`:

```rust
use std::path::Path;

pub fn load_dataset_from_path<P: AsRef<Path>>(path: P) -> Dataset {
    let path_ref = path.as_ref();
    let raw = std::fs::read_to_string(path_ref)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path_ref.display()));
    match load_dataset_from_str(&raw) {
        Ok(ds) => ds,
        Err(e) => panic!("invalid dataset at {}: {e}", path_ref.display()),
    }
}
```

This is the production entry point — it panics on any failure so the app refuses to start with bad data.

- [ ] **Step 11: Verify build**

Run: `cargo build && cargo test`
Expected: all tests pass.

- [ ] **Step 12: Commit**

```bash
git add src/data.rs
git commit -m "feat: add load_dataset_from_path that panics on invalid data"
```

---

## Task 5: Routes with shared state

**Files:**
- Create: `src/routes.rs`
- Modify: `src/lib.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Write failing test — /api/companies returns dataset JSON**

Append to `tests/data_integration.rs`:

```rust
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
```

Run: `cargo test --test data_integration api_companies_returns_dataset`
Expected: FAIL — `fusion_tracker::routes` module doesn't exist.

- [ ] **Step 2: Create `src/routes.rs`**

```rust
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use std::sync::Arc;
use tower_http::services::ServeDir;

use crate::models::Dataset;

pub type SharedState = Arc<Dataset>;

pub fn app(state: SharedState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/api/companies", get(companies))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state)
}

async fn index() -> impl IntoResponse {
    match tokio::fs::read_to_string("static/index.html").await {
        Ok(html) => (
            StatusCode::OK,
            [("content-type", "text/html; charset=utf-8")],
            html,
        )
            .into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "index.html missing").into_response(),
    }
}

async fn companies(State(ds): State<SharedState>) -> Json<Dataset> {
    Json((*ds).clone())
}
```

- [ ] **Step 3: Wire `routes` module into `src/lib.rs`**

Replace `src/lib.rs` with:

```rust
pub mod data;
pub mod models;
pub mod routes;
```

- [ ] **Step 4: Run test — expect pass**

Run: `cargo test --test data_integration api_companies_returns_dataset`
Expected: PASS.

- [ ] **Step 5: Wire into `main.rs`**

Replace `src/main.rs` with:

```rust
use fusion_tracker::{data, routes};
use std::net::SocketAddr;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let dataset = data::load_dataset_from_path("data/companies.json");
    let app = routes::app(Arc::new(dataset));

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3000);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    println!("listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

- [ ] **Step 6: Verify build**

Run: `cargo build`
Expected: compiles. `cargo run` will now fail at startup (no `data/companies.json` yet) — that's expected; Task 6 fixes it.

- [ ] **Step 7: Commit**

```bash
git add src/lib.rs src/main.rs src/routes.rs tests/data_integration.rs
git commit -m "feat: add routes with index, /api/companies, /static"
```

---

## Task 6: Seed `data/companies.json`

**Files:**
- Create: `data/companies.json`

- [ ] **Step 1: Create `data/companies.json` with all 15 companies**

Content (full file):

```json
{
  "companies": [
    {
      "id": "cfs",
      "name": "Commonwealth Fusion Systems",
      "description": "Tokamak with high-temperature superconducting magnets; has raised about a third of all private fusion capital.",
      "reactor_type": "Tokamak (HTS magnets)",
      "funding_raised_usd": 3000000000,
      "funding_display": "~$3B",
      "current_milestone": "Expects Sparc operational late 2026 or early 2027.",
      "source_url": "https://techcrunch.com/2026/04/10/every-fusion-startup-that-has-raised-over-100m/"
    },
    {
      "id": "tae",
      "name": "TAE Technologies",
      "description": "Field-reversed configuration with particle beam bombardment; founded 1998 as a UC Irvine spinout.",
      "reactor_type": "Field-reversed configuration",
      "funding_raised_usd": 1790000000,
      "funding_display": "$1.79B",
      "current_milestone": "Announced merger with Trump Media & Technology Group (Dec 2025) at a $6B combined valuation.",
      "source_url": "https://techcrunch.com/2026/04/10/every-fusion-startup-that-has-raised-over-100m/"
    },
    {
      "id": "helion",
      "name": "Helion",
      "description": "Field-reversed configuration with direct electricity harvesting; has the most aggressive commercialization timeline.",
      "reactor_type": "Field-reversed configuration",
      "funding_raised_usd": 1030000000,
      "funding_display": "$1.03B",
      "current_milestone": "Plans to produce electricity by 2028 for Microsoft; raised $425M in Jan 2025.",
      "source_url": "https://techcrunch.com/2026/04/10/every-fusion-startup-that-has-raised-over-100m/"
    },
    {
      "id": "pacific-fusion",
      "name": "Pacific Fusion",
      "description": "Inertial confinement using 156 coordinated Marx generators; led by Eric Lander and Will Regan.",
      "reactor_type": "Inertial confinement (pulsed power)",
      "funding_raised_usd": 900000000,
      "funding_display": "$900M Series A",
      "current_milestone": "Funding structured in milestone-gated tranches.",
      "source_url": "https://techcrunch.com/2026/04/10/every-fusion-startup-that-has-raised-over-100m/"
    },
    {
      "id": "shine",
      "name": "Shine Technologies",
      "description": "Pragmatic path generating near-term revenue via neutron testing, medical isotopes, and waste recycling.",
      "reactor_type": "Neutron source / fusion precursor",
      "funding_raised_usd": 1000000000,
      "funding_display": "$1B",
      "current_milestone": "Raised $240M in Feb 2026.",
      "source_url": "https://techcrunch.com/2026/04/10/every-fusion-startup-that-has-raised-over-100m/"
    },
    {
      "id": "general-fusion",
      "name": "General Fusion",
      "description": "Magnetized target fusion with liquid metal compression; founded 2002 by physicist Michel Laberge.",
      "reactor_type": "Magnetized target fusion",
      "funding_raised_usd": 612000000,
      "funding_display": "$612M",
      "current_milestone": "Planning IPO via reverse merger, potentially raising an additional $335M.",
      "source_url": "https://techcrunch.com/2026/04/10/every-fusion-startup-that-has-raised-over-100m/"
    },
    {
      "id": "inertia",
      "name": "Inertia Enterprises",
      "description": "Laser-based inertial confinement with silicon nanostructure targets; founders include Annie Kritcher, Mike Dunne, and Jeff Lawson.",
      "reactor_type": "Laser inertial confinement",
      "funding_raised_usd": 450000000,
      "funding_display": "$450M Series A",
      "current_milestone": "Emerged from stealth Feb 2026; demonstration facility with CSU targeted for 2027.",
      "source_url": "https://techcrunch.com/2026/04/10/every-fusion-startup-that-has-raised-over-100m/"
    },
    {
      "id": "tokamak-energy",
      "name": "Tokamak Energy",
      "description": "Compact spherical tokamak with HTS magnets; squeezed design reduces magnet requirements.",
      "reactor_type": "Spherical tokamak (HTS)",
      "funding_raised_usd": 336000000,
      "funding_display": "$336M",
      "current_milestone": "Raised $125M in Nov 2024; Demo 4 under construction.",
      "source_url": "https://techcrunch.com/2026/04/10/every-fusion-startup-that-has-raised-over-100m/"
    },
    {
      "id": "zap-energy",
      "name": "Zap Energy",
      "description": "Electrostatic confinement generating its own magnetic field; compresses plasma to ~1 mm for ignition.",
      "reactor_type": "Sheared-flow Z-pinch",
      "funding_raised_usd": 327000000,
      "funding_display": "$327M",
      "current_milestone": "Backed by Breakthrough Energy Ventures.",
      "source_url": "https://techcrunch.com/2026/04/10/every-fusion-startup-that-has-raised-over-100m/"
    },
    {
      "id": "type-one",
      "name": "Type One Energy",
      "description": "Stellarator magnetic confinement; plans to license technology for TVA coal plant site conversion.",
      "reactor_type": "Stellarator",
      "funding_raised_usd": 269000000,
      "funding_display": "$269M",
      "current_milestone": "Raised $87M equity round ahead of a $250M Series B in progress.",
      "source_url": "https://techcrunch.com/2026/04/10/every-fusion-startup-that-has-raised-over-100m/"
    },
    {
      "id": "proxima",
      "name": "Proxima Fusion",
      "description": "Stellarator with twisted, bulging magnetic confinement; bucking the tokamak trend.",
      "reactor_type": "Stellarator",
      "funding_raised_usd": 200000000,
      "funding_display": "€185M",
      "current_milestone": "Raised €130M Series A in Jun 2025.",
      "source_url": "https://techcrunch.com/2026/04/10/every-fusion-startup-that-has-raised-over-100m/"
    },
    {
      "id": "kyoto-fusioneering",
      "name": "Kyoto Fusioneering",
      "description": "Balance-of-plant components and integration systems including gyrotrons and heat extraction.",
      "reactor_type": "Supporting technology",
      "funding_raised_usd": 191000000,
      "funding_display": "$191M",
      "current_milestone": "Positioning as a supplier for whichever fusion technology succeeds.",
      "source_url": "https://techcrunch.com/2026/04/10/every-fusion-startup-that-has-raised-over-100m/"
    },
    {
      "id": "marvel",
      "name": "Marvel Fusion",
      "description": "Laser-based inertial confinement using semiconductor-manufactured silicon nanostructure targets.",
      "reactor_type": "Laser inertial confinement",
      "funding_raised_usd": 162000000,
      "funding_display": "$162M",
      "current_milestone": "Raised $70M in 2024; CSU demonstration facility targeted for 2027.",
      "source_url": "https://techcrunch.com/2026/04/10/every-fusion-startup-that-has-raised-over-100m/"
    },
    {
      "id": "first-light",
      "name": "First Light Fusion",
      "description": "Projectile-based inertial confinement via two-stage gun using gunpowder and hydraulic compression.",
      "reactor_type": "Projectile inertial confinement",
      "funding_raised_usd": 108000000,
      "funding_display": "$108M",
      "current_milestone": "Pivoted to licensing technology rather than building power plants (Mar 2025).",
      "source_url": "https://techcrunch.com/2026/04/10/every-fusion-startup-that-has-raised-over-100m/"
    },
    {
      "id": "xcimer",
      "name": "Xcimer",
      "description": "Laser-based inertial confinement with molten salt walls; aims for a 10-megajoule laser (5× NIF).",
      "reactor_type": "Laser inertial confinement",
      "funding_raised_usd": 100000000,
      "funding_display": "$100M",
      "current_milestone": "Reached the $100M funding threshold; founded January 2022.",
      "source_url": "https://techcrunch.com/2026/04/10/every-fusion-startup-that-has-raised-over-100m/"
    }
  ],
  "locations": [
    {"id": "cfs-hq",    "company_id": "cfs",              "name": "CFS Headquarters",           "location_type": "hq",       "city": "Devens",       "country": "USA",     "lat": 42.5359, "lng": -71.6070, "status": "active"},
    {"id": "cfs-sparc", "company_id": "cfs",              "name": "Sparc Reactor",              "location_type": "facility", "city": "Devens",       "country": "USA",     "lat": 42.5359, "lng": -71.6070, "status": "active"},
    {"id": "cfs-arc",   "company_id": "cfs",              "name": "Arc Commercial Plant",       "location_type": "facility", "city": "near Richmond","country": "USA",     "lat": 37.5407, "lng": -77.4360, "status": "planned"},

    {"id": "tae-hq",    "company_id": "tae",              "name": "TAE Headquarters",           "location_type": "hq",       "city": "Foothill Ranch","country": "USA",    "lat": 33.6856, "lng": -117.6653, "status": "active"},

    {"id": "helion-hq",      "company_id": "helion",      "name": "Helion Headquarters",        "location_type": "hq",       "city": "Everett",      "country": "USA",     "lat": 47.9790, "lng": -122.2021, "status": "active"},
    {"id": "helion-polaris", "company_id": "helion",      "name": "Polaris Prototype",          "location_type": "facility", "city": "Everett",      "country": "USA",     "lat": 47.9790, "lng": -122.2021, "status": "active"},

    {"id": "pacific-hq", "company_id": "pacific-fusion",  "name": "Pacific Fusion HQ",          "location_type": "hq",       "city": "Fremont",      "country": "USA",     "lat": 37.5485, "lng": -121.9886, "status": "active"},

    {"id": "shine-hq",  "company_id": "shine",            "name": "Shine Headquarters",         "location_type": "hq",       "city": "Janesville",   "country": "USA",     "lat": 42.6828, "lng": -89.0187, "status": "active"},

    {"id": "gf-hq",     "company_id": "general-fusion",   "name": "General Fusion HQ",          "location_type": "hq",       "city": "Richmond, BC", "country": "Canada",  "lat": 49.1666, "lng": -123.1336, "status": "active"},
    {"id": "gf-lm26",   "company_id": "general-fusion",   "name": "LM26 Device",                "location_type": "facility", "city": "Richmond, BC", "country": "Canada",  "lat": 49.1666, "lng": -123.1336, "status": "active"},

    {"id": "inertia-hq",  "company_id": "inertia",        "name": "Inertia Enterprises HQ",     "location_type": "hq",       "city": "Livermore",    "country": "USA",     "lat": 37.6819, "lng": -121.7680, "status": "active"},
    {"id": "inertia-csu", "company_id": "inertia",        "name": "CSU Demonstration Facility", "location_type": "facility", "city": "Fort Collins", "country": "USA",     "lat": 40.5853, "lng": -105.0844, "status": "planned"},

    {"id": "te-hq",      "company_id": "tokamak-energy", "name": "Tokamak Energy HQ",           "location_type": "hq",       "city": "Abingdon",     "country": "UK",      "lat": 51.6708, "lng":  -1.2880, "status": "active"},
    {"id": "te-st40",    "company_id": "tokamak-energy", "name": "ST40 Prototype",              "location_type": "facility", "city": "Abingdon",     "country": "UK",      "lat": 51.6708, "lng":  -1.2880, "status": "active"},
    {"id": "te-demo4",   "company_id": "tokamak-energy", "name": "Demo 4",                      "location_type": "facility", "city": "Abingdon",     "country": "UK",      "lat": 51.6708, "lng":  -1.2880, "status": "planned"},

    {"id": "zap-hq",     "company_id": "zap-energy",      "name": "Zap Energy HQ",              "location_type": "hq",       "city": "Everett",      "country": "USA",     "lat": 47.9790, "lng": -122.2021, "status": "active"},

    {"id": "type1-hq",   "company_id": "type-one",        "name": "Type One Energy HQ",         "location_type": "hq",       "city": "Madison",      "country": "USA",     "lat": 43.0731, "lng": -89.4012, "status": "active"},
    {"id": "type1-tva",  "company_id": "type-one",        "name": "TVA Bull Run Site",          "location_type": "facility", "city": "Oak Ridge",    "country": "USA",     "lat": 36.0103, "lng": -84.2696, "status": "planned"},

    {"id": "proxima-hq", "company_id": "proxima",         "name": "Proxima Fusion HQ",          "location_type": "hq",       "city": "Munich",       "country": "Germany", "lat": 48.1351, "lng":  11.5820, "status": "active"},

    {"id": "kyoto-hq",   "company_id": "kyoto-fusioneering","name": "Kyoto Fusioneering HQ",    "location_type": "hq",       "city": "Tokyo",        "country": "Japan",   "lat": 35.6762, "lng": 139.6503, "status": "active"},

    {"id": "marvel-hq",  "company_id": "marvel",          "name": "Marvel Fusion HQ",           "location_type": "hq",       "city": "Munich",       "country": "Germany", "lat": 48.1351, "lng":  11.5820, "status": "active"},
    {"id": "marvel-csu", "company_id": "marvel",          "name": "CSU Demonstration Facility", "location_type": "facility", "city": "Fort Collins", "country": "USA",     "lat": 40.5853, "lng": -105.0844, "status": "planned"},

    {"id": "fl-hq",      "company_id": "first-light",     "name": "First Light Fusion HQ",      "location_type": "hq",       "city": "Yarnton",      "country": "UK",      "lat": 51.8054, "lng":  -1.3093, "status": "active"},

    {"id": "xcimer-hq",  "company_id": "xcimer",          "name": "Xcimer Energy HQ",           "location_type": "hq",       "city": "Denver",       "country": "USA",     "lat": 39.7392, "lng": -104.9903, "status": "active"}
  ]
}
```

- [ ] **Step 2: Verify loader accepts it**

Add an integration test. Append to `tests/data_integration.rs`:

```rust
#[test]
fn real_seed_data_is_valid() {
    let raw = std::fs::read_to_string("data/companies.json").expect("seed data missing");
    let ds = load_dataset_from_str(&raw).expect("seed data must validate");
    assert_eq!(ds.companies.len(), 15);
    assert!(ds.locations.len() >= 15);
}
```

Run: `cargo test --test data_integration real_seed_data_is_valid`
Expected: PASS.

- [ ] **Step 3: Smoke-test the server**

Run: `cargo run`
In another terminal: `curl -s http://localhost:3000/api/companies | head -c 200`
Expected: JSON starting with `{"companies":[{"id":"cfs",...`
Kill `cargo run` with Ctrl+C.

- [ ] **Step 4: Commit**

```bash
git add data/companies.json tests/data_integration.rs
git commit -m "feat: seed 15 companies and 23 locations from TechCrunch roundup"
```

---

## Task 7: Static shell (HTML + base CSS)

**Files:**
- Create: `static/index.html`
- Create: `static/styles.css`

- [ ] **Step 1: Create `static/index.html`**

```html
<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Fusion Tracker</title>
  <link rel="stylesheet" href="/static/styles.css">
</head>
<body>
  <main>
    <div id="globe"></div>

    <aside id="sidebar" hidden role="dialog" aria-labelledby="sidebar-company-name">
      <button id="sidebar-close" type="button" aria-label="Close company details">×</button>
      <h2 id="sidebar-company-name"></h2>
      <p id="sidebar-description"></p>
      <div id="sidebar-location-badge" class="badge"></div>
      <dl class="facts">
        <dt>Headquarters</dt><dd id="sidebar-hq"></dd>
        <dt>Reactor</dt><dd id="sidebar-reactor"></dd>
        <dt>Funding raised</dt><dd id="sidebar-funding"></dd>
        <dt>Current milestone</dt><dd id="sidebar-milestone"></dd>
      </dl>
      <a id="sidebar-source" href="#" target="_blank" rel="noopener">Read more →</a>
    </aside>

    <div id="toast" hidden></div>
  </main>

  <script src="https://unpkg.com/globe.gl@2.32.3/dist/globe.gl.min.js"></script>
  <script src="/static/globe.js"></script>
</body>
</html>
```

- [ ] **Step 2: Create `static/styles.css`**

```css
:root {
  --bg: #05070d;
  --fg: #e6eef7;
  --muted: #8da1bb;
  --accent-teal: #4dd0e1;
  --accent-amber: #ffb74d;
  --surface: rgba(14, 20, 32, 0.78);
  --border: rgba(140, 170, 210, 0.18);
}

* { box-sizing: border-box; }

html, body {
  margin: 0;
  padding: 0;
  background: var(--bg);
  color: var(--fg);
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Inter, sans-serif;
  overflow: hidden;
  height: 100%;
}

main {
  position: relative;
  width: 100vw;
  height: 100vh;
}

#globe {
  width: 100%;
  height: 100%;
  cursor: grab;
}
#globe:active { cursor: grabbing; }

#sidebar {
  position: fixed;
  top: 0;
  right: 0;
  height: 100vh;
  width: 360px;
  padding: 64px 28px 28px;
  background: var(--surface);
  border-left: 1px solid var(--border);
  backdrop-filter: blur(14px);
  -webkit-backdrop-filter: blur(14px);
  transform: translateX(100%);
  transition: transform 250ms ease-out;
  overflow-y: auto;
  z-index: 10;
}
#sidebar:not([hidden]) { transform: translateX(0); }
#sidebar[hidden] { display: block; transform: translateX(100%); pointer-events: none; }

#sidebar-close {
  position: absolute;
  top: 14px;
  right: 14px;
  background: transparent;
  color: var(--fg);
  border: 1px solid var(--border);
  width: 32px;
  height: 32px;
  border-radius: 50%;
  font-size: 18px;
  cursor: pointer;
}
#sidebar-close:hover { background: rgba(255,255,255,0.06); }

#sidebar h2 { margin: 0 0 8px; font-size: 22px; }
#sidebar p { color: var(--muted); margin: 0 0 16px; line-height: 1.45; }

.badge {
  display: inline-block;
  font-size: 12px;
  padding: 4px 10px;
  border-radius: 999px;
  background: rgba(77, 208, 225, 0.12);
  color: var(--accent-teal);
  border: 1px solid rgba(77, 208, 225, 0.3);
  margin-bottom: 20px;
}

.facts {
  display: grid;
  grid-template-columns: auto 1fr;
  gap: 6px 16px;
  margin: 0 0 20px;
}
.facts dt { color: var(--muted); font-size: 12px; text-transform: uppercase; letter-spacing: 0.05em; align-self: center; }
.facts dd { margin: 0; font-size: 14px; }

#sidebar a { color: var(--accent-teal); text-decoration: none; font-size: 13px; }
#sidebar a:hover { text-decoration: underline; }

/* Marker hover label (globe.gl uses its own label markup; we style via a .label class below) */
.marker-label {
  background: rgba(10, 14, 22, 0.92);
  color: var(--fg);
  padding: 4px 8px;
  border-radius: 4px;
  border: 1px solid var(--border);
  font-size: 12px;
  white-space: nowrap;
}

/* Toast for fetch failures */
#toast {
  position: fixed;
  top: 16px;
  right: 16px;
  background: rgba(80, 20, 20, 0.9);
  border: 1px solid rgba(255, 120, 120, 0.3);
  padding: 10px 14px;
  border-radius: 6px;
  font-size: 13px;
  z-index: 20;
}

/* Mobile: bottom sheet */
@media (max-width: 768px) {
  #sidebar {
    width: 100vw;
    height: 60vh;
    top: auto;
    bottom: 0;
    border-left: none;
    border-top: 1px solid var(--border);
    transform: translateY(100%);
    padding: 48px 20px 20px;
  }
  #sidebar:not([hidden]) { transform: translateY(0); }
  #sidebar[hidden] { transform: translateY(100%); }
}
```

- [ ] **Step 3: Smoke-test in browser**

Run: `cargo run`
Open `http://localhost:3000/` — expect a dark empty page (no globe yet — that's Task 8). No console errors for the CSS/HTML load.

- [ ] **Step 4: Commit**

```bash
git add static/index.html static/styles.css
git commit -m "feat: add index.html shell and base dark theme"
```

---

## Task 8: Globe rendering with markers

**Files:**
- Create: `static/globe.js`

- [ ] **Step 1: Create `static/globe.js` — data fetch + globe init + markers**

```javascript
const NIGHT_LIGHTS_URL = "https://unpkg.com/three-globe@2.31.1/example/img/earth-night.jpg";
const TEAL = "#4dd0e1";
const AMBER = "#ffb74d";
const FACILITY_ALTITUDE = 0.015;
const HQ_ALTITUDE = 0.0;

const state = {
  dataset: null,
  selectedCompanyId: null,
  globe: null,
  autoRotateTimer: null,
};

async function main() {
  try {
    const res = await fetch("/api/companies");
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    state.dataset = await res.json();
  } catch (err) {
    console.error("failed to load /api/companies", err);
    showToast("Couldn't load companies. Refresh to retry.");
    initGlobe([]);
    return;
  }
  initGlobe(state.dataset.locations);
  attachSidebarHandlers();
}

function initGlobe(locations) {
  const globeEl = document.getElementById("globe");
  const globe = Globe()(globeEl)
    .globeImageUrl(NIGHT_LIGHTS_URL)
    .backgroundColor("#05070d")
    .atmosphereColor("#4dd0e1")
    .atmosphereAltitude(0.15)
    .pointsData(locations)
    .pointLat((d) => d.lat)
    .pointLng((d) => d.lng)
    .pointAltitude((d) => (d.location_type === "facility" ? FACILITY_ALTITUDE : HQ_ALTITUDE))
    .pointRadius(0.4)
    .pointColor((d) =>
      state.selectedCompanyId === d.company_id ? AMBER : TEAL
    )
    .pointLabel((d) => {
      const company = findCompany(d.company_id);
      return `<div class="marker-label">${escapeHtml(d.name)} — ${escapeHtml(company?.name ?? "")}</div>`;
    })
    .onPointClick((d) => handleMarkerClick(d));

  globe.controls().autoRotate = true;
  globe.controls().autoRotateSpeed = 0.3;
  globe.controls().addEventListener("start", () => pauseAutoRotate());

  state.globe = globe;
}

function handleMarkerClick(location) {
  const company = findCompany(location.company_id);
  if (!company) {
    console.warn("click on unknown company_id", location.company_id);
    return;
  }
  state.selectedCompanyId = company.id;
  state.globe.pointColor((d) =>
    state.selectedCompanyId === d.company_id ? AMBER : TEAL
  );

  centerOnLocation(location);
  populateSidebar(company, location);
  openSidebar();
  pauseAutoRotate({ resume: false });
}

function centerOnLocation(location) {
  // offset the longitude so the marker lands at ~35% from the left
  // with a 360px right drawer, that's roughly -20 degrees of eastward offset
  const lngOffset = window.innerWidth > 768 ? -20 : 0;
  state.globe.pointOfView(
    { lat: location.lat, lng: location.lng + lngOffset, altitude: 1.8 },
    800
  );
}

function findCompany(id) {
  return state.dataset?.companies.find((c) => c.id === id);
}

function findHqLocation(companyId) {
  return state.dataset?.locations.find(
    (l) => l.company_id === companyId && l.location_type === "hq"
  );
}

function populateSidebar(company, location) {
  const hq = findHqLocation(company.id);
  document.getElementById("sidebar-company-name").textContent = company.name;
  document.getElementById("sidebar-description").textContent = company.description;

  const typeLabel = location.location_type === "hq" ? "HQ" : "Facility";
  document.getElementById("sidebar-location-badge").textContent =
    `Viewing: ${location.name} · ${typeLabel} (${location.status})`;

  document.getElementById("sidebar-hq").textContent = hq
    ? `${hq.city}, ${hq.country}`
    : "—";
  document.getElementById("sidebar-reactor").textContent = company.reactor_type;
  document.getElementById("sidebar-funding").textContent = company.funding_display;
  document.getElementById("sidebar-milestone").textContent = company.current_milestone;
  const src = document.getElementById("sidebar-source");
  src.href = company.source_url;
}

function openSidebar() {
  const sidebar = document.getElementById("sidebar");
  sidebar.hidden = false;
  setTimeout(() => document.getElementById("sidebar-close").focus(), 50);
}

function closeSidebar() {
  const sidebar = document.getElementById("sidebar");
  sidebar.hidden = true;
  state.selectedCompanyId = null;
  if (state.globe) {
    state.globe.pointColor(() => TEAL);
  }
  setTimeout(() => resumeAutoRotate(), 500);
}

function attachSidebarHandlers() {
  document.getElementById("sidebar-close").addEventListener("click", closeSidebar);
  document.addEventListener("keydown", (e) => {
    if (e.key === "Escape") closeSidebar();
  });
  // click outside sidebar (onto globe) closes it
  document.getElementById("globe").addEventListener("click", (e) => {
    // only close if click wasn't on a marker (marker clicks are handled by globe.gl first and re-open)
    // globe.gl's onPointClick has already fired before this bubbles, so if selectedCompanyId didn't change, close
    // Simpler: if sidebar is open and no marker click happened in this tick, close.
    // We rely on the marker click handler to *set* selectedCompanyId; a click that hits empty globe
    // won't set it. Debounce check with a short delay:
    setTimeout(() => {
      if (!state.lastMarkerClickAt || Date.now() - state.lastMarkerClickAt > 100) {
        if (!document.getElementById("sidebar").hidden) closeSidebar();
      }
    }, 0);
  }, true);
}

function pauseAutoRotate({ resume = true } = {}) {
  if (!state.globe) return;
  state.globe.controls().autoRotate = false;
  if (state.autoRotateTimer) clearTimeout(state.autoRotateTimer);
  if (resume) {
    state.autoRotateTimer = setTimeout(() => {
      if (state.globe && document.getElementById("sidebar").hidden) {
        state.globe.controls().autoRotate = true;
      }
    }, 5000);
  }
}

function resumeAutoRotate() {
  if (!state.globe) return;
  state.globe.controls().autoRotate = true;
}

function showToast(msg) {
  const toast = document.getElementById("toast");
  toast.textContent = msg;
  toast.hidden = false;
}

function escapeHtml(s) {
  return String(s).replace(/[&<>"']/g, (c) => ({
    "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;"
  }[c]));
}

main();
```

- [ ] **Step 2: Fix marker-click-vs-globe-click race**

The naive globe-click handler above will race with marker clicks. Replace the `attachSidebarHandlers` function with a version that tracks marker click timing via the existing `handleMarkerClick`. In `handleMarkerClick`, add as the first line:

```javascript
  state.lastMarkerClickAt = Date.now();
```

The existing `setTimeout` check in `attachSidebarHandlers` then correctly skips closing when a marker was just clicked.

- [ ] **Step 3: Smoke-test**

Run: `cargo run`, then open `http://localhost:3000/`.

Verify:
- Globe loads with night-lights texture.
- Teal markers visible on every HQ; slightly-elevated markers at facility locations.
- Slow auto-rotate in idle.
- Drag pauses rotation.
- Hover shows `"{location.name} — {company.name}"` label.
- Click on CFS marker → all 3 CFS markers turn amber, globe rotates to center, sidebar slides in with "Commonwealth Fusion Systems" and the correct location badge.
- Click a different company → sidebar swaps.
- Esc closes sidebar; X closes sidebar; click on empty ocean closes sidebar.

- [ ] **Step 4: Commit**

```bash
git add static/globe.js
git commit -m "feat: render globe, markers, hover labels, and sidebar interactions"
```

---

## Task 9: Marker selection pulse + sidebar crossfade

**Files:**
- Modify: `static/globe.js`
- Modify: `static/styles.css`

- [ ] **Step 1: Add a one-shot pulse on selection**

In `static/globe.js`, replace `handleMarkerClick` with:

```javascript
function handleMarkerClick(location) {
  state.lastMarkerClickAt = Date.now();
  const company = findCompany(location.company_id);
  if (!company) {
    console.warn("click on unknown company_id", location.company_id);
    return;
  }
  const previousId = state.selectedCompanyId;
  state.selectedCompanyId = company.id;

  // base recolor (teal → amber for selected company)
  state.globe.pointColor((d) =>
    state.selectedCompanyId === d.company_id ? AMBER : TEAL
  );

  // one-shot pulse: briefly boost radius on selected markers, then settle
  const baseRadius = 0.4;
  const pulseRadius = 0.75;
  state.globe.pointRadius((d) =>
    state.selectedCompanyId === d.company_id ? pulseRadius : baseRadius
  );
  setTimeout(() => {
    state.globe.pointRadius(() => baseRadius);
  }, 400);

  centerOnLocation(location);
  const sidebarOpen = !document.getElementById("sidebar").hidden;
  if (sidebarOpen && previousId !== company.id) {
    crossfadeSidebar(() => {
      populateSidebar(company, location);
    });
  } else {
    populateSidebar(company, location);
    openSidebar();
  }
  pauseAutoRotate({ resume: false });
}

function crossfadeSidebar(updateFn) {
  const sidebar = document.getElementById("sidebar");
  sidebar.classList.add("fading");
  setTimeout(() => {
    updateFn();
    sidebar.classList.remove("fading");
  }, 150);
}
```

- [ ] **Step 2: Add crossfade CSS**

Append to `static/styles.css`:

```css
#sidebar.fading > :not(#sidebar-close) {
  opacity: 0;
  transition: opacity 150ms ease;
}
#sidebar > :not(#sidebar-close) {
  opacity: 1;
  transition: opacity 150ms ease;
}
```

- [ ] **Step 3: Smoke-test**

Run: `cargo run`, reload. Click a company marker — brief radius pulse, sidebar opens. While sidebar is open, click a different company's marker — content crossfades in place without close/re-open flicker.

- [ ] **Step 4: Commit**

```bash
git add static/globe.js static/styles.css
git commit -m "feat: add selection pulse and sidebar crossfade on company switch"
```

---

## Task 10: Dockerfile + Railway config

**Files:**
- Create: `Dockerfile`
- Create: `railway.json`

- [ ] **Step 1: Create `Dockerfile`**

```dockerfile
# ---- builder ----
FROM rust:1.75-slim AS builder
WORKDIR /app

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src

# Copy source and rebuild
COPY src ./src
COPY tests ./tests
RUN touch src/main.rs && cargo build --release

# ---- runtime ----
FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/fusion-tracker /app/fusion-tracker
COPY static ./static
COPY data ./data

ENV PORT=8080
EXPOSE 8080
CMD ["/app/fusion-tracker"]
```

- [ ] **Step 2: Create `railway.json`**

```json
{
  "$schema": "https://railway.app/railway.schema.json",
  "build": {
    "builder": "DOCKERFILE",
    "dockerfilePath": "Dockerfile"
  },
  "deploy": {
    "healthcheckPath": "/api/companies",
    "healthcheckTimeout": 30,
    "restartPolicyType": "ON_FAILURE",
    "restartPolicyMaxRetries": 3
  }
}
```

- [ ] **Step 3: Local docker smoke-test**

Run:

```bash
docker build -t fusion-tracker .
docker run -p 8080:8080 fusion-tracker
```

In another terminal: `curl -s http://localhost:8080/api/companies | head -c 80`
Expected: JSON starting with `{"companies":[{"id":"cfs"`. Kill container.

- [ ] **Step 4: Commit**

```bash
git add Dockerfile railway.json
git commit -m "chore: add Dockerfile and Railway deploy config"
```

---

## Task 11: GitHub Actions CI

**Files:**
- Create: `.github/workflows/ci.yml`

- [ ] **Step 1: Create CI workflow**

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - uses: Swatinem/rust-cache@v2

      - name: fmt
        run: cargo fmt --all -- --check

      - name: clippy
        run: cargo clippy --all-targets -- -D warnings

      - name: test
        run: cargo test --all
```

- [ ] **Step 2: Run checks locally before commit**

Run:

```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test --all
```

Expected: fmt makes no changes (or re-run and commit the formatting), clippy passes with no warnings, all tests pass. Fix any clippy warnings by addressing the underlying issue (not by suppressing).

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: add fmt/clippy/test workflow"
```

---

## Task 12: README

**Files:**
- Create: `README.md`

- [ ] **Step 1: Create README**

```markdown
# Fusion Tracker

Dark, globe-first website showing major fusion companies and their reactor/demo facilities around the world. Seed dataset: the 15 companies in TechCrunch's April 2026 roundup of fusion startups that have raised over $100M.

## Run locally

```bash
cargo run
```

Open http://localhost:3000.

## Test

```bash
cargo test
```

## Deploy

Railway auto-deploys from `main` via the included `Dockerfile` + `railway.json`.

## Manual smoke-test checklist

- [ ] Page loads with a dark, rotating globe and visible teal markers.
- [ ] Dragging the globe pauses auto-rotation.
- [ ] Hovering a marker shows a label.
- [ ] Clicking a marker turns that company's markers amber, rotates the globe, and opens the right-side drawer.
- [ ] Drawer shows name, description, viewing badge, HQ, reactor, funding, milestone, source link.
- [ ] Clicking a different company's marker crossfades the sidebar.
- [ ] `X`, `Esc`, and clicking empty globe each close the drawer.
- [ ] Mobile viewport (<768px): drawer appears as a bottom sheet.

## Layout

- `src/` — Axum app (`main.rs`, `routes.rs`, `data.rs`, `models.rs`).
- `data/companies.json` — seed dataset, validated at startup.
- `static/` — `index.html`, `styles.css`, `globe.js`.
- `tests/` — Rust integration tests.
```

- [ ] **Step 2: Commit**

```bash
git add README.md
git commit -m "docs: add README with run/test/smoke-test instructions"
```

---

## Task 13: Final end-to-end verification

- [ ] **Step 1: Full local CI parity**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all
```

Expected: all three pass.

- [ ] **Step 2: Full manual smoke-test against `cargo run`**

Run `cargo run`. Work through every item in the README's manual smoke-test checklist. Verify on both desktop and a <768px viewport (browser devtools responsive mode).

- [ ] **Step 3: Docker parity check**

Run:

```bash
docker build -t fusion-tracker .
docker run -p 8080:8080 fusion-tracker
```

Open `http://localhost:8080` and verify the globe loads and a marker click works. Kill the container.

- [ ] **Step 4: Final commit (if any cleanup)**

If the smoke tests surfaced any small fixes, commit them:

```bash
git add -A
git commit -m "fix: <specific issue>"
```

Otherwise nothing to commit.

---

## Notes for the implementer

- **TDD discipline:** every Rust-side behavior gets a test before code. Frontend is verified manually per the checklist.
- **Frequent commits:** one commit per task minimum; per step-group is fine.
- **Don't over-engineer:** the PRD's out-of-scope list and the spec's explicit non-goals are binding. No search, no profile pages, no arcs, no DB.
- **When in doubt, fail loud:** startup-time data validation panics are intentional. Silent fallbacks in v1 would hide data bugs the dataset is explicitly small enough to catch.
