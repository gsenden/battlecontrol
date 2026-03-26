import type { ShipStats } from '../ship-stats.js';

export const CHMMR_AVATAR: ShipStats = {
  raceName: 'Chmmr',
  shipClass: 'Avatar',
  captainNames: ['Mnzgk', 'Chzrmn', 'Bzztrm', 'Zrnzrk', 'Tzzqrn', 'Kzzrn', 'Vzrzn', 'Qrntz', 'Rmnzk', 'Szmrnz', 'Zbzzn', 'Frnkzk', 'Prmtzz', 'Tzrtzn', 'Kztztz', 'Mrnkzt'],
  cost: 30,

  mass: 10,
  thrustIncrement: 1.4,
  maxSpeed: 5.8,
  turnRate: Math.PI / 8,

  turnWait: 3,
  thrustWait: 5,
  weaponWait: 0,
  specialWait: 0,

  maxEnergy: 42,
  energyRegeneration: 1,
  energyWait: 1,
  weaponEnergyCost: 2,
  specialEnergyCost: 1,

  maxCrew: 42,

  spritePrefix: 'chmmr-avatar',
  color: 0xa8fff5,
  size: 22,
};
