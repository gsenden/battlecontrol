import { describe, expect, it } from 'vitest';
import { buildHitPolygonFromOpaqueRows, getShipHitPolygon } from './hit-polygon.js';

describe('getShipHitPolygon', () => {
  it('builds a polygon from opaque rows', () => {
    expect(buildHitPolygonFromOpaqueRows([
      { y: 0, left: 2, right: 3 },
      { y: 1, left: 1, right: 4 },
      { y: 2, left: 0, right: 5 },
    ], 6, 3)).toEqual([
      { x: 0, y: -1 },
      { x: -1, y: 0 },
      { x: -2, y: 1 },
      { x: 3, y: 1 },
      { x: 2, y: 0 },
      { x: 1, y: -1 },
    ]);
  });

  it('scales the human cruiser polygon', () => {
    expect(getShipHitPolygon('human-cruiser', -Math.PI / 2, 0.5)).toEqual([
      { x: 0, y: -34 },
      { x: 6, y: -32 },
      { x: 9, y: -28 },
      { x: 9, y: -21 },
      { x: 6, y: -17 },
      { x: 3, y: -13 },
      { x: 3, y: -7 },
      { x: 7, y: -5 },
      { x: 11, y: 3 },
      { x: 13, y: 20 },
      { x: 12, y: 34 },
      { x: 9, y: 34 },
      { x: 7, y: 21 },
      { x: 5, y: 4 },
      { x: 3, y: 0 },
      { x: 3, y: 33 },
      { x: -3, y: 33 },
      { x: -3, y: 0 },
      { x: -5, y: 4 },
      { x: -7, y: 21 },
      { x: -8, y: 34 },
      { x: -12, y: 34 },
      { x: -12, y: 20 },
      { x: -10, y: 3 },
      { x: -6, y: -5 },
      { x: -3, y: -7 },
      { x: -3, y: -13 },
      { x: -5, y: -17 },
      { x: -8, y: -21 },
      { x: -8, y: -28 },
      { x: -5, y: -32 },
    ]);
  });

  it('returns the human cruiser polygon for facing up', () => {
    expect(getShipHitPolygon('human-cruiser', -Math.PI / 2)).toEqual([
      { x: 0, y: -68 },
      { x: 11, y: -65 },
      { x: 17, y: -57 },
      { x: 17, y: -43 },
      { x: 11, y: -35 },
      { x: 6, y: -27 },
      { x: 6, y: -14 },
      { x: 13, y: -10 },
      { x: 21, y: 5 },
      { x: 25, y: 39 },
      { x: 24, y: 67 },
      { x: 17, y: 68 },
      { x: 14, y: 42 },
      { x: 10, y: 8 },
      { x: 6, y: 0 },
      { x: 6, y: 66 },
      { x: -6, y: 66 },
      { x: -6, y: 0 },
      { x: -10, y: 8 },
      { x: -14, y: 42 },
      { x: -17, y: 68 },
      { x: -24, y: 67 },
      { x: -25, y: 39 },
      { x: -21, y: 5 },
      { x: -13, y: -10 },
      { x: -6, y: -14 },
      { x: -6, y: -27 },
      { x: -11, y: -35 },
      { x: -17, y: -43 },
      { x: -17, y: -57 },
      { x: -11, y: -65 },
    ]);
  });

  it('returns the androsynth blazer polygon for facing up', () => {
    expect(getShipHitPolygon('androsynth-blazer', -Math.PI / 2)).toEqual([
      { x: 0, y: -60 },
      { x: 14, y: -46 },
      { x: 18, y: -22 },
      { x: 12, y: 6 },
      { x: 6, y: 36 },
      { x: 0, y: 58 },
      { x: -6, y: 36 },
      { x: -12, y: 6 },
      { x: -18, y: -22 },
      { x: -14, y: -46 },
      { x: -6, y: -58 },
      { x: 6, y: -58 },
    ]);
  });
});
