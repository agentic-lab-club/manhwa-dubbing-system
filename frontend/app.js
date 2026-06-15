const $ = (selector) => document.querySelector(selector);
const $$ = (selector) => Array.from(document.querySelectorAll(selector));

const state = {
  latestJob: null,
  pollTimer: null,
};

function formPayload() {
  const form = new FormData($("#jobForm"));
  const payload = {};
  for (const [key, value] of form.entries()) {
    if (value !== "") payload[key] = value;
  }
  payload.render = Boolean($("#jobForm [name=render]").checked);
  payload.synthesize_voice = Boolean($("#jobForm [name=synthesize_voice]").checked);
  return payload;
}

async function api(path, options = {}) {
  const response = await fetch(path, {
    headers: { "Content-Type": "application/json", ...(options.headers || {}) },
    ...options,
  });
  const text = await response.text();
  let data;
  try {
    data = text ? JSON.parse(text) : null;
  } catch {
    data = { raw: text };
  }
  if (!response.ok) {
    throw new Error(data?.message || response.statusText);
  }
  return data;
}

function setPill(element, value, status = "muted") {
  element.textContent = value;
  element.className = `pill ${status}`;
}

async function refreshAll() {
  await Promise.allSettled([loadMetrics(), loadMusic()]);
  if (state.latestJob) await loadJob(state.latestJob);
}

async function loadMetrics() {
  try {
    const data = await api("/api/v1/metrics");
    setPill($("#apiState"), "online");
    const caps = data.capabilities || {};
    $("#metricsGrid").innerHTML = [
      metric("Jobs", data.jobs_total ?? 0),
      metric("Latest", data.latest_job || "none"),
      metric("ffmpeg", caps.ffmpeg ? "yes" : "no"),
      metric("tesseract", caps.tesseract ? "yes" : "no"),
      metric("poetry", caps.poetry ? "yes" : "no"),
      metric("Music files", data.music_files ?? 0),
      metric("ML stages", data.latest_artifacts ?? 0),
      metric("Status", data.latest_status || "idle"),
    ].join("");
  } catch (error) {
    setPill($("#apiState"), "offline", "danger");
    $("#metricsGrid").innerHTML = metric("Error", error.message);
  }
}

function metric(label, value) {
  return `<div class="metric"><strong>${escapeHtml(String(value))}</strong><span>${escapeHtml(label)}</span></div>`;
}

async function startJob() {
  setPill($("#jobState"), "starting", "warn");
  $("#jobOutput").textContent = "Starting pipeline...";
  try {
    const result = await api("/api/v1/dubbing/start", {
      method: "POST",
      body: JSON.stringify(formPayload()),
    });
    state.latestJob = result.job_id;
    renderJob(result);
    startPolling();
  } catch (error) {
    setPill($("#jobState"), "failed", "danger");
    $("#jobOutput").textContent = error.message;
  } finally {
    await loadMetrics();
  }
}

function startPolling() {
  clearInterval(state.pollTimer);
  state.pollTimer = setInterval(async () => {
    if (state.latestJob) await loadJob(state.latestJob);
  }, 2500);
}

async function loadJob(jobId) {
  try {
    const data = await api(`/api/v1/status/${encodeURIComponent(jobId)}`);
    renderJob(data);
  } catch {
    clearInterval(state.pollTimer);
  }
}

function renderJob(job) {
  const status = job.status || "unknown";
  setPill($("#jobState"), status, status === "failed" ? "danger" : status === "completed" ? "" : "warn");
  $("#jobOutput").textContent = JSON.stringify(job, null, 2);
  const artifacts = job.artifacts || [];
  $("#artifactList").innerHTML = artifacts
    .map(
      (item) => `
        <div class="artifact">
          <strong>${escapeHtml(item.stage || "")}</strong>
          <span class="pill ${artifactTone(item.status)}">${escapeHtml(item.status || "")}</span>
          <span class="path">${escapeHtml(item.path || item.message || "")}</span>
        </div>
      `,
    )
    .join("");
}

function artifactTone(status) {
  if (status === "failed") return "danger";
  if (status === "fallback" || status === "empty" || status === "ready") return "warn";
  return "";
}

async function loadMusic() {
  const data = await api("/api/v1/music/library");
  const files = data.files || [];
  const tracks = data.catalog?.tracks || [];
  const rows = [
    ...tracks.map((track) => ({
      id: track.id || track.title || "catalog",
      mood: track.mood || "neutral",
      path: track.path || "",
    })),
    ...files.map((path) => ({
      id: path.split(/[\\/]/).pop(),
      mood: "scanned",
      path,
    })),
  ];
  $("#musicList").innerHTML =
    rows.length === 0
      ? `<div class="path">Music library is empty. Add files to assets/music or register a path below.</div>`
      : rows
          .map(
            (row) => `
              <div class="music-row">
                <strong>${escapeHtml(row.id)}</strong>
                <span class="pill">${escapeHtml(row.mood)}</span>
                <span class="path">${escapeHtml(row.path)}</span>
              </div>
            `,
          )
          .join("");
}

async function registerMusic(event) {
  event.preventDefault();
  const form = new FormData($("#musicForm"));
  const payload = Object.fromEntries(form.entries());
  try {
    await api("/api/v1/music/register", {
      method: "POST",
      body: JSON.stringify(payload),
    });
    $("#musicForm").reset();
    await loadMusic();
    await loadMetrics();
  } catch (error) {
    alert(error.message);
  }
}

function escapeHtml(value) {
  return value.replace(/[&<>"']/g, (char) =>
    ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[char],
  );
}

function setupTabs() {
  $$(".tab").forEach((tab) => {
    tab.addEventListener("click", () => {
      $$(".tab").forEach((item) => item.classList.remove("active"));
      $$(".tab-page").forEach((item) => item.classList.remove("active"));
      tab.classList.add("active");
      $(`[data-page="${tab.dataset.tab}"]`).classList.add("active");
    });
  });
}

$("#startBtn").addEventListener("click", startJob);
$("#refreshBtn").addEventListener("click", refreshAll);
$("#reloadMusicBtn").addEventListener("click", loadMusic);
$("#musicForm").addEventListener("submit", registerMusic);
setupTabs();
refreshAll();
