import type { ShipStats } from '../ship-stats.js';

export const ILWRATH_AVENGER: ShipStats = {
  raceName: 'Ilwrath',
  shipClass: 'Avenger',
  captainNames: ['Gorgon', 'Taragon', 'Kalgon', 'Borgo', 'Dirga', 'Slygor', 'Rogash', 'Argarak', 'Kayzar', 'Baylor', 'Zoggak', 'Targa', 'Vogar', 'Lurgo', 'Regorjo', 'Manglor'],
  cost: 10,

  mass: 7,
  thrustIncrement: 1,
  maxSpeed: 4.2,
  turnRate: Math.PI / 8,

  turnWait: 2,
  thrustWait: 0,
  weaponWait: 0,
  specialWait: 13,

  maxEnergy: 16,
  energyRegeneration: 4,
  energyWait: 4,
  weaponEnergyCost: 1,
  specialEnergyCost: 3,

  maxCrew: 22,

  spritePrefix: 'ilwrath-avenger',
  color: 0xffffff,
  size: 18,
};
