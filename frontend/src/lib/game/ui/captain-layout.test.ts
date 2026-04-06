import { describe, expect, it } from 'vitest';
import { parseCaptainFrameFiles, parseCaptainFrameStyles, parseCaptainLayout } from './captain-layout.js';

describe('parseCaptainLayout', () => {
  it('preserves repeated frame entries from the ani file', () => {
    const frames = parseCaptainFrameFiles(`
avenger-cap-000.png 7 1 0 0
avenger-cap-001.png -1 1 0 0
avenger-cap-002.png -1 1 0 0
avenger-cap-002.png -1 1 0 0
avenger-cap-002.png -1 1 0 0
avenger-cap-003.png -1 1 -128 0
`);

    expect(frames).toEqual([
      'avenger-cap-000.png',
      'avenger-cap-001.png',
      'avenger-cap-002.png',
      'avenger-cap-002.png',
      'avenger-cap-002.png',
      'avenger-cap-003.png',
    ]);
  });

  it('preserves per-frame styles from the ani file', () => {
    const styles = parseCaptainFrameStyles(`
avenger-cap-000.png 7 1 0 0
avenger-cap-001.png -1 1 -3 0
avenger-cap-002.png -1 1 4 -2
`);

    expect(styles).toEqual([
      'left: 0px; top: 0px;',
      'left: 3px; top: 0px;',
      'left: -4px; top: 2px;',
    ]);
  });

  it('derives the earthling layout from ani offsets', () => {
    const layout = parseCaptainLayout(`
# Captain overlay ANI
# Columns: <png-file> <x-scale> <y-scale> <x-offset> <y-offset>
# Background
cruiser-cap-000.png -2 1 0 0

# Turn left/right
cruiser-cap-001.png -2 1 -122 0
cruiser-cap-002.png -2 1 -122 0
cruiser-cap-002.png -2 1 -122 0
cruiser-cap-002.png -2 1 -122 0
cruiser-cap-003.png -2 1 -122 0

# Thrust
cruiser-cap-004.png -2 1 -151 0
cruiser-cap-004.png -2 1 -151 0
cruiser-cap-005.png -2 1 -151 0

# Primary weapon
cruiser-cap-006.png -2 1 0 0
cruiser-cap-006.png -2 1 0 0
cruiser-cap-007.png -2 1 0 0

# Special
cruiser-cap-008.png -2 1 -85 -4
cruiser-cap-008.png -2 1 -85 -4
cruiser-cap-009.png -2 1 -85 -4
`);

    expect(layout).toEqual({
      turnLeft: { start: 1, count: 5, style: '' },
      thrust: { start: 6, count: 3, style: '' },
      weapon: { start: 9, count: 3, style: '' },
      special: { start: 12, count: 3, style: '' },
    });
  });
});
