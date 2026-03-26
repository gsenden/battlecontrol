import type { ShipStats } from '../ship-stats.js';

export const MYCON_PODSHIP: ShipStats = {
  raceName: 'Mycon',
  shipClass: 'Podship',
  captainNames: ['Blort', 'Chupp', 'Floos', 'Glish', 'Glob', 'Glush', 'Plork', 'Shlish', 'Shlupp', 'Slingy', 'Sploozo', 'Spork', 'Uffo', 'Yush', 'Zaloop', 'Znuff'],
  cost: 21,

  mass: 7,
  thrustIncrement: 1.8,
  maxSpeed: 4.5,
  turnRate: Math.PI / 8,

  turnWait: 6,
  thrustWait: 6,
  weaponWait: 5,
  specialWait: 0,

  maxEnergy: 40,
  energyRegeneration: 1,
  energyWait: 4,
  weaponEnergyCost: 20,
  specialEnergyCost: 40,

  maxCrew: 20,

  spritePrefix: 'mycon-podship',
  color: 0xffffff,
  size: 18,
};
