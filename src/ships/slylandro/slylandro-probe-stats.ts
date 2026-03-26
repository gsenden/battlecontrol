import type { ShipStats } from '../ship-stats.js';

export const SLYLANDRO_PROBE: ShipStats = {
  raceName: 'Slylandro',
  shipClass: 'Probe',
  captainNames: ['2418-B', '2418-B', '2418-B', '2418-B', '2418-B', '2418-B', '2418-B', '2418-B', '2418-B', '2418-B', '2418-B', '2418-B', '2418-B', '2418-B', '2418-B', '2418-B'],
  cost: 17,

  mass: 1,
  thrustIncrement: 12,
  maxSpeed: 10,
  turnRate: Math.PI / 8,

  turnWait: 0,
  thrustWait: 0,
  weaponWait: 17,
  specialWait: 20,

  maxEnergy: 20,
  energyRegeneration: 0,
  energyWait: 10,
  weaponEnergyCost: 2,
  specialEnergyCost: 0,

  maxCrew: 12,

  spritePrefix: 'slylandro-probe',
  color: 0xffffff,
  size: 12,
};
