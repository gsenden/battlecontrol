import type { ShipStats } from '../ship-stats.js';

export const KOHRAH_MARAUDER: ShipStats = {
  raceName: 'Kohr-Ah',
  shipClass: 'Marauder',
  captainNames: ['Death 11', 'Death 17', 'Death 37', 'Death 23', 'Death 7', 'Death 13', 'Death 19', 'Death 29', 'Death 31', 'Death 41', 'Death 43', 'Death 3', 'Death 5', 'Death 47', 'Death 53', 'Death 83'],
  cost: 30,

  mass: 10,
  thrustIncrement: 1.2,
  maxSpeed: 5,
  turnRate: Math.PI / 8,

  turnWait: 4,
  thrustWait: 6,
  weaponWait: 6,
  specialWait: 9,

  maxEnergy: 42,
  energyRegeneration: 1,
  energyWait: 4,
  weaponEnergyCost: 6,
  specialEnergyCost: 21,

  maxCrew: 42,

  spritePrefix: 'kohrah-marauder',
  color: 0xffffff,
  size: 22,
};
