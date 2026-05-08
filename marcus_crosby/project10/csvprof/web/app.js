const encoder = new TextEncoder();
const decoder = new TextDecoder();

const sampleCsv = `neighborhood,active_vacant_notices,crime_events,shooting,updated_at
Downtown,52,7388,false,2026-04-24
Sandtown-Winchester,611,2455,true,2026-04-24
Carrollton Ridge,750,1481,false,2026-04-24
Broadway East,725,1421,false,2026-04-24
Federal Hill,8,372,false,2026-04-24
Mount Vernon,14,998,false,2026-04-24
`;

const els = {
  status: document.querySelector("#status"),
  fileInput: document.querySelector("#fileInput"),
  sampleButton: document.querySelector("#sampleButton"),
  profileButton: document.querySelector("#profileButton"),
  csvInput: document.querySelector("#csvInput"),
  outputFormat: document.querySelector("#outputFormat"),
  delimiter: document.querySelector("#delimiter"),
  hasHeaders: document.querySelector("#hasHeaders"),
  percentiles: document.querySelector("#percentiles"),
  topK: document.querySelector("#topK"),
  sampleSize: document.querySelector("#sampleSize"),
  rowCount: document.querySelector("#rowCount"),
  columnCount: document.querySelector("#columnCount"),
  numericCount: document.querySelector("#numericCount"),
  nullCount: document.querySelector("#nullCount"),
  columnsBody: document.querySelector("#columnsBody"),
  renderedReport: document.querySelector("#renderedReport"),
  jsonReport: document.querySelector("#jsonReport"),
  tabs: {
    columns: document.querySelector("#columnsTab"),
    report: document.querySelector("#reportTab"),
    json: document.querySelector("#jsonTab"),
  },
  panels: {
    columns: document.querySelector("#columnsPanel"),
    report: document.querySelector("#reportPanel"),
    json: document.querySelector("#jsonPanel"),
  },
};

let wasm = null;

init();

async function init() {
  setBusy(true);
  bindEvents();
  els.csvInput.value = sampleCsv;

  try {
    wasm = await loadWasm();
    setStatus("Ready");
    setBusy(false);
    runProfile();
  } catch (error) {
    setStatus("WASM unavailable", true);
    setBusy(false);
    renderError(error.message);
  }
}

function bindEvents() {
  els.sampleButton.addEventListener("click", () => {
    els.csvInput.value = sampleCsv;
    els.fileInput.value = "";
    runProfile();
  });

  els.profileButton.addEventListener("click", runProfile);

  els.fileInput.addEventListener("change", async () => {
    const [file] = els.fileInput.files;
    if (!file) {
      return;
    }
    els.csvInput.value = await file.text();
    runProfile();
  });

  for (const [name, tab] of Object.entries(els.tabs)) {
    tab.addEventListener("click", () => activateTab(name));
  }
}

async function loadWasm() {
  const response = await fetch("pkg/csvprof.wasm");
  if (!response.ok) {
    throw new Error(`failed to fetch WASM bundle (${response.status})`);
  }

  const bytes = await response.arrayBuffer();
  const { instance } = await WebAssembly.instantiate(bytes, {});
  if (!instance.exports.memory) {
    throw new Error("WASM memory export is missing");
  }

  return instance.exports;
}

function runProfile() {
  if (!wasm) {
    return;
  }

  setBusy(true);
  try {
    const response = profileCsv(els.csvInput.value, readOptions());
    if (!response.ok) {
      throw new Error(response.error || "profile failed");
    }
    renderSuccess(response);
    setStatus("Ready");
  } catch (error) {
    setStatus("Profile error", true);
    renderError(error.message);
  } finally {
    setBusy(false);
  }
}

function profileCsv(csvText, options) {
  const input = passString(csvText);
  const opts = passString(JSON.stringify(options));
  let outputPtr = 0;
  let outputLen = 0;

  try {
    outputPtr = wasm.csvprof_profile(input.ptr, input.len, opts.ptr, opts.len);
    outputLen = wasm.csvprof_last_output_len();
    if (!outputPtr && outputLen > 0) {
      throw new Error("WASM profiler returned an empty pointer");
    }

    const output = readString(outputPtr, outputLen);
    return JSON.parse(output);
  } finally {
    wasm.csvprof_free(input.ptr, input.len);
    wasm.csvprof_free(opts.ptr, opts.len);
    if (outputPtr) {
      wasm.csvprof_free(outputPtr, outputLen);
    }
  }
}

function passString(value) {
  const bytes = encoder.encode(value);
  const ptr = wasm.csvprof_alloc(bytes.length);
  if (!ptr && bytes.length > 0) {
    throw new Error("WASM allocation failed");
  }

  new Uint8Array(wasm.memory.buffer, ptr, bytes.length).set(bytes);
  return { ptr, len: bytes.length };
}

function readString(ptr, len) {
  if (len === 0) {
    return "";
  }
  return decoder.decode(new Uint8Array(wasm.memory.buffer, ptr, len));
}

function readOptions() {
  return {
    outputFormat: els.outputFormat.value,
    delimiter: els.delimiter.value || ",",
    hasHeaders: els.hasHeaders.checked,
    percentiles: parsePercentiles(els.percentiles.value),
    topK: readPositiveInteger(els.topK, 5),
    sampleSize: readPositiveInteger(els.sampleSize, 4096),
  };
}

function parsePercentiles(value) {
  if (!value.trim()) {
    return [];
  }

  return value.split(",").map((item) => {
    const parsed = Number(item.trim());
    if (!Number.isFinite(parsed)) {
      throw new Error("percentiles must be numbers");
    }
    return parsed;
  });
}

function readPositiveInteger(input, fallback) {
  const parsed = Number(input.value);
  if (!Number.isFinite(parsed) || parsed <= 0) {
    return fallback;
  }
  return Math.trunc(parsed);
}

function renderSuccess(response) {
  const report = response.report;
  const columns = report.columns || [];
  const numericColumns = columns.filter((column) =>
    ["int", "float"].includes(column.inferred_type)
  );
  const nullCells = columns.reduce((sum, column) => sum + Number(column.null_count || 0), 0);

  els.rowCount.textContent = formatInteger(report.rows);
  els.columnCount.textContent = formatInteger(columns.length);
  els.numericCount.textContent = formatInteger(numericColumns.length);
  els.nullCount.textContent = formatInteger(nullCells);
  els.renderedReport.textContent = response.rendered || "";
  els.jsonReport.textContent = JSON.stringify(report, null, 2);
  renderColumns(columns);
}

function renderColumns(columns) {
  els.columnsBody.replaceChildren();

  if (!columns.length) {
    const row = document.createElement("tr");
    row.className = "empty-row";
    const cell = document.createElement("td");
    cell.colSpan = 9;
    cell.textContent = "No columns";
    row.append(cell);
    els.columnsBody.append(row);
    return;
  }

  for (const column of columns) {
    const row = document.createElement("tr");
    row.append(
      textCell(column.name),
      typeCell(column.inferred_type),
      textCell(formatNulls(column)),
      textCell(formatDistinct(column)),
      textCell(column.min ?? "-"),
      textCell(column.max ?? "-"),
      textCell(formatNumber(column.mean)),
      textCell(formatNumber(column.median)),
      textCell(formatTopValues(column.top_values || []))
    );
    els.columnsBody.append(row);
  }
}

function renderError(message) {
  els.rowCount.textContent = "0";
  els.columnCount.textContent = "0";
  els.numericCount.textContent = "0";
  els.nullCount.textContent = "0";
  els.columnsBody.replaceChildren();
  const row = document.createElement("tr");
  row.className = "empty-row";
  const cell = document.createElement("td");
  cell.colSpan = 9;
  cell.textContent = message;
  row.append(cell);
  els.columnsBody.append(row);
  els.renderedReport.textContent = message;
  els.jsonReport.textContent = JSON.stringify({ ok: false, error: message }, null, 2);
}

function textCell(value) {
  const cell = document.createElement("td");
  cell.textContent = value;
  return cell;
}

function typeCell(type) {
  const cell = document.createElement("td");
  const pill = document.createElement("span");
  pill.className = "type-pill";
  pill.textContent = type || "unknown";
  cell.append(pill);
  return cell;
}

function formatNulls(column) {
  const rows = Number(column.rows || 0);
  const nulls = Number(column.null_count || 0);
  if (!rows) {
    return "0";
  }
  return `${formatInteger(nulls)} (${((nulls / rows) * 100).toFixed(1)}%)`;
}

function formatDistinct(column) {
  if (column.distinct_count == null) {
    return "-";
  }
  return `${column.distinct_is_approximate ? "~" : ""}${formatInteger(column.distinct_count)}`;
}

function formatTopValues(values) {
  if (!values.length) {
    return "-";
  }
  return values
    .map((entry) => `${entry.value || "(empty)"} (${formatInteger(entry.count)})`)
    .join(", ");
}

function formatNumber(value) {
  if (value == null) {
    return "-";
  }
  const number = Number(value);
  if (!Number.isFinite(number)) {
    return "-";
  }
  return Number.isInteger(number) ? formatInteger(number) : number.toFixed(4);
}

function formatInteger(value) {
  return new Intl.NumberFormat().format(Number(value || 0));
}

function activateTab(activeName) {
  for (const [name, tab] of Object.entries(els.tabs)) {
    const active = name === activeName;
    tab.classList.toggle("active", active);
    tab.setAttribute("aria-selected", String(active));
    els.panels[name].hidden = !active;
  }
}

function setBusy(isBusy) {
  els.profileButton.disabled = isBusy;
}

function setStatus(message, isError = false) {
  els.status.textContent = message;
  els.status.classList.toggle("error", isError);
}
