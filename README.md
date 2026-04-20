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
