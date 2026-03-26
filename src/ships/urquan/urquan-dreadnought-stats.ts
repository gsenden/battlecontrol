import type { ShipStats } from '../ship-stats.js';

export const URQUAN_DREADNOUGHT: ShipStats = {
  raceName: 'Ur-Quan',
  shipClass: 'Dreadnought',
  captainNames: ['Lord 999', 'Lord 342', 'Lord 88', 'Lord 156', 'Lord 43', 'Lord 412', 'Lord 666', 'Lord 18', 'Lord 237', 'Lord 89', 'Lord 3', 'Lord 476', 'Lord 103', 'Lord 783', 'Lord 52', 'Lord 21'],
  cost: 30,

  mass: 10,
  thrustIncrement: 1.2,
  maxSpeed: 5,
  turnRate: Math.PI / 8,

  turnWait: 4,
  thrustWait: 6,
  weaponWait: 6,
  specialWait: 9,

  maxEnergy: 42,
  energyRegeneration: 1,
  energyWait: 4,
  weaponEnergyCost: 6,
  specialEnergyCost: 8,

  maxCrew: 42,

  spritePrefix: 'urquan-dreadnought',
  color: 0xffffff,
  size: 22,
};
