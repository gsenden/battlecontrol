import type { ShipStats } from '../ship-stats.js';

export const SYREEN_PENETRATOR: ShipStats = {
  raceName: 'Syreen',
  shipClass: 'Penetrator',
  captainNames: ['Teela', 'Dejah', 'Penny', 'Alia', "Be'lit", 'Ripley', 'Yarr', 'Ardala', 'Sparta', 'Munro', 'Danning', 'Brawne', 'Maya', 'Aelita', 'Alura', 'Dale'],
  cost: 13,

  mass: 2,
  thrustIncrement: 1.8,
  maxSpeed: 6,
  turnRate: Math.PI / 8,

  turnWait: 1,
  thrustWait: 1,
  weaponWait: 8,
  specialWait: 20,

  maxEnergy: 16,
  energyRegeneration: 1,
  energyWait: 6,
  weaponEnergyCost: 1,
  specialEnergyCost: 5,

  maxCrew: 12,

  spritePrefix: 'syreen-penetrator',
  color: 0xffffff,
  size: 12,
};
