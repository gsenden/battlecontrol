import type { ShipStats } from '../ship-stats.js';

export const UTWIG_JUGGER: ShipStats = {
  raceName: 'Utwig',
  shipClass: 'Jugger',
  captainNames: ['Endo', 'Vermi', 'Manny', 'Uuter', 'Nergo', 'Sami', 'Duna', 'Frann', 'Krisk', 'Lololo', 'Snoon', 'Nestor', 'Lurg', 'Thory', 'Jujuby', 'Erog'],
  cost: 22,

  mass: 8,
  thrustIncrement: 1.2,
  maxSpeed: 6,
  turnRate: Math.PI / 8,

  turnWait: 1,
  thrustWait: 6,
  weaponWait: 7,
  specialWait: 12,

  maxEnergy: 20,
  energyRegeneration: 0,
  energyWait: 255,
  weaponEnergyCost: 0,
  specialEnergyCost: 1,

  maxCrew: 20,

  spritePrefix: 'utwig-jugger',
  color: 0xffffff,
  size: 19,
};
