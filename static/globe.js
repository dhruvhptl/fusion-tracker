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
  lastMarkerClickAt: 0,
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
  state.lastMarkerClickAt = Date.now();
  const company = findCompany(location.company_id);
  if (!company) {
    console.warn("click on unknown company_id", location.company_id);
    return;
  }
  const previousId = state.selectedCompanyId;
  state.selectedCompanyId = company.id;

  state.globe.pointColor((d) =>
    state.selectedCompanyId === d.company_id ? AMBER : TEAL
  );

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

function centerOnLocation(location) {
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
  document.getElementById("globe").addEventListener("click", () => {
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
