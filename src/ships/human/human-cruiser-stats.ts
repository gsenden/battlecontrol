import type { ShipStats } from '../ship-stats.js';

export const HUMAN_CRUISER: ShipStats = {
  raceName: 'Earthling',
  shipClass: 'Cruiser',
  captainNames: ['Decker', 'Trent', 'Adama', 'Spiff', 'Graeme', 'Kirk', 'Pike', 'Halleck', 'Tuf', 'Pirx', 'Wu', 'VanRijn', 'Ender', 'Buck', 'Solo', 'Belt'],
  cost: 11,

  mass: 6,
  thrustIncrement: 0.6,
  maxSpeed: 4,
  turnRate: Math.PI / 8,

  turnWait: 1,
  thrustWait: 4,
  weaponWait: 10,
  specialWait: 9,

  maxEnergy: 18,
  energyRegeneration: 1,
  energyWait: 8,
  weaponEnergyCost: 9,
  specialEnergyCost: 4,

  maxCrew: 18,

  spritePrefix: 'human-cruiser',
  color: 0x4488ff,
  size: 16,
};
