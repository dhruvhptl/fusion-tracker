# Fusion Tracker — Design Spec

**Date:** 2026-04-19
**Source PRD:** `fusion-tracker-prd.md`
**Data source:** TechCrunch, "Every fusion startup that has raised over $100M" (April 2026)

## Summary

Fusion Tracker is a dark, globe-first website that lets users explore the 15 fusion companies in the TechCrunch April 2026 roundup through an interactive 3D globe. Clicking a marker opens a slide-in sidebar with company detail and highlights every location tied to that company. The stack is Axum (Rust) serving static HTML/CSS/JS plus a single JSON data endpoint, deployed to Railway with GitHub Actions running tests on push.

This spec covers v1 scope only. Out-of-scope items from the PRD (search, profile pages, HQ↔facility arcs, milestone taxonomy, accounts, CMS) are not part of this implementation.

## Architecture

### Stack
- **Backend:** Axum (Rust), serves static assets + one JSON endpoint.
- **Frontend:** Vanilla HTML/CSS/JS, no bundler, no framework.
- **Globe:** `globe.gl` loaded from CDN.
- **Data:** Static `data/companies.json` loaded into memory at startup.
- **Deployment:** Railway via Dockerfile; CI via GitHub Actions.

### File layout

```
fusiontracker/
├── Cargo.toml
├── Dockerfile
├── railway.json
├── .gitignore
├── README.md
├── .github/workflows/ci.yml
├── src/
│   ├── main.rs             # Axum bootstrap, reads $PORT
│   ├── routes.rs           # GET /, GET /static/*, GET /api/companies
│   ├── data.rs             # loads + validates data/companies.json at startup
│   └── models.rs           # Company, Location structs with serde
├── data/
│   └── companies.json
└── static/
    ├── index.html
    ├── styles.css
    └── globe.js
```

### Backend responsibilities

1. At startup, read and parse `data/companies.json`. Validate (see Data Integrity below). If anything fails, panic with a clear message — the app must not start with bad data.
2. Hold the validated dataset in an `Arc<Dataset>` inside Axum app state.
3. Serve three routes:
   - `GET /` → `static/index.html`
   - `GET /static/*` → files from the `static/` directory
   - `GET /api/companies` → the cached dataset as JSON
4. Bind to the port from `$PORT` env var (Railway convention), default `3000` locally.

### Frontend responsibilities

1. `index.html` is a minimal shell: `<div id="globe">`, `<aside id="sidebar" hidden role="dialog">`, and a `<script src="/static/globe.js">`.
2. `globe.js` on load:
   - Fetches `/api/companies`.
   - Initializes globe.gl with NASA night-lights texture.
   - Renders markers with HQ/facility altitude differentiation.
   - Wires hover, click, drag, and sidebar handlers.
3. On fetch failure: renders empty globe + a small top-right toast "Couldn't load companies. Refresh to retry."

## Data model

### companies.json shape

```json
{
  "companies": [
    {
      "id": "string (stable slug, e.g. 'cfs')",
      "name": "string",
      "description": "string (one sentence)",
      "reactor_type": "string",
      "funding_raised_usd": 0,
      "funding_display": "string (human-readable, e.g. '~$3B', '€185M')",
      "current_milestone": "string",
      "source_url": "string (full URL)"
    }
  ],
  "locations": [
    {
      "id": "string (stable slug, e.g. 'cfs-sparc')",
      "company_id": "string (references companies.id)",
      "name": "string",
      "location_type": "hq | facility",
      "city": "string",
      "country": "string",
      "lat": 0.0,
      "lng": 0.0,
      "status": "active | planned | announced"
    }
  ]
}
```

### Rust structs (`models.rs`)

```rust
#[derive(Serialize, Deserialize, Clone)]
pub struct Dataset {
    pub companies: Vec<Company>,
    pub locations: Vec<Location>,
}

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize, Clone)]
pub struct Location {
    pub id: String,
    pub company_id: String,
    pub name: String,
    pub location_type: LocationType,   // enum with serde rename_all = "lowercase"
    pub city: String,
    pub country: String,
    pub lat: f64,
    pub lng: f64,
    pub status: LocationStatus,        // enum with serde rename_all = "lowercase"
}
```

### Data integrity (enforced at startup)

- Every `location.company_id` references a real `company.id`.
- Every `company.id` has at least one `location`.
- `lat` in `[-90, 90]`, `lng` in `[-180, 180]`.
- `id` values are unique within their collection.

Violations panic at startup with the offending `id`.

### Seed data

Seeded from the 15 companies in the TechCrunch April 2026 roundup: Commonwealth Fusion Systems, TAE Technologies, Helion, Pacific Fusion, Shine Technologies, General Fusion, Inertia Enterprises, Tokamak Energy, Zap Energy, Type One Energy, Proxima Fusion, Kyoto Fusioneering, Marvel Fusion, First Light Fusion, Xcimer.

For companies with named facilities in the article (CFS Sparc + Arc, Helion Polaris, Tokamak Energy ST40 + Demo 4, General Fusion LM26, Inertia + Marvel Fusion demo at Colorado State, Type One at TVA Tennessee site), separate `facility` location records are created. Colocated HQ+facility records share lat/lng; the rendering layer handles the visual offset.

For companies without a specific city in the article, lat/lng is picked for a representative HQ city (verified against public sources during data entry).

## Visual direction

### Globe
- NASA night-lights Earth texture on a near-black background.
- Subtle atmosphere glow.
- Auto-rotation at ~0.3°/sec when idle.

### Markers
- **HQ:** solid dot on globe surface, teal `#4DD0E1`, base radius ~0.5 (globe.gl units).
- **Facility:** same solid dot, same teal, same radius, but altitude offset of ~0.015 so it reads as floating slightly above the surface.
- **Selected (any marker for the clicked company):** warm amber `#FFB74D`, brief single-pulse on transition.
- Hover: cursor → pointer, label appears above the dot as `{location.name} — {company.name}`.

### Sidebar
- Right-side drawer, ~360px wide on desktop.
- Glassy overlay: backdrop-blur, soft border, ~85% opacity dark fill.
- Slide animation: 250ms ease-out on open, 250ms ease-in on close.
- Bottom-sheet variant on mobile (<768px): covers ~60% of viewport height from the bottom.

### Sidebar content (top to bottom)

1. Close button (top-right, X icon).
2. Company name (large).
3. One-line description.
4. Selected-location badge: `Viewing: {location.name} · {HQ | Facility} ({status})`.
5. **Headquarters:** `{city}, {country}` (always the HQ location, even when a facility is selected).
6. **Reactor / approach:** `{reactor_type}`.
7. **Funding raised:** `{funding_display}`.
8. **Current milestone:** `{current_milestone}`.
9. **Source:** link to `source_url` with text "Read more →".

## Interactions

### Idle
- Globe auto-rotates at ~0.3°/sec.
- All markers in default teal.

### Drag globe
- Auto-rotation pauses immediately.
- Resumes after 5 seconds of no interaction.

### Hover marker
- Cursor → pointer.
- Label appears: `{location.name} — {company.name}`.
- No other markers change.

### Click marker
1. Auto-rotation stops (until sidebar closes).
2. Globe smoothly rotates over ~800ms so the clicked marker sits at ~35% from the left edge of the viewport.
3. All markers belonging to the clicked company transition to selected amber with a one-shot pulse.
4. Sidebar slides in from the right over 250ms with content populated for the clicked location's company.

### Click a different marker while sidebar open
- Sidebar content crossfades in place over ~150ms.
- Globe re-centers on the new marker (~800ms).
- Highlighted company updates (previous company's markers return to teal, new company's markers go amber).
- No close-then-open flicker.

### Close sidebar
- Via X button, Esc key, or click onto the globe outside any marker.
- Sidebar slides out (250ms).
- Markers return to default teal.
- Auto-rotation resumes after ~500ms.

## Accessibility

- Sidebar `<aside>` has `role="dialog"` and `aria-labelledby` pointing at the company-name element.
- When hidden: `hidden` attribute set, all focusables inside get `tabindex="-1"` (or skip by virtue of `hidden`) — keyboard tab order does not land in the sidebar.
- When opened: focus moves to the close button.
- Close button has `aria-label="Close company details"`.
- Esc key closes sidebar from anywhere.
- Marker labels on hover are decorative; they don't need ARIA since click handlers on the markers are the actionable affordance.

## Error handling

- **`data/companies.json` missing or unreadable at startup:** panic with path.
- **Malformed JSON:** serde error propagated in startup panic.
- **Validation failure** (see Data integrity): panic naming the offending `id`.
- **Frontend fetch of `/api/companies` fails:** render empty globe + top-right toast: "Couldn't load companies. Refresh to retry." No automatic retry, no offline mode.
- **Unknown marker clicked** (defensive, shouldn't happen): log to console, no sidebar.

## Testing

### Rust (`cargo test`)
- `data.rs` loads and validates `data/companies.json` successfully.
- `data.rs` panics on: missing file, malformed JSON, orphan `company_id`, out-of-range lat/lng, duplicate ids, company with no locations.
- `/api/companies` endpoint returns the expected shape (tested via Axum's `TestServer` or `tower::ServiceExt::oneshot`).

### Frontend
- No automated tests for v1.
- Manual smoke-test checklist in `README.md`: load page, drag globe, hover marker, click marker, verify sidebar content, close via X/Esc/globe-click, click second marker with sidebar open, mobile viewport check.

## Deployment

### Dockerfile
- Multi-stage: `rust:1-slim` builder, `debian:bookworm-slim` runtime.
- Builder stage: `cargo build --release`.
- Runtime stage: copy binary, `static/`, and `data/` directories.
- Expose port from `$PORT` env var.

### railway.json
- Builder: `DOCKERFILE`.
- Healthcheck: `GET /api/companies`.

### CI (`.github/workflows/ci.yml`)
- Runs on push and pull request.
- Steps: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`.
- No deploy step (Railway auto-deploys from main).

### Local dev
- `cargo run` starts the server on `localhost:3000`.
- No watch/reload tooling — Rust compile cycle is short enough for this codebase size.

## Explicit non-goals for v1

- Search bar.
- Dedicated company profile pages (the sidebar is the only detail view).
- Arc connections between HQ and facilities on the globe.
- Fusion milestone taxonomy (breakeven, Q>1, etc.) beyond the single `current_milestone` string.
- User accounts, admin UI, CMS.
- SQLite or any database.
- Analytics or error reporting service.
- Rate limiting or auth on `/api/companies`.
- Automated frontend tests.
- Dev watch/reload tooling.

## Open seams for future extensions

- `Dataset` can be swapped from static JSON to SQLite without changing `models.rs` or the frontend — only `data.rs` changes.
- Adding new location types (e.g. `research_lab`, `partnership`) is a matter of extending the `LocationType` enum and the marker-rendering switch.
- Adding arcs is additive: a `connections` array with `from_location_id` / `to_location_id` and a globe.gl arcs layer.
