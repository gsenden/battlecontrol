import { describe, expect, it } from 'vitest';
import { projectileFrameFor, velocityToSaturnFrame } from './projectile-frame.js';

describe('velocityToSaturnFrame', () => {
  it('uses the same 16-facing frame mapping as ships', () => {
    expect(velocityToSaturnFrame(1, 0)).toBe(4);
  });
});

describe('projectileFrameFor', () => {
  it('uses animation frames for non-directional projectiles', () => {
    expect(projectileFrameFor('chenjesu-doggy', 0, 0, 9)).toBe(2);
  });
});
