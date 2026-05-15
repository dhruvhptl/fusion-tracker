# Fusion Tracker — Claude Code Context

## What This Project Is
A dark, globe-first website for tracking commercial fusion power projects worldwide. Users rotate an interactive 3D globe, click markers to open a slide-in sidebar with company/project details. Deployed live at https://fusion-tracker.up.railway.app/

## Stack
- **Backend**: Rust + Axum, async with Tokio
- **Frontend**: Vanilla HTML/CSS/JavaScript (no framework, no build step)
- **Globe**: globe.gl (client-side, CDN)
- **Data**: Static JSON file loaded at startup into in-memory state (`Arc<Vec<Project>>`)
- **Deployment**: Railway via Dockerfile

## Project Structure
```
src/
  main.rs        — entrypoint, loads JSON data, starts Axum server
  lib.rs         — lib root
  models.rs      — Project, CredibilityBucket, ProjectLayer, FusionData structs
  routes.rs      — Axum router, SharedState type, existing endpoints
  data.rs        — data loading logic
static/
  index.html     — entire frontend (HTML + CSS + JS in one file)
data/            — JSON dataset files
Dockerfile       — Railway deployment
railway.json     — Railway config
```

## Existing API Routes
- `GET /` — serves static/index.html
- `GET /api/projects` — returns all projects as JSON (`ProjectsResponse`)
- `GET /static/*` — serves static assets

## Data Model (Key Fields on `Project`)
- `id`, `project_name`, `company_or_operator`, `country`, `city_or_region`
- `latitude`, `longitude` — globe marker position
- `reactor_type`, `facility_type`, `project_stage`
- `funding_raised_usd`, `target_online_year`
- `offtaker`, `utility_partner`
- `current_milestone`, `next_milestone`
- `source_url`
- `credibility_bucket` — enum: `FrontRunner | Credible | Speculative | Conceptual`
- `globe_modes` — Vec<String>, controls which tab filter shows this project
- `layer` — enum: `PrimaryProject | PrecursorFacility | ProgramRoadmap | Watchlist`

## Frontend Architecture
- Single `static/index.html` file contains all HTML, CSS, and JavaScript
- globe.gl loaded via CDN, initialized with `pointsData`, `arcsData` layers
- Click handler opens a right-side slide-in drawer/sidebar
- Tab switcher filters globe by `globe_modes` field (`commercial_race` / `important_unique`)

## Conventions
- No Node.js, no npm, no build tools — keep frontend vanilla
- Keep all Rust async, use `?` for error propagation
- Use `anyhow` for error handling where needed
- New API routes go in `routes.rs`, new types go in `models.rs`
- Environment variables for API keys (never hardcode)
- Data mutations are not persisted to disk in the current architecture — state is in-memory

## Planned Features (in order)
1. **Arc connections** — globe.gl `arcsData` layer connecting HQ to facility markers on company select (frontend only)
2. **Auto-populate / enrich** — `POST /api/enrich`: takes company name, calls Tavily search API then OpenRouter LLM to extract structured `Project` fields, returns pre-populated JSON for human review
3. **Credibility scoring** — LLM-generated numeric score + rationale, displayed in sidebar (piggybacks on OpenRouter setup from enrich)
4. **Dataset expansion** — use enrich endpoint to add more companies

## Environment Variables
- `PORT` — set by Railway automatically
- `OPENROUTER_API_KEY` — for LLM calls (OpenRouter)
- `TAVILY_API_KEY` — for web search grounding (Tavily)
