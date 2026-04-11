#!/usr/bin/env bash
set -euo pipefail

SESSION_NAME="${1:-battlecontrol}"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if ! command -v tmux >/dev/null 2>&1; then
	echo "tmux is niet geinstalleerd"
	exit 1
fi

if tmux has-session -t "${SESSION_NAME}" 2>/dev/null; then
	exec tmux attach -t "${SESSION_NAME}"
fi

tmux new-session -d -s "${SESSION_NAME}" -c "${ROOT_DIR}" "cargo watch -x 'run -p server'"
tmux split-window -h -t "${SESSION_NAME}" -c "${ROOT_DIR}/frontend" "npm run dev"
tmux select-layout -t "${SESSION_NAME}" even-horizontal
exec tmux attach -t "${SESSION_NAME}"
