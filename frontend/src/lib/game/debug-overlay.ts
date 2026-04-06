const DEBUG_OVERLAY_ID = 'debug-overlay';
const DEBUG_ENABLED_STORAGE_KEY = 'battlecontrol.debug-enabled';
const DEBUG_LOG_ENDPOINT = '/__debug-log';

type DebugLogSnapshot = {
  logs: string[];
  trace: string[];
  status: string;
  updatedAt: string;
};

function getOverlayLogs(overlay: HTMLElement) {
  return overlay.dataset.logs ? JSON.parse(overlay.dataset.logs) as string[] : [];
}

function getOverlayStatus(overlay: HTMLElement) {
  return overlay.dataset.status ?? '';
}

function getOverlayTrace(overlay: HTMLElement) {
  return overlay.dataset.trace ? JSON.parse(overlay.dataset.trace) as string[] : [];
}

export function buildDebugLogSnapshot(status: string, logs: string[], trace: string[]): DebugLogSnapshot {
  return {
    status,
    logs,
    trace,
    updatedAt: new Date().toISOString(),
  };
}

function syncDebugLogFile(overlay: HTMLElement) {
  if (!import.meta.env.DEV || !isDebugUiEnabled()) {
    return;
  }

  const snapshot = buildDebugLogSnapshot(
    getOverlayStatus(overlay),
    getOverlayLogs(overlay),
    getOverlayTrace(overlay),
  );
  void fetch(DEBUG_LOG_ENDPOINT, {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
    },
    body: JSON.stringify(snapshot),
  }).catch(() => {});
}

function renderOverlay(overlay: HTMLElement) {
  const status = getOverlayStatus(overlay);
  const logs = getOverlayLogs(overlay);
  overlay.textContent = [status, ...logs].filter(Boolean).join('\n');
}

export function mountDebugOverlay() {
  const overlay = document.createElement('div');
  overlay.id = DEBUG_OVERLAY_ID;
  overlay.dataset.logs = '[]';
  overlay.dataset.trace = '[]';
  overlay.hidden = !isDebugUiEnabled();
  document.body.appendChild(overlay);
  syncDebugLogFile(overlay);
}

export function isDebugUiEnabled() {
  return window.localStorage.getItem(DEBUG_ENABLED_STORAGE_KEY) === 'true';
}

export function toggleDebugUi() {
  const next = !isDebugUiEnabled();
  window.localStorage.setItem(DEBUG_ENABLED_STORAGE_KEY, String(next));

  const overlay = document.getElementById(DEBUG_OVERLAY_ID);
  if (overlay) {
    overlay.hidden = !next;
    if (next) {
      const logs = getOverlayLogs(overlay);
      overlay.dataset.logs = JSON.stringify(['debug enabled', ...logs].slice(0, 8));
      renderOverlay(overlay);
    }
    syncDebugLogFile(overlay);
  }
}

export function appendDebugLine(line: string) {
  const overlay = document.getElementById(DEBUG_OVERLAY_ID);
  if (!overlay) {
    return;
  }

  const logs = getOverlayLogs(overlay);
  overlay.dataset.logs = JSON.stringify([line, ...logs].slice(0, 8));
  renderOverlay(overlay);
  syncDebugLogFile(overlay);
}

export function appendDebugTrace(line: string) {
  const overlay = document.getElementById(DEBUG_OVERLAY_ID);
  if (!overlay) {
    return;
  }

  const trace = getOverlayTrace(overlay);
  overlay.dataset.trace = JSON.stringify([...trace, line].slice(-180));
  syncDebugLogFile(overlay);
}

export function clearDebugTrace() {
  const overlay = document.getElementById(DEBUG_OVERLAY_ID);
  if (!overlay) {
    return;
  }

  overlay.dataset.trace = '[]';
  syncDebugLogFile(overlay);
}

export function setDebugStatus(line: string) {
  const overlay = document.getElementById(DEBUG_OVERLAY_ID);
  if (!overlay) {
    return;
  }

  overlay.dataset.status = line;
  renderOverlay(overlay);
  syncDebugLogFile(overlay);
}
