import type { ShipStats } from '../ship-stats.js';

export const MELNORME_TRADER: ShipStats = {
  raceName: 'Melnorme',
  shipClass: 'Trader',
  captainNames: ['Reddish', 'Orangy', 'Aqua', 'Crimson', 'Magenta', 'Cheruse', 'Beige', 'Fuchsia', 'Umber', 'Cerise', 'Mauve', 'Grayish', 'Yellow', 'Black', 'Bluish', 'Purple'],
  cost: 18,

  mass: 7,
  thrustIncrement: 1.2,
  maxSpeed: 6,
  turnRate: Math.PI / 8,

  turnWait: 4,
  thrustWait: 4,
  weaponWait: 1,
  specialWait: 20,

  maxEnergy: 42,
  energyRegeneration: 1,
  energyWait: 4,
  weaponEnergyCost: 5,
  specialEnergyCost: 20,

  maxCrew: 20,

  spritePrefix: 'melnorme-trader',
  color: 0xffffff,
  size: 18,
};
