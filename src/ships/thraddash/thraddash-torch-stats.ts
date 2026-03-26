import type { ShipStats } from '../ship-stats.js';

export const THRADDASH_TORCH: ShipStats = {
  raceName: 'Thraddash',
  shipClass: 'Torch',
  captainNames: ['Dthunk', 'Bardat', 'Znonk', 'Mnump', 'Bronk', 'Smup', 'Grulk', 'Hornk', 'Knarg', 'Drulg', 'Dgako', 'Znork', 'Kwamp', 'Fkank', 'Pdump', 'Whumps'],
  cost: 10,

  mass: 7,
  thrustIncrement: 1.4,
  maxSpeed: 4.7,
  turnRate: Math.PI / 8,

  turnWait: 1,
  thrustWait: 0,
  weaponWait: 12,
  specialWait: 0,

  maxEnergy: 24,
  energyRegeneration: 1,
  energyWait: 6,
  weaponEnergyCost: 2,
  specialEnergyCost: 1,

  maxCrew: 8,

  spritePrefix: 'thraddash-torch',
  color: 0xffffff,
  size: 18,
};
