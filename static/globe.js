const NIGHT_LIGHTS_URL = "https://unpkg.com/three-globe@2.31.1/example/img/earth-night.jpg";
const AMBER = "#ffb74d";
const TEAL = "#4dd0e1";
const MUTED = "#8da1bb";
const BASE_RADIUS = 0.4;
const PULSE_RADIUS = 0.75;

const DEFAULT_MODE = "all";

const state = {
  projects: [],
  mode: DEFAULT_MODE,
  selectedCompany: null,
  globe: null,
  autoRotateTimer: null,
  lastMarkerClickAt: 0,
  currentArcs: [],
  allCompanyArcs: [],
  hoveredArcCompany: null,
};

function visibleProjects() {
  if (state.mode === "all") {
    return state.projects;
  }
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
  state.allCompanyArcs = generateAllArcs();
  state.globe.arcsData(state.allCompanyArcs);
  attachSidebarHandlers();
  attachModeToggle();
  attachEnrichHandlers();
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

function getArcColor(arc) {
  if (!state.hoveredArcCompany && !state.selectedCompany) {
    return `rgba(77, 208, 225, 0.4)`;
  }
  if (state.hoveredArcCompany === arc.company) {
    return `rgba(77, 208, 225, 0.9)`;
  }
  if (state.selectedCompany === arc.company) {
    return `rgba(255, 183, 77, 0.8)`;
  }
  return `rgba(77, 208, 225, 0.15)`;
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
    .onPointClick((d) => handleMarkerClick(d))
    .arcsData([])
    .arcStartLat((d) => d.startLat)
    .arcStartLng((d) => d.startLng)
    .arcEndLat((d) => d.endLat)
    .arcEndLng((d) => d.endLng)
    .arcColor(getArcColor)
    .arcAltitude((d) => (state.hoveredArcCompany === d.company || state.selectedCompany === d.company) ? 0.35 : 0.25)
    .arcDashInitialGap(() => 1)
    .arcDashAnimateTime(1500)
    .onArcHover((arc) => {
      state.hoveredArcCompany = arc?.company || null;
      state.globe.arcsData(state.allCompanyArcs);
      updateArcTooltip(arc);
    });

  globe.controls().autoRotate = true;
  globe.controls().autoRotateSpeed = 0.3;
  globe.controls().addEventListener("start", () => pauseAutoRotate());

  state.globe = globe;
}

function generateAllArcs() {
  const companies = new Set(state.projects.map((p) => p.company_or_operator));
  const arcs = [];

  companies.forEach((company) => {
    const companyProjects = state.projects.filter(
      (p) => p.company_or_operator === company
    );

    if (companyProjects.length >= 2) {
      for (let i = 0; i < companyProjects.length; i++) {
        for (let j = i + 1; j < companyProjects.length; j++) {
          const fromProject = companyProjects[i];
          const toProject = companyProjects[j];

          if (fromProject.latitude != null && fromProject.longitude != null &&
              toProject.latitude != null && toProject.longitude != null) {
            arcs.push({
              startLat: fromProject.latitude,
              startLng: fromProject.longitude,
              endLat: toProject.latitude,
              endLng: toProject.longitude,
              company: company,
            });
          }
        }
      }
    }
  });

  return arcs;
}

function generateArcsForCompany(company) {
  return state.allCompanyArcs.filter((arc) => arc.company === company);
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

  state.globe.arcsData(state.allCompanyArcs);

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
  state.hoveredArcCompany = null;
  if (state.globe) {
    state.globe.pointColor(markerColor);
    state.globe.arcsData(state.allCompanyArcs);
  }
  hideArcTooltip();
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
      state.hoveredArcCompany = null;
      if (!document.getElementById("sidebar").hidden) closeSidebar();
      if (state.globe) {
        state.globe.pointsData(visibleProjects());
        state.allCompanyArcs = generateAllArcs();
        state.globe.arcsData(state.allCompanyArcs);
      }
      hideArcTooltip();
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

function updateArcTooltip(arc) {
  if (!arc) {
    hideArcTooltip();
    return;
  }

  let tooltip = document.getElementById("arc-tooltip");
  if (!tooltip) {
    tooltip = document.createElement("div");
    tooltip.id = "arc-tooltip";
    tooltip.className = "arc-tooltip";
    document.body.appendChild(tooltip);
  }

  const projectCount = state.projects.filter(
    (p) => p.company_or_operator === arc.company
  ).length;

  tooltip.innerHTML = `
    <div class="arc-tooltip-content">
      <strong>${escapeHtml(arc.company)}</strong>
      <span class="arc-tooltip-count">${projectCount} location${projectCount !== 1 ? 's' : ''}</span>
    </div>
  `;
  tooltip.style.display = "block";
}

function hideArcTooltip() {
  const tooltip = document.getElementById("arc-tooltip");
  if (tooltip) {
    tooltip.style.display = "none";
  }
}

function attachEnrichHandlers() {
  const modal = document.getElementById("enrich-modal");
  const addBtn = document.getElementById("add-company-btn");
  const closeBtn = modal.querySelector(".modal-close");
  const inputForm = document.getElementById("enrich-input-form");
  const reviewForm = document.getElementById("enrich-review-form");
  const loadingDiv = document.getElementById("enrich-loading");
  const companyInput = document.getElementById("enrich-company-input");
  const submitBtn = document.getElementById("enrich-submit-btn");
  const saveBtn = document.getElementById("enrich-save-btn");

  addBtn.addEventListener("click", () => {
    modal.hidden = false;
    inputForm.hidden = false;
    reviewForm.hidden = true;
    loadingDiv.hidden = true;
    companyInput.value = "";
    clearEnrichErrors();
    companyInput.focus();
  });

  closeBtn.addEventListener("click", closeEnrichModal);
  modal.querySelector(".modal-overlay").addEventListener("click", closeEnrichModal);

  companyInput.addEventListener("keydown", (e) => {
    if (e.key === "Enter") submitBtn.click();
  });

  submitBtn.addEventListener("click", async () => {
    const company = companyInput.value.trim();
    if (!company) {
      showInlineError("enrich-input-form", "Please enter a company name");
      return;
    }

    submitBtn.disabled = true;
    clearEnrichErrors();
    inputForm.hidden = true;
    loadingDiv.hidden = false;

    try {
      const res = await fetch("/api/enrich", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ company_name: company }),
      });

      if (!res.ok) {
        const err = await res.json();
        submitBtn.disabled = false;
        inputForm.hidden = false;
        loadingDiv.hidden = true;
        showInlineError("enrich-input-form", `${err.error || "Failed to enrich"}`);
        return;
      }

      const project = await res.json();
      populateReviewForm(project);
      loadingDiv.hidden = true;
      reviewForm.hidden = false;
    } catch (err) {
      console.error("enrich error:", err);
      submitBtn.disabled = false;
      inputForm.hidden = false;
      loadingDiv.hidden = true;
      showInlineError("enrich-input-form", "Failed to enrich: " + err.message);
    }
  });

  saveBtn.addEventListener("click", async () => {
    const project = collectReviewForm();
    saveBtn.disabled = true;
    clearEnrichErrors();

    try {
      const res = await fetch("/api/projects", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(project),
      });

      if (res.status === 409) {
        saveBtn.disabled = false;
        showInlineError("enrich-review-form", "A project with this ID already exists");
        return;
      }

      if (!res.ok) {
        const err = await res.json().catch(() => ({ error: "Unknown error" }));
        saveBtn.disabled = false;
        showInlineError("enrich-review-form", `${err.error || "Failed to save project"}`);
        return;
      }

      showToast("Project added to map!");
      await refreshGlobeData();
      closeEnrichModal();
    } catch (err) {
      console.error("save error:", err);
      saveBtn.disabled = false;
      showInlineError("enrich-review-form", "Failed to save: " + err.message);
    }
  });
}

function showInlineError(formId, message) {
  let errorDiv = document.getElementById("enrich-error-msg");
  if (!errorDiv) {
    errorDiv = document.createElement("div");
    errorDiv.id = "enrich-error-msg";
    errorDiv.className = "enrich-error";
    const form = document.getElementById(formId);
    form.insertBefore(errorDiv, form.firstChild);
  }
  errorDiv.textContent = message;
  errorDiv.hidden = false;
}

function clearEnrichErrors() {
  const errorDiv = document.getElementById("enrich-error-msg");
  if (errorDiv) {
    errorDiv.hidden = true;
  }
}

async function refreshGlobeData() {
  try {
    const res = await fetch("/api/projects");
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    const body = await res.json();
    state.projects = body.projects ?? [];
    if (state.globe) {
      state.globe.pointsData(visibleProjects());
      state.allCompanyArcs = generateAllArcs();
      state.globe.arcsData(state.allCompanyArcs);
    }
  } catch (err) {
    console.error("failed to refresh projects", err);
    showToast("Warning: globe may not reflect new project");
  }
}

function closeEnrichModal() {
  const modal = document.getElementById("enrich-modal");
  modal.hidden = true;
}

function populateReviewForm(project) {
  const inputs = document.querySelectorAll("#enrich-review-form [data-field]");
  inputs.forEach((input) => {
    const field = input.dataset.field;
    const value = project[field];
    input.value = value !== null && value !== undefined ? value : "";
  });
}

function collectReviewForm() {
  const inputs = document.querySelectorAll("#enrich-review-form [data-field]");
  const project = {
    globe_modes: ["hq"],
  };

  inputs.forEach((input) => {
    const field = input.dataset.field;
    let value = input.value.trim();

    if (!value) {
      project[field] = null;
      return;
    }

    if (field === "latitude" || field === "longitude" || field === "intended_output_mwe") {
      const num = parseFloat(value);
      project[field] = isNaN(num) ? null : num;
    } else if (field === "funding_raised_usd" || field === "target_online_year") {
      const num = parseInt(value, 10);
      project[field] = isNaN(num) ? null : num;
    } else {
      project[field] = value;
    }
  });

  return project;
}

main();
