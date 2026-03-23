#!/bin/bash
# Generate SC2 reference data for BattleControl tests
#
# Usage:
#   ./generate.sh                          # uses default SC2 path
#   ./generate.sh /path/to/sc2             # custom SC2 source path
#   SC2_SRC=/path/to/sc2 ./generate.sh     # via environment variable

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SC2_SRC="${1:-${SC2_SRC:-$(dirname "$SCRIPT_DIR")/../sc2}}"

# Resolve to absolute path
SC2_SRC="$(cd "$SC2_SRC" 2>/dev/null && pwd)" || {
    echo "Error: SC2 source directory not found: $SC2_SRC"
    echo "Usage: $0 /path/to/sc2"
    exit 1
}

echo "SC2 source: $SC2_SRC"

# Verify SC2 source exists
if [ ! -f "$SC2_SRC/src/uqm/velocity.c" ]; then
    echo "Error: $SC2_SRC does not look like the SC2/UQM source directory"
    echo "Expected to find src/uqm/velocity.c"
    exit 1
fi

cd "$SCRIPT_DIR"

echo "Compiling reference data generator..."
gcc -std=c11 -Wall -Wextra -O2 \
    -o generate_bin \
    generate.c \
    sc2_velocity.c \
    -lm

echo "Generating reference data..."
./generate_bin reference.json

echo "Cleaning up..."
rm -f generate_bin

echo "Done! Reference data written to $SCRIPT_DIR/reference.json"
