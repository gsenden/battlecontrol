import type { ShipStats } from '../ship-stats.js';

export const ARILOU_SKIFF: ShipStats = {
  raceName: 'Arilou',
  shipClass: 'Skiff',
  captainNames: ['Fefaloo', 'Bezabu', 'Tiptushi', 'Marypup', 'Tinkafo', 'Patooti', 'Tifiwilo', 'Loleelu', 'Louifoui', 'Pinywiny', 'Oowbabe', 'Dingdup', 'Wewalia', 'Yipyapi', 'Ropilup', 'Wolwali'],
  cost: 16,

  mass: 1,
  thrustIncrement: 8,
  maxSpeed: 6.7,
  turnRate: Math.PI / 8,

  turnWait: 0,
  thrustWait: 0,
  weaponWait: 1,
  specialWait: 2,

  maxEnergy: 20,
  energyRegeneration: 1,
  energyWait: 6,
  weaponEnergyCost: 2,
  specialEnergyCost: 3,

  maxCrew: 6,

  spritePrefix: 'arilou-skiff',
  color: 0xffffff,
  size: 12,
};
