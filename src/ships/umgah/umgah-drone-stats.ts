import type { ShipStats } from '../ship-stats.js';

export const UMGAH_DRONE: ShipStats = {
  raceName: 'Umgah',
  shipClass: 'Drone',
  captainNames: ["Julg'ka", "Gibj'o", "Baguk'i", "O'guk'e", "Gwap'he", "Chez'ef", "Znork'i", 'Bob', "Kwik'ow", "Ei'Ei'o", "Brewz'k", "Pruk'u", "O'bargy", "Kterbi'a", "Chup'he", "I'buba"],
  cost: 7,

  mass: 1,
  thrustIncrement: 1.2,
  maxSpeed: 3,
  turnRate: Math.PI / 8,

  turnWait: 4,
  thrustWait: 3,
  weaponWait: 0,
  specialWait: 2,

  maxEnergy: 30,
  energyRegeneration: 30,
  energyWait: 150,
  weaponEnergyCost: 0,
  specialEnergyCost: 1,

  maxCrew: 10,

  spritePrefix: 'umgah-drone',
  color: 0xffffff,
  size: 12,
};
