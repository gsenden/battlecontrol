import type { ShipStats } from '../ship-stats.js';

export const ZOQFOTPIK_STINGER: ShipStats = {
  raceName: 'Zoq-Fot-Pik',
  shipClass: 'Stinger',
  captainNames: ['NikNak', 'FipPat', 'DipPak', 'FatPot', 'ZikFat', 'PukYor', 'TopNik', 'PorKoo', 'TikTak', 'RinTin', 'FitFap', 'TotToe', 'ZipZak', 'TikTok', 'MikMok', 'SikSok'],
  cost: 6,

  mass: 5,
  thrustIncrement: 2,
  maxSpeed: 6.7,
  turnRate: Math.PI / 8,

  turnWait: 1,
  thrustWait: 0,
  weaponWait: 0,
  specialWait: 6,

  maxEnergy: 10,
  energyRegeneration: 1,
  energyWait: 4,
  weaponEnergyCost: 1,
  specialEnergyCost: 7,

  maxCrew: 10,

  spritePrefix: 'zoqfotpik-stinger',
  color: 0xffffff,
  size: 15,
};
