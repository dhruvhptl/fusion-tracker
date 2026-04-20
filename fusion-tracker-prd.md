# Fusion Tracker PRD

## Overview
Fusion Tracker is a dark, globe-first website that helps users explore major fusion companies and facilities around the world through an interactive 3D globe.[cite:27][cite:92] The initial release is a visual demo focused on companies listed in TechCrunch's April 2026 roundup of fusion startups that have raised over $100M, with room to expand the dataset later.[cite:27]

## Product goal
The product should make the global fusion landscape feel tangible by letting users rotate the globe, discover company headquarters and reactor/demo facilities, and open a sidebar with concise company information.[cite:27][cite:92] The first release is intentionally not a full technical fusion intelligence platform; it is a visual tracker that can later grow into one.[cite:27][cite:16]

## Target user
The primary user is a curious technical reader who wants to quickly see where major fusion companies are located and what each company is building.[cite:27] The site should also be understandable to users who are still learning the field, since fusion milestones and terms like breakeven or \(Q\) can be confusing without context.[cite:16][cite:18]

## Scope
### In scope for v1
- Interactive 3D globe as the homepage.[cite:92]
- Dark, space-inspired visual style with subtle ambient motion.[cite:117][cite:121]
- Slow auto-rotation on idle to signal interactivity, with rotation pausing on user interaction.[cite:130][cite:135]
- Markers for both company headquarters and reactor/demo facilities when known.[cite:27][cite:110][cite:111]
- Hover labels for markers.[cite:92][cite:175]
- Click interaction that opens a slide-in sidebar.[cite:92][cite:201]
- When one company marker is clicked, all markers tied to that company highlight together.[cite:51][cite:92]
- Sidebar content showing company information anchored to the clicked location.[cite:27]
- Data limited to the companies covered in the TechCrunch article for the first version.[cite:27]

### Out of scope for v1
- Search bar.[cite:92]
- Dedicated profile pages for each company.[cite:27]
- Arc connections between HQ and facilities on the globe.[cite:123]
- Full milestone taxonomy or authoritative tracking of who reaches \(Q>1\) first.[cite:16][cite:18]
- User accounts, admin dashboard, or CMS.[cite:31]

## User experience
### Primary flow
1. The user lands on the homepage and sees a slowly rotating dark globe with glowing markers.[cite:117][cite:130]
2. The user drags the globe to explore regions and hovers over a marker to see a lightweight label.[cite:92][cite:175]
3. The user clicks a marker.[cite:92]
4. A right-side drawer slides in with company information for that selected location, and all locations tied to that company light up on the globe.[cite:51][cite:201]

### Interaction principles
The globe should remain the main focus of the page, while the sidebar acts as a secondary layer for detail rather than a competing layout column.[cite:201][cite:197] Labels should appear only on hover or click so the interface stays uncluttered even when multiple HQ and facility markers are visible.[cite:92][cite:175]

## Information architecture
### Entities
The data model should separate companies from locations so one company can own multiple points on the globe.[cite:27][cite:111]

| Entity | Purpose | Core fields |
|---|---|---|
| Company | Top-level company record | `id`, `name`, `description`, `reactor_type`, `funding_raised`, `source_url` [cite:27] |
| Location | Map point tied to a company | `id`, `company_id`, `name`, `location_type`, `city`, `country`, `lat`, `lng`, `status` [cite:110][cite:111] |

### Location types
Locations should support at least two types: `hq` and `facility`.[cite:27][cite:110] A `status` field should distinguish active, planned, or announced locations when that is relevant.[cite:111]

## Sidebar content
The sidebar should stay concise and readable. Each card should include:
- Company name.[cite:27]
- One-line description.[cite:27]
- Headquarters location.[cite:27]
- Selected location name and whether it is HQ or facility.[cite:110][cite:111]
- Reactor or fusion approach.[cite:27]
- Total funding raised.[cite:27]
- Current notable project or milestone, described briefly and carefully.[cite:27][cite:16]
- Source link to the article or supporting source.[cite:27]

## Visual direction
The visual direction should be dark and space-inspired rather than a neutral dashboard.[cite:117][cite:121] The globe should use a near-black background, subtle atmosphere glow, restrained Earth styling, bright teal or cyan markers, and a slightly warmer selected state to distinguish an active company cluster.[cite:117][cite:121]

The sidebar should feel like a glassy overlay drawer with blur, soft border contrast, and a smooth slide animation of roughly 200 to 300 milliseconds.[cite:200][cite:201] The overall tone should feel closer to an observatory interface than a flashy sci-fi landing page.[cite:157]

## Technical approach
The site should use Axum as the backend, serving HTML, CSS, JavaScript, and JSON data endpoints from a Rust application.[cite:31][cite:39] The 3D globe should run in the browser with `globe.gl`, which supports point layers, labels, and click interactions while remaining compatible with a Rust backend because rendering happens client-side.[cite:51][cite:92]

### Proposed stack
| Layer | Choice | Rationale |
|---|---|---|
| Backend | Axum | Rust-native web stack aligned with deployment preferences.[cite:31][cite:39] |
| Frontend | HTML, CSS, vanilla JavaScript | Keeps the app light and avoids a Node-based app architecture.[cite:39][cite:61] |
| Globe | `globe.gl` | Fastest route to an interactive 3D globe with point interactions.[cite:51][cite:92] |
| Data storage | Static JSON for v1, SQLite later if needed | Small initial dataset and minimal complexity.[cite:27][cite:31] |
| Deployment | Rust-hosted website on Railway or VPS | Fits Axum deployment paths without relying on Vercel.[cite:35][cite:34] |

## Functional requirements
- The homepage must render a 3D globe on first load.[cite:92]
- The globe must auto-rotate slowly on idle and pause when the user interacts.[cite:130][cite:133]
- The globe must display markers for both HQ and facility locations.[cite:110][cite:111]
- Marker labels must appear only on hover or click.[cite:92][cite:175]
- Clicking a marker must open a right-side drawer.[cite:201]
- Clicking a marker must also highlight all markers tied to the same company.[cite:51][cite:92]
- The drawer must show company information plus the selected location context.[cite:27][cite:111]
- The drawer must be closable and hidden from keyboard focus when closed for accessibility.[cite:195][cite:196]

## Non-functional requirements
- The site should feel visually polished on desktop first, while remaining usable on mobile.[cite:201][cite:204]
- The globe should remain readable and uncluttered, which is why labels are not always visible.[cite:92][cite:175]
- The initial version should optimize for smooth interaction over dataset breadth.[cite:53][cite:185]
- The implementation should preserve a simple architecture that can later absorb more companies and richer metadata without rewriting the core model.[cite:27][cite:31]

## Success criteria
The v1 release is successful if a user can land on the site, recognize that the globe is interactive, explore multiple regions, click on companies or facilities, and understand what each company is from the sidebar without needing additional navigation.[cite:130][cite:201] It is also successful if the codebase remains simple enough to later add more companies, richer fusion milestone data, and optional arcs between related locations.[cite:123][cite:27]

## Future extensions
Potential next steps after v1:
- Add arc connections between HQ and facilities.[cite:123]
- Add a larger company/lab dataset beyond the TechCrunch list.[cite:27]
- Add search and filtering by reactor type or geography.[cite:27]
- Add dedicated company profile pages.[cite:27]
- Add milestone categories that distinguish scientific breakeven, engineering breakeven, and commercial progress more carefully.[cite:16][cite:18]
