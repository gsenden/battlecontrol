import type { ShipStats } from '../ship-stats.js';

export const CHENJESU_BROODHOME: ShipStats = {
  raceName: 'Chenjesu',
  shipClass: 'Broodhome',
  captainNames: ['Kzzakk', 'Tzrrow', 'Zzmzmm', 'Vziziz', 'Hmmhmm', 'Bzrak', 'Krrtzz', 'Zzzzz', 'Zxzakz', 'Brrzap', 'Tzaprak', 'Pzkrakz', 'Fzzzz', 'Vrroww', 'Zznaz', 'Zzzhmm'],
  cost: 28,

  mass: 10,
  thrustIncrement: 0.6,
  maxSpeed: 4.5,
  turnRate: Math.PI / 8,

  turnWait: 6,
  thrustWait: 4,
  weaponWait: 0,
  specialWait: 0,

  maxEnergy: 30,
  energyRegeneration: 1,
  energyWait: 4,
  weaponEnergyCost: 5,
  specialEnergyCost: 30,

  maxCrew: 36,

  spritePrefix: 'chenjesu-broodhome',
  color: 0xffffff,
  size: 22,
};
