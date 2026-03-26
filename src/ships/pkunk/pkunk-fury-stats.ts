import type { ShipStats } from '../ship-stats.js';

export const PKUNK_FURY: ShipStats = {
  raceName: 'Pkunk',
  shipClass: 'Fury',
  captainNames: ['Awwky', 'Tweety', 'WudStok', 'Poppy', 'Brakky', 'Hooter', 'Buzzard', 'Polly', 'Ernie', 'Yompin', 'Fuzzy', 'Raven', 'Crow', 'Jay', 'Screech', 'Twitter'],
  cost: 20,

  mass: 1,
  thrustIncrement: 3.2,
  maxSpeed: 10.7,
  turnRate: Math.PI / 8,

  turnWait: 0,
  thrustWait: 0,
  weaponWait: 0,
  specialWait: 16,

  maxEnergy: 12,
  energyRegeneration: 0,
  energyWait: 0,
  weaponEnergyCost: 1,
  specialEnergyCost: 2,

  maxCrew: 8,

  spritePrefix: 'pkunk-fury',
  color: 0xffffff,
  size: 12,
};
