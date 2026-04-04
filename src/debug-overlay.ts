const DEBUG_OVERLAY_ID = 'debug-overlay';

function renderOverlay(overlay: HTMLElement) {
  const status = overlay.dataset.status;
  const logs = overlay.dataset.logs ? JSON.parse(overlay.dataset.logs) as string[] : [];
  overlay.textContent = [status, ...logs].filter(Boolean).join('\n');
}

export function mountDebugOverlay() {
  const overlay = document.createElement('div');
  overlay.id = DEBUG_OVERLAY_ID;
  overlay.dataset.logs = '[]';
  document.body.appendChild(overlay);
}

export function appendDebugLine(line: string) {
  const overlay = document.getElementById(DEBUG_OVERLAY_ID);
  if (!overlay) {
    return;
  }

  const logs = overlay.dataset.logs ? JSON.parse(overlay.dataset.logs) as string[] : [];
  overlay.dataset.logs = JSON.stringify([line, ...logs].slice(0, 8));
  renderOverlay(overlay);
}

export function setDebugStatus(line: string) {
  const overlay = document.getElementById(DEBUG_OVERLAY_ID);
  if (!overlay) {
    return;
  }

  overlay.dataset.status = line;
  renderOverlay(overlay);
}
