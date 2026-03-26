import { describe, expect, it } from 'vitest';
import {
  getCaptainAnimationFrameIndex,
  getCaptainTurnFrameIndexes,
  stepCaptainOffset,
} from './hud-state.svelte.js';

describe('hud-state captain helpers', () => {
  it('uses the original captain turn frame stack for left turns', () => {
    expect(getCaptainTurnFrameIndexes(1, 1, 0)).toEqual([4, 3, 2]);
    expect(getCaptainTurnFrameIndexes(1, 2, 0)).toEqual([4, 3, 2, 1]);
  });

  it('uses the original captain turn frame stack for right turns', () => {
    expect(getCaptainTurnFrameIndexes(1, 0, 1)).toEqual([2, 3, 4]);
    expect(getCaptainTurnFrameIndexes(1, 0, 2)).toEqual([2, 3, 4, 5]);
  });

  it('uses the two-step captain overlay progression from the original game', () => {
    expect(stepCaptainOffset(0, true)).toBe(1);
    expect(stepCaptainOffset(1, true)).toBe(2);
    expect(getCaptainAnimationFrameIndex(6, 1)).toBe(7);
    expect(getCaptainAnimationFrameIndex(6, 2)).toBe(8);
  });
});
