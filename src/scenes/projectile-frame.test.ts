import { describe, expect, it } from 'vitest';
import { velocityToSaturnFrame } from './projectile-frame.js';

describe('velocityToSaturnFrame', () => {
  it('uses the same 16-facing frame mapping as ships', () => {
    expect(velocityToSaturnFrame(1, 0)).toBe(4);
  });
});
