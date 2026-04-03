import { describe, it, expect } from 'vitest';
import referenceData from './reference.json';

// These tests verify that we understand the SC2 reference data correctly
// and serve as documentation of the original game's behavior

describe('SC2 Reference Data', () => {
  describe('collision_head_on', () => {
    const scenario = referenceData.collision_head_on;

    it('reverses both ships in a head-on equal-mass collision', () => {
      expect([
        scenario.after.ship0.vx,
        scenario.after.ship0.vy,
        scenario.after.ship1.vx,
        scenario.after.ship1.vy,
      ]).toEqual([-127, 0, 127, 0]);
    });
  });

  describe('collision_asymmetric_head_on', () => {
    const scenario = referenceData.collision_asymmetric_head_on;

    it('preserves the faster outgoing ships when the post-collision speed stays above the minimum', () => {
      expect([
        scenario.after.ship0.vx,
        scenario.after.ship0.vy,
        scenario.after.ship1.vx,
        scenario.after.ship1.vy,
      ]).toEqual([-127, 0, 192, 0]);
    });
  });

  describe('collision_existing_cooldowns', () => {
    const scenario = referenceData.collision_existing_cooldowns;

    it('keeps higher existing collision cooldowns', () => {
      expect([scenario.turnWait, scenario.thrustWait]).toEqual([2, 4]);
    });
  });

  describe('collision_cooldowns', () => {
    const scenario = referenceData.collision_cooldowns;

    it('uses the original collision turn and thrust wait values', () => {
      expect([scenario.turnWait, scenario.thrustWait]).toEqual([1, 3]);
    });
  });

  describe('thrust_straight', () => {
    const scenario = referenceData.thrust_straight;

    it('has 60 frames of data', () => {
      expect(scenario.frames).toHaveLength(60);
    });

    it('starts at position (5000, 5000) area', () => {
      const f0 = scenario.frames[0];
      expect(f0.x).toBe(5000);
      // y changes on first frame due to thrust
    });

    it('accelerates in discrete steps due to thrustWait=4', () => {
      // thrust_increment=3 display units = 12 world = 384 velocity
      // But velocity is in internal units, every 5th frame (wait=4) gets a thrust
      const f0 = scenario.frames[0];
      const f5 = scenario.frames[5];
      // First thrust at frame 0: vel = -96 (facing 0 = up in SC2)
      expect(f0.vy).toBe(-96);
      // Second thrust at frame 5: vel should increase
      expect(f5.vy).toBe(-192);
    });

    it('reaches max speed around frame 35-40', () => {
      const f40 = scenario.frames[40];
      // At max speed, velocity should be -768 (max_thrust=24 display = 96 world = 768 vel... but capped)
      expect(f40.vy).toBe(-768);
      expect(f40.statusFlags & 128).toBe(128); // SHIP_AT_MAX_SPEED
    });

    it('velocity stays constant after max speed', () => {
      for (let i = 40; i < 60; i++) {
        expect(scenario.frames[i].vy).toBe(-768);
      }
    });
  });

  describe('turn_and_thrust', () => {
    const scenario = referenceData.turn_and_thrust;

    it('turns from facing 0 during first 10 frames', () => {
      const f0 = scenario.frames[0];
      const f9 = scenario.frames[9];
      // turnWait=1 means a turn every 2 frames
      // 10 frames of left turn = 5 actual turns
      expect(f0.facing).toBe(15); // turned left once on frame 0: 0-1 = 15
      expect(f9.facing).not.toBe(0);
    });

    it('thrusts in the turned direction after frame 10', () => {
      const f15 = scenario.frames[15];
      // Should have velocity components matching the turned facing
      expect(f15.vx !== 0 || f15.vy !== 0).toBe(true);
    });
  });

  describe('max_speed', () => {
    const scenario = referenceData.max_speed;

    it('has 120 frames', () => {
      expect(scenario.frames).toHaveLength(120);
    });

    it('velocity is capped after reaching max', () => {
      // Find the first frame with SHIP_AT_MAX_SPEED (flag 128)
      const maxFrame = scenario.frames.find(f => (f.statusFlags & 128) !== 0);
      expect(maxFrame).toBeDefined();

      // All subsequent frames should have same velocity magnitude
      const maxVy = maxFrame!.vy;
      for (let i = maxFrame!.frame; i < 120; i++) {
        expect(scenario.frames[i].vy).toBe(maxVy);
      }
    });
  });

  describe('energy', () => {
    const scenario = referenceData.energy;

    it('weapon drains 9 energy on first frame', () => {
      expect(scenario.frames[0].energy).toBe(18 - 9); // 9
    });

    it('energy regenerates over time', () => {
      const lastFrame = scenario.frames[scenario.frames.length - 1];
      // After 80 frames with 2 weapon fires, energy should have partially recovered
      expect(lastFrame.energy).toBeGreaterThan(0);
    });
  });

  describe('all_facings', () => {
    const facings = referenceData.all_facings.facings;

    it('has 16 facings', () => {
      expect(facings).toHaveLength(16);
    });

    it('facing 0 points up (negative Y)', () => {
      expect(facings[0].vx).toBe(0);
      expect(facings[0].vy).toBeLessThan(0);
    });

    it('facing 4 points right (positive X)', () => {
      expect(facings[4].vx).toBeGreaterThan(0);
      expect(facings[4].vy).toBe(0);
    });

    it('facing 8 points down (positive Y)', () => {
      expect(facings[8].vx).toBe(0);
      expect(facings[8].vy).toBeGreaterThan(0);
    });

    it('facing 12 points left (negative X)', () => {
      expect(facings[12].vx).toBeLessThan(0);
      expect(facings[12].vy).toBe(0);
    });

    it('all facings have roughly the same speed magnitude', () => {
      const speeds = facings.map((f: { vx: number; vy: number }) =>
        Math.sqrt(f.vx * f.vx + f.vy * f.vy)
      );
      const avgSpeed = speeds.reduce((a: number, b: number) => a + b) / speeds.length;
      for (const speed of speeds) {
        // Allow ~3% variance due to integer rounding
        expect(Math.abs(speed - avgSpeed) / avgSpeed).toBeLessThan(0.03);
      }
    });
  });

  describe('gravity_well', () => {
    const scenario = referenceData.gravity_well;

    it('ships is pulled toward planet', () => {
      // Ship starts at x=3400, planet at x=3000
      // Gravity should pull ships left (negative vx)
      expect(scenario.frames[0].vx).toBeLessThan(0);
    });

    it('ships accelerates under gravity', () => {
      const v5 = Math.abs(scenario.frames[5].vx);
      const v15 = Math.abs(scenario.frames[15].vx);
      expect(v15).toBeGreaterThan(v5);
    });

    it('ships passes planet and oscillates', () => {
      // Ship should pass x=3000 at some point
      const passedPlanet = scenario.frames.some(f => f.x < 3000);
      expect(passedPlanet).toBe(true);
    });
  });

  describe('gravity_whip', () => {
    const scenario = referenceData.gravity_whip;

    it('ships exceeds normal max speed in gravity well', () => {
      // SHIP_BEYOND_MAX_SPEED = 64, SHIP_IN_GRAVITY_WELL = 256
      const beyondMax = scenario.frames.find(f => (f.statusFlags & 64) !== 0);
      expect(beyondMax).toBeDefined();
    });

    it('velocity exceeds normal max after gravity assist', () => {
      // Normal max velocity for Human Cruiser: 768
      const maxVel = Math.max(...scenario.frames.map(f => Math.abs(f.vy)));
      expect(maxVel).toBeGreaterThan(768);
    });
  });
});
