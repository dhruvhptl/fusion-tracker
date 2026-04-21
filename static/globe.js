const NIGHT_LIGHTS_URL = "https://unpkg.com/three-globe@2.31.1/example/img/earth-night.jpg";
const AMBER = "#ffb74d";
const TEAL = "#4dd0e1";
const MUTED = "#8da1bb";
const BASE_RADIUS = 0.4;
const PULSE_RADIUS = 0.75;

const DEFAULT_MODE = "commercial_race";

const state = {
  projects: [],
  mode: DEFAULT_MODE,
  selectedCompany: null,
  globe: null,
  autoRotateTimer: null,
  lastMarkerClickAt: 0,
};

function visibleProjects() {
  return state.projects.filter((p) => (p.globe_modes || []).includes(state.mode));
}

async function main() {
  try {
    const res = await fetch("/api/projects");
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    const body = await res.json();
    state.projects = body.projects ?? [];
  } catch (err) {
    console.error("failed to load /api/projects", err);
    showToast("Couldn't load projects. Refresh to retry.");
    initGlobe([]);
    return;
  }
  initGlobe(visibleProjects());
  attachSidebarHandlers();
  attachModeToggle();
}

function markerColor(p) {
  if (state.selectedCompany && p.company_or_operator === state.selectedCompany) {
    return AMBER;
  }
  switch (p.credibility_bucket) {
    case "front_runner":
      return AMBER;
    case "credible":
      return TEAL;
    default:
      return MUTED;
  }
}

function initGlobe(projects) {
  const globeEl = document.getElementById("globe");
  const globe = Globe()(globeEl)
    .globeImageUrl(NIGHT_LIGHTS_URL)
    .backgroundColor("#05070d")
    .atmosphereColor("#4dd0e1")
    .atmosphereAltitude(0.15)
    .pointsData(projects)
    .pointLat((d) => d.latitude)
    .pointLng((d) => d.longitude)
    .pointAltitude(0.01)
    .pointRadius(BASE_RADIUS)
    .pointColor(markerColor)
    .pointLabel(
      (d) =>
        `<div class="marker-label">${escapeHtml(d.project_name)} — ${escapeHtml(d.company_or_operator)}</div>`
    )
    .onPointClick((d) => handleMarkerClick(d));

  globe.controls().autoRotate = true;
  globe.controls().autoRotateSpeed = 0.3;
  globe.controls().addEventListener("start", () => pauseAutoRotate());

  state.globe = globe;
}

function handleMarkerClick(project) {
  state.lastMarkerClickAt = Date.now();
  const previousCompany = state.selectedCompany;
  state.selectedCompany = project.company_or_operator;

  state.globe.pointColor(markerColor);

  state.globe.pointRadius((d) =>
    d.company_or_operator === state.selectedCompany ? PULSE_RADIUS : BASE_RADIUS
  );
  setTimeout(() => {
    state.globe.pointRadius(() => BASE_RADIUS);
  }, 400);

  centerOnProject(project);
  const sidebarOpen = !document.getElementById("sidebar").hidden;
  if (sidebarOpen && previousCompany !== project.company_or_operator) {
    crossfadeSidebar(() => populateSidebar(project));
  } else {
    populateSidebar(project);
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

function centerOnProject(project) {
  const lngOffset = window.innerWidth > 768 ? -20 : 0;
  state.globe.pointOfView(
    { lat: project.latitude, lng: project.longitude + lngOffset, altitude: 1.8 },
    800
  );
}

function fmtFunding(usd) {
  if (usd == null) return "—";
  if (usd >= 1_000_000_000) return `$${(usd / 1_000_000_000).toFixed(usd % 1_000_000_000 === 0 ? 0 : 2)}B`;
  if (usd >= 1_000_000) return `$${Math.round(usd / 1_000_000)}M`;
  return `$${usd.toLocaleString()}`;
}

function fmtStage(stage) {
  if (!stage) return "—";
  return stage.replace(/_/g, " ").replace(/\b\w/g, (c) => c.toUpperCase());
}

function fmtOrDash(v) {
  return v == null || v === "" ? "—" : v;
}

function populateSidebar(p) {
  document.getElementById("sidebar-project-name").textContent = p.project_name;
  document.getElementById("sidebar-company").textContent = p.company_or_operator;

  const bucket = p.credibility_bucket ? fmtStage(p.credibility_bucket) : "—";
  const stage = fmtStage(p.project_stage);
  document.getElementById("sidebar-badge").textContent = `${bucket} · ${stage}`;

  document.getElementById("sidebar-location").textContent =
    `${p.city_or_region}, ${p.country}`;
  document.getElementById("sidebar-reactor").textContent = fmtOrDash(p.reactor_type);
  document.getElementById("sidebar-funding").textContent = fmtFunding(p.funding_raised_usd);
  document.getElementById("sidebar-target").textContent = fmtOrDash(p.target_online_year);
  document.getElementById("sidebar-offtaker").textContent = fmtOrDash(p.offtaker);
  document.getElementById("sidebar-utility").textContent = fmtOrDash(p.utility_partner);
  document.getElementById("sidebar-milestone").textContent = fmtOrDash(p.current_milestone);

  const src = document.getElementById("sidebar-source");
  if (p.source_url) {
    src.href = p.source_url;
    src.hidden = false;
  } else {
    src.hidden = true;
  }
}

function openSidebar() {
  const sidebar = document.getElementById("sidebar");
  sidebar.hidden = false;
  setTimeout(() => document.getElementById("sidebar-close").focus(), 50);
}

function closeSidebar() {
  const sidebar = document.getElementById("sidebar");
  sidebar.hidden = true;
  state.selectedCompany = null;
  if (state.globe) {
    state.globe.pointColor(markerColor);
  }
  setTimeout(() => resumeAutoRotate(), 500);
}

function attachModeToggle() {
  const buttons = document.querySelectorAll("#mode-toggle button");
  buttons.forEach((btn) => {
    btn.addEventListener("click", () => {
      const mode = btn.dataset.mode;
      if (mode === state.mode) return;
      state.mode = mode;
      buttons.forEach((b) =>
        b.setAttribute("aria-selected", b.dataset.mode === mode ? "true" : "false")
      );
      state.selectedCompany = null;
      if (!document.getElementById("sidebar").hidden) closeSidebar();
      if (state.globe) state.globe.pointsData(visibleProjects());
    });
  });
}

function attachSidebarHandlers() {
  document.getElementById("sidebar-close").addEventListener("click", closeSidebar);
  document.addEventListener("keydown", (e) => {
    if (e.key === "Escape") closeSidebar();
  });
  document.getElementById("globe").addEventListener(
    "click",
    () => {
      setTimeout(() => {
        if (!state.lastMarkerClickAt || Date.now() - state.lastMarkerClickAt > 100) {
          if (!document.getElementById("sidebar").hidden) closeSidebar();
        }
      }, 0);
    },
    true
  );
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
