import type { ShipStats } from '../ship-stats.js';

export const DRUUGE_MAULER: ShipStats = {
  raceName: 'Druuge',
  shipClass: 'Mauler',
  captainNames: ['Tuuga', 'Siinur', 'Kaapo', 'Juugl', 'Paato', 'Feezo', 'Maad', 'Moola', 'Kooli', 'Faazur', 'Zooto', 'Biitur', 'Duulard', 'Piini', 'Soopi', 'Peeru'],
  cost: 17,

  mass: 5,
  thrustIncrement: 0.4,
  maxSpeed: 3.3,
  turnRate: Math.PI / 8,

  turnWait: 4,
  thrustWait: 1,
  weaponWait: 10,
  specialWait: 30,

  maxEnergy: 32,
  energyRegeneration: 1,
  energyWait: 50,
  weaponEnergyCost: 4,
  specialEnergyCost: 16,

  maxCrew: 14,

  spritePrefix: 'druuge-mauler',
  color: 0xffffff,
  size: 15,
};
