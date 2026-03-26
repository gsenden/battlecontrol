import type { ShipStats } from '../ship-stats.js';

export const SHOFIXTI_SCOUT: ShipStats = {
  raceName: 'Shofixti',
  shipClass: 'Scout',
  captainNames: ['Hiyata', 'Wasabe', 'Kudzu', 'Ichiban', 'Bonsai!', 'Genjiro', 'Ginzu', 'Busu', 'Gaijin', 'Daikon', 'Sushi', 'Naninani', 'Chimchim', 'Tora-3', 'Tofu', 'Kimba'],
  cost: 5,

  mass: 1,
  thrustIncrement: 1,
  maxSpeed: 5.8,
  turnRate: Math.PI / 8,

  turnWait: 1,
  thrustWait: 0,
  weaponWait: 3,
  specialWait: 0,

  maxEnergy: 4,
  energyRegeneration: 1,
  energyWait: 9,
  weaponEnergyCost: 1,
  specialEnergyCost: 0,

  maxCrew: 6,

  spritePrefix: 'shofixti-scout',
  color: 0xffffff,
  size: 12,
};
