# Fusion Tracker

Dark, globe-first interactive map of global fusion energy projects. Rotate the globe to explore 53+ commercial and research fusion companies and facilities worldwide. Click any marker to view project details: funding, reactor type, milestones, partners, and more.

**Live at:** https://fusion-tracker.up.railway.app/

## Features

- **Interactive 3D globe** — Drag to rotate, hover to explore. Auto-rotates on idle, pauses on interaction.
- **53+ fusion projects** — Commercial front-runners, research prototypes, and international roadmaps.
- **Three filter modes:**
  - **All** — Every tracked project
  - **Commercial race** — Companies pursuing near-term commercialization
  - **Important / unique** — Key research facilities and novel approaches
- **Multi-location tracking** — Markers for company headquarters and reactor facilities when known. Arc connections between related locations (coming soon).
- **Rich project sidebar** — Click any marker to open a details pane showing company name, funding raised, reactor type, current milestone, offtaker partnerships, target online year, and source link.
- **Add new companies** — Click the **+** button to enrich and add new projects via the `/api/enrich` endpoint (requires `OPENROUTER_API_KEY`).
- **Mobile-friendly** — Responsive design; sidebar becomes a bottom sheet on smaller screens.

## Stack

| Layer | Technology |
|-------|-----------|
| Backend | Rust / [Axum](https://github.com/tokio-rs/axum) |
| Frontend | HTML, CSS, vanilla JavaScript |
| Globe | [globe.gl](https://globe.gl/) (via CDN) |
| Data enrichment | OpenRouter API (LLM) + Tavily API (web search) |
| Deployment | Railway (Docker) |

## Data Model

Each project (`data/projects.json`) includes:
- **Location:** `latitude`, `longitude`, `country`, `city_or_region`
- **Identity:** `project_name`, `company_or_operator`, `reactor_type`, `facility_type`
- **Finance:** `funding_raised_usd`, `target_online_year`
- **Status:** `project_stage`, `credibility_bucket` (FrontRunner / Credible / Speculative / Conceptual)
- **Business:** `offtaker`, `utility_partner`
- **Progress:** `current_milestone`, `next_milestone`
- **Discovery:** `source_url`
- **Display:** `globe_modes` (controls filter tab visibility), `layer` (PrimaryProject / PrecursorFacility / ProgramRoadmap / Watchlist)

## Running Locally

### Prerequisites
- Rust 1.70+
- Environment variables (optional for basic browsing):
  ```bash
  OPENROUTER_API_KEY=sk-...  # for enrichment endpoint
  TAVILY_API_KEY=tvly-...    # for web search in enrichment
  ```

### Start the dev server
```bash
cargo run
```

Open http://localhost:3000.

### Run tests
```bash
cargo test
```

## Deployment

Railway auto-deploys from `main` using the included `Dockerfile` and `railway.json`.

```bash
# Manual Railway deployment (if needed)
railway up
```

The Docker image:
- Builds the Rust binary in a builder stage (`rust:latest`)
- Runs on `debian:bookworm-slim` with only the compiled binary, static assets, and data
- Exposes port 8080 (configurable via `PORT` env var)

## Project Structure

```
src/
  main.rs         # App entry point, server setup, data loading
  lib.rs          # Library root
  models.rs       # Project, CredibilityBucket, Layer, FusionData types
  routes.rs       # Axum router and request handlers
  data.rs         # JSON loading and validation
  bin/
    bulk_enrich.rs # CLI tool to enrich multiple projects at once

static/
  index.html      # All frontend HTML + inline CSS + JavaScript
  styles.css      # Global styles (dark theme, flexbox layout, animations)
  globe.js        # Globe initialization, interaction handlers, API calls

data/
  projects.json   # Main dataset (81 projects total, 55 visible)
  [other .json]   # Additional layer data files

tests/            # Rust integration tests

Dockerfile        # Multi-stage build for Railway
railway.json      # Railway deployment config
Cargo.toml        # Rust dependencies
Cargo.lock        # Pinned versions
```

## API Endpoints

| Method | Endpoint | Purpose |
|--------|----------|---------|
| `GET` | `/` | Serves `static/index.html` |
| `GET` | `/api/projects` | Returns all visible projects as JSON |
| `POST` | `/api/enrich` | Takes `{ company_name: string }`, calls LLM + search, returns enriched project fields |
| `POST` | `/api/projects` | Saves a new project; returns 409 Conflict if duplicate |
| `GET` | `/static/*` | Serves static assets |

## Enrichment Workflow

The `/api/enrich` endpoint powers the **+** button to add new companies:

1. Frontend sends company name to `POST /api/enrich`
2. Backend calls **Tavily API** for web search
3. Backend calls **OpenRouter LLM** to extract structured fields from search results
4. Frontend displays pre-populated form for human review
5. User submits; new project is saved to `data/projects.json`

For bulk enrichment:

```bash
OPENROUTER_API_KEY=sk-... TAVILY_API_KEY=tvly-... \
cargo run --bin bulk_enrich
```

## Roadmap

### Completed
- ✅ Interactive globe with multi-location tracking
- ✅ Filter tabs (All, Commercial race, Important / unique)
- ✅ Sidebar details panel with project information
- ✅ Auto-rotation with pause-on-interaction
- ✅ Mobile bottom-sheet drawer
- ✅ API-driven enrichment and data updates
- ✅ Dark, space-inspired visual design

### Planned
- Arc connections between HQ and facility markers (visual grouping)
- Search and geographic filtering
- Dedicated company profile pages
- Milestone taxonomy (breakeven, engineering targets, commercialization)
- Expanded dataset beyond initial TechCrunch list

## Manual Testing Checklist

- [ ] Page loads with dark, rotating globe and glowing teal/amber markers
- [ ] Filter toggle at top shows All (default), Commercial race, Important / unique; switching swaps visible markers
- [ ] Dragging globe pauses auto-rotation
- [ ] Hovering a marker shows project/operator label
- [ ] Clicking a marker highlights all locations for that company, rotates globe, opens right sidebar with details
- [ ] Sidebar shows: project name, operator, credibility badge, location, reactor type, funding, target year, offtaker, partner, milestone, source link
- [ ] Clicking a different marker crossfades sidebar content
- [ ] `X`, `Esc`, or clicking empty globe closes drawer without losing position
- [ ] Mobile (<768px): drawer appears as bottom sheet, landscape rotation works
- [ ] **+** button opens enrichment modal
- [ ] Enrichment modal accepts company name and shows loading state
- [ ] On success, new project appears in dataset and globe

## Troubleshooting

**Markers not showing?**
- Ensure `data/projects.json` has valid coordinates and non-empty `globe_modes`
- Check browser console for JavaScript errors
- Verify `/api/projects` returns data: `curl http://localhost:3000/api/projects`

**Enrichment failing?**
- Check `OPENROUTER_API_KEY` and `TAVILY_API_KEY` are set
- Monitor server logs for API rate limits or network errors
- Verify project name is a real fusion company

**Sidebar not opening?**
- Ensure marker coordinates are in valid range: [-90, 90] lat, [-180, 180] lon
- Check browser console for click handler errors

## License

[Add license info if applicable]
