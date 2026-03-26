import type { ShipStats } from '../ship-stats.js';

export const SUPOX_BLADE: ShipStats = {
  raceName: 'Supox',
  shipClass: 'Blade',
  captainNames: ['Trifid', 'Crinoid', 'FlyTrap', 'Thistle', 'Ivy', 'Sprout', 'Twig', 'Root', 'Branch', 'Thorn', 'Bark', 'Bud', 'Nut', 'Stem', 'Bramble', 'Thicket'],
  cost: 16,

  mass: 4,
  thrustIncrement: 1.6,
  maxSpeed: 6.7,
  turnRate: Math.PI / 8,

  turnWait: 1,
  thrustWait: 0,
  weaponWait: 2,
  specialWait: 0,

  maxEnergy: 16,
  energyRegeneration: 1,
  energyWait: 4,
  weaponEnergyCost: 1,
  specialEnergyCost: 1,

  maxCrew: 12,

  spritePrefix: 'supox-blade',
  color: 0xffffff,
  size: 13,
};
