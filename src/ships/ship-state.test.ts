import { describe, it, expect } from 'vitest';
import { ShipState } from './ship-state.js';
import { HUMAN_CRUISER } from './ship-stats.js';

const NO_INPUT = { left: false, right: false, thrust: false, weapon: false, special: false };

function createShip() {
  return new ShipState(HUMAN_CRUISER, 0);
}

describe('ShipState', () => {
  describe('initialization', () => {
    it('starts with full crew and energy', () => {
      const ship = createShip();
      expect(ship.crew).toBe(HUMAN_CRUISER.maxCrew);
      expect(ship.energy).toBe(HUMAN_CRUISER.maxEnergy);
    });

    it('starts with all counters at 0', () => {
      const ship = createShip();
      expect(ship.turnCounter).toBe(0);
      expect(ship.thrustCounter).toBe(0);
      expect(ship.weaponCounter).toBe(0);
      expect(ship.specialCounter).toBe(0);
    });
  });

  describe('turning', () => {
    it('turns left by turnRate', () => {
      const ship = createShip();
      const startAngle = ship.facing;
      ship.update({ ...NO_INPUT, left: true }, 0);
      expect(ship.facing).toBeCloseTo(startAngle - HUMAN_CRUISER.turnRate);
    });

    it('turns right by turnRate', () => {
      const ship = createShip();
      const startAngle = ship.facing;
      ship.update({ ...NO_INPUT, right: true }, 0);
      expect(ship.facing).toBeCloseTo(startAngle + HUMAN_CRUISER.turnRate);
    });

    it('respects turnWait cooldown', () => {
      const ship = createShip();
      const startAngle = ship.facing;

      // First turn happens immediately
      ship.update({ ...NO_INPUT, right: true }, 0);
      const afterFirstTurn = ship.facing;
      expect(afterFirstTurn).not.toBeCloseTo(startAngle);

      // Next frame: still in cooldown (turnWait=1), no turn
      ship.update({ ...NO_INPUT, right: true }, 0);
      expect(ship.facing).toBeCloseTo(afterFirstTurn);

      // After cooldown expires, turn again
      ship.update({ ...NO_INPUT, right: true }, 0);
      expect(ship.facing).toBeCloseTo(afterFirstTurn + HUMAN_CRUISER.turnRate);
    });

    it('does not turn without input', () => {
      const ship = createShip();
      const startAngle = ship.facing;
      ship.update(NO_INPUT, 0);
      expect(ship.facing).toBe(startAngle);
    });
  });

  describe('thrust', () => {
    it('generates applyForce command when thrusting', () => {
      const ship = createShip();
      const commands = ship.update({ ...NO_INPUT, thrust: true }, 0);
      const forceCmd = commands.find(c => c.type === 'applyForce');
      expect(forceCmd).toBeDefined();
      expect(forceCmd!.fx).toBeDefined();
      expect(forceCmd!.fy).toBeDefined();
    });

    it('applies force in facing direction', () => {
      const ship = createShip();
      // Default facing is -PI/2 (up), so force should be mostly in -y
      const commands = ship.update({ ...NO_INPUT, thrust: true }, 0);
      const forceCmd = commands.find(c => c.type === 'applyForce')!;
      expect(forceCmd.fy!).toBeLessThan(0); // upward
      expect(Math.abs(forceCmd.fx!)).toBeLessThan(0.0001); // negligible x
    });

    it('respects thrustWait cooldown', () => {
      const ship = createShip();

      // First thrust (frame 0): fires, counter set to 4
      let commands = ship.update({ ...NO_INPUT, thrust: true }, 0);
      expect(commands.find(c => c.type === 'applyForce')).toBeDefined();

      // Frames 1-4: in cooldown (counter 4->3->2->1->0)
      for (let i = 0; i < HUMAN_CRUISER.thrustWait; i++) {
        commands = ship.update({ ...NO_INPUT, thrust: true }, 0);
        expect(commands.find(c => c.type === 'applyForce')).toBeUndefined();
      }

      // Frame 5: counter reached 0 last frame, now fires again
      commands = ship.update({ ...NO_INPUT, thrust: true }, 0);
      expect(commands.find(c => c.type === 'applyForce')).toBeDefined();
    });

    it('thrust direction changes after turning', () => {
      const ship = createShip();

      // Turn right
      ship.update({ ...NO_INPUT, right: true }, 0);
      // Wait for turn cooldown
      ship.update(NO_INPUT, 0);

      // Thrust in new direction
      const commands = ship.update({ ...NO_INPUT, thrust: true }, 0);
      const forceCmd = commands.find(c => c.type === 'applyForce')!;
      // After turning right from -PI/2, force should have positive x component
      expect(forceCmd.fx!).toBeGreaterThan(0);
    });
  });

  describe('speed cap', () => {
    it('generates capSpeed when over maxSpeed', () => {
      const ship = createShip();
      const commands = ship.update(NO_INPUT, HUMAN_CRUISER.maxSpeed + 1);
      const capCmd = commands.find(c => c.type === 'capSpeed');
      expect(capCmd).toBeDefined();
      expect(capCmd!.maxSpeed).toBe(HUMAN_CRUISER.maxSpeed);
    });

    it('does not cap speed when under maxSpeed', () => {
      const ship = createShip();
      const commands = ship.update(NO_INPUT, HUMAN_CRUISER.maxSpeed - 1);
      expect(commands.find(c => c.type === 'capSpeed')).toBeUndefined();
    });
  });

  describe('energy system', () => {
    it('regenerates energy after energyWait frames', () => {
      const ship = createShip();
      // Drain some energy first
      ship.energy = 10;

      // Counter starts at 0, so first update regens immediately
      ship.update(NO_INPUT, 0);
      expect(ship.energy).toBe(11);

      // Now counter is set to energyWait (8). Must wait 8 frames.
      for (let i = 0; i < HUMAN_CRUISER.energyWait; i++) {
        ship.update(NO_INPUT, 0);
      }
      expect(ship.energy).toBe(11); // still waiting (counter just hit 0)

      // Next frame: counter is 0, regens again
      ship.update(NO_INPUT, 0);
      expect(ship.energy).toBe(12);
    });

    it('does not regenerate above maxEnergy', () => {
      const ship = createShip();
      ship.update(NO_INPUT, 0);
      expect(ship.energy).toBe(HUMAN_CRUISER.maxEnergy);
    });

    it('weapon costs energy', () => {
      const ship = createShip();
      ship.update({ ...NO_INPUT, weapon: true }, 0);
      expect(ship.energy).toBe(HUMAN_CRUISER.maxEnergy - HUMAN_CRUISER.weaponEnergyCost);
    });

    it('cannot fire weapon without enough energy', () => {
      const ship = createShip();
      ship.energy = HUMAN_CRUISER.weaponEnergyCost - 1;
      // Energy regen happens before weapon check. Counter=0 and energy<max,
      // so energy regens to weaponEnergyCost, then weapon fires!
      // To truly test "not enough energy", set energyCounter so regen doesn't fire.
      ship.energyCounter = 5;
      const startEnergy = ship.energy;
      ship.update({ ...NO_INPUT, weapon: true }, 0);
      expect(ship.energy).toBe(startEnergy); // unchanged, weapon didn't fire
    });

    it('special costs energy', () => {
      const ship = createShip();
      ship.update({ ...NO_INPUT, special: true }, 0);
      expect(ship.energy).toBe(HUMAN_CRUISER.maxEnergy - HUMAN_CRUISER.specialEnergyCost);
    });

    it('weapon respects weaponWait cooldown', () => {
      const ship = createShip();
      ship.update({ ...NO_INPUT, weapon: true }, 0);
      // energy: 18 - 9 = 9. energyCounter=0 since it was max before.
      // But regen ran first this frame (energy was max, so no regen).
      // After weapon fire: energy=9, weaponCounter=10

      // Next frame: energy regen fires (counter=0, energy<max) -> energy=10
      // Weapon still in cooldown (counter=10->9), doesn't fire
      ship.update({ ...NO_INPUT, weapon: true }, 0);
      expect(ship.energy).toBe(10); // 9 + 1 regen, no weapon fire
    });
  });

  describe('damage', () => {
    it('reduces crew', () => {
      const ship = createShip();
      ship.takeDamage(5);
      expect(ship.crew).toBe(HUMAN_CRUISER.maxCrew - 5);
    });

    it('returns true when crew reaches 0', () => {
      const ship = createShip();
      expect(ship.takeDamage(HUMAN_CRUISER.maxCrew)).toBe(true);
      expect(ship.crew).toBe(0);
    });

    it('does not go below 0 crew', () => {
      const ship = createShip();
      ship.takeDamage(999);
      expect(ship.crew).toBe(0);
    });
  });
});
