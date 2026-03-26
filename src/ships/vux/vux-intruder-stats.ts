import type { ShipStats } from '../ship-stats.js';

export const VUX_INTRUDER: ShipStats = {
  raceName: 'Vux',
  shipClass: 'Intruder',
  captainNames: ['ZIK', 'PUZ', 'ZUK', 'VIP', 'ZIT', 'YUK', 'DAK', 'ZRN', 'PIF', 'FIZ', 'FUP', 'ZUP', 'NRF', 'ZOG', 'ORZ', 'ZEK'],
  cost: 12,

  mass: 6,
  thrustIncrement: 1.4,
  maxSpeed: 3.5,
  turnRate: Math.PI / 8,

  turnWait: 6,
  thrustWait: 4,
  weaponWait: 0,
  specialWait: 7,

  maxEnergy: 40,
  energyRegeneration: 1,
  energyWait: 8,
  weaponEnergyCost: 1,
  specialEnergyCost: 2,

  maxCrew: 20,

  spritePrefix: 'vux-intruder',
  color: 0xffffff,
  size: 16,
};
