import { describe, expect, it } from 'vitest';
import { parseCaptainLayout } from './captain-layout.js';

describe('parseCaptainLayout', () => {
  it('derives the earthling layout from ani offsets', () => {
    const layout = parseCaptainLayout(`
cruiser-cap-000.png -2 1 0 0
cruiser-cap-001.png -2 1 -122 0
cruiser-cap-002.png -2 1 -122 0
cruiser-cap-002.png -2 1 -122 0
cruiser-cap-002.png -2 1 -122 0
cruiser-cap-003.png -2 1 -122 0
cruiser-cap-004.png -2 1 -151 0
cruiser-cap-004.png -2 1 -151 0
cruiser-cap-005.png -2 1 -151 0
cruiser-cap-006.png -2 1 0 0
cruiser-cap-006.png -2 1 0 0
cruiser-cap-007.png -2 1 0 0
cruiser-cap-008.png -2 1 -85 -4
cruiser-cap-008.png -2 1 -85 -4
cruiser-cap-009.png -2 1 -85 -4
`);

    expect(layout).toEqual({
      turnLeft: { start: 1, count: 3, style: 'left: 122px; top: 0px;' },
      thrust: { start: 4, count: 2, style: 'left: 151px; top: 0px;' },
      weapon: { start: 6, count: 2, style: 'left: 0px; top: 0px;' },
      special: { start: 8, count: 2, style: 'left: 85px; top: 4px;' },
    });
  });
});
