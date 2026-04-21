# Fusion Tracker

Dark, globe-first website showing major fusion projects and facilities around the world. Seed dataset covers commercial-race front-runners plus important / unique science prototypes and international roadmaps.

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

- [ ] Page loads with a dark, rotating globe and visible teal/amber markers.
- [ ] Mode toggle at top shows "Commercial race" (default) and "Important / unique"; flipping it swaps the marker set.
- [ ] Dragging the globe pauses auto-rotation.
- [ ] Hovering a marker shows a label with project and operator.
- [ ] Clicking a marker turns all markers from the same operator amber, rotates the globe, and opens the right-side drawer.
- [ ] Drawer shows project name, operator, credibility/stage badge, location, reactor, funding, target year, offtaker, utility partner, current milestone, source link.
- [ ] Clicking a marker for a different operator crossfades the sidebar.
- [ ] `X`, `Esc`, and clicking empty globe each close the drawer.
- [ ] Mobile viewport (<768px): drawer appears as a bottom sheet.

## Layout

- `src/` — Axum app (`main.rs`, `routes.rs`, `data.rs`, `models.rs`).
- `data/projects.json` — seed dataset, validated at startup.
- `static/` — `index.html`, `styles.css`, `globe.js`.
- `tests/` — Rust integration tests.
