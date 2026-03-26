import type { ShipStats } from '../ship-stats.js';

export const SPATHI_ELUDER: ShipStats = {
  raceName: 'Spathi',
  shipClass: 'Eluder',
  captainNames: ['Thwil', 'Pwappy', 'Phwiff', 'Wiffy', 'Plibnik', 'Snurfel', 'Kwimp', 'Pkunky', 'Jinkeze', 'Thintho', 'Rupatup', 'Nargle', 'Phlendo', 'Snelopy', 'Bwinkin', 'Whuff'],
  cost: 18,

  mass: 5,
  thrustIncrement: 2.4,
  maxSpeed: 8,
  turnRate: Math.PI / 8,

  turnWait: 1,
  thrustWait: 1,
  weaponWait: 0,
  specialWait: 7,

  maxEnergy: 10,
  energyRegeneration: 1,
  energyWait: 10,
  weaponEnergyCost: 2,
  specialEnergyCost: 3,

  maxCrew: 30,

  spritePrefix: 'spathi-eluder',
  color: 0xffffff,
  size: 15,
};
