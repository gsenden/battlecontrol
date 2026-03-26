import type { ShipStats } from '../ship-stats.js';

export const YEHAT_TERMINATOR: ShipStats = {
  raceName: 'Yehat',
  shipClass: 'Terminator',
  captainNames: ['Heep-eep', 'Feep-eep', 'Reep-eep', 'Yeep-eep', 'Beep-eep', 'Eeep-eep', 'Meep-eep', 'Teep-eep', 'Jeep-eep', 'Leep-eep', 'Peep-eep', 'Weep-eep', 'Veep-eep', 'Geep-eep', 'Zeep-eep', 'Neep-eep'],
  cost: 23,

  mass: 3,
  thrustIncrement: 1.2,
  maxSpeed: 5,
  turnRate: Math.PI / 8,

  turnWait: 2,
  thrustWait: 2,
  weaponWait: 0,
  specialWait: 2,

  maxEnergy: 10,
  energyRegeneration: 2,
  energyWait: 6,
  weaponEnergyCost: 1,
  specialEnergyCost: 3,

  maxCrew: 20,

  spritePrefix: 'yehat-terminator',
  color: 0xffffff,
  size: 12,
};
