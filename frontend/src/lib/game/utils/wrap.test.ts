import { describe, expect, it } from 'vitest';
import { shortestWrappedDelta, wrapPoint } from './wrap.js';

describe('wrap helpers', () => {
  describe('wrapPoint', () => {
    it('keeps points inside bounds unchanged', () => {
      expect(wrapPoint(120, 340, 1000, 800)).toEqual({ x: 120, y: 340 });
    });

    it('wraps points that pass the positive edge', () => {
      expect(wrapPoint(1005, 805, 1000, 800)).toEqual({ x: 5, y: 5 });
    });

    it('wraps points that pass the negative edge', () => {
      expect(wrapPoint(-5, -10, 1000, 800)).toEqual({ x: 995, y: 790 });
    });
  });

  describe('shortestWrappedDelta', () => {
    it('returns the direct delta when no wrap is shorter', () => {
      expect(shortestWrappedDelta(100, 180, 1000)).toBe(80);
      expect(shortestWrappedDelta(180, 100, 1000)).toBe(-80);
    });

    it('chooses the wrapped path across the positive edge', () => {
      expect(shortestWrappedDelta(980, 20, 1000)).toBe(40);
    });

    it('chooses the wrapped path across the negative edge', () => {
      expect(shortestWrappedDelta(20, 980, 1000)).toBe(-40);
    });
  });
});
