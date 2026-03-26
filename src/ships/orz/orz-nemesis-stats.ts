import type { ShipStats } from '../ship-stats.js';

export const ORZ_NEMESIS: ShipStats = {
  raceName: 'Orz',
  shipClass: 'Nemesis',
  captainNames: ['*Wet*', '*Happy*', '*Frumple*', '*Camper*', '*Loner*', '*Dancer*', '*Singer*', '*Heavy*', '*NewBoy*', '*FatFun*', '*Pepper*', '*Hungry*', '*Deep*', '*Smell*', '*Juice*', '*Squirt*'],
  cost: 23,

  mass: 4,
  thrustIncrement: 1,
  maxSpeed: 5.8,
  turnRate: Math.PI / 8,

  turnWait: 1,
  thrustWait: 0,
  weaponWait: 4,
  specialWait: 12,

  maxEnergy: 20,
  energyRegeneration: 1,
  energyWait: 6,
  weaponEnergyCost: 6,
  specialEnergyCost: 0,

  maxCrew: 16,

  spritePrefix: 'orz-nemesis',
  color: 0xffffff,
  size: 13,
};
