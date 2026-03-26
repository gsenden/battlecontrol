import type { ShipStats } from '../ship-stats.js';

export const ANDROSYNTH_GUARDIAN: ShipStats = {
  raceName: 'Androsynth',
  shipClass: 'Guardian',
  captainNames: ['BOOJI-1', 'DORN-3', 'BIM-XT', 'JOR-15', '976-KILL', 'KORB-7B', 'XR4-TI', 'CRC-16', 'BHS-79', 'DOS-1.0', 'ME-262', 'AK-47', '1040-EZ', 'NECRO-99', 'HAL-2001', 'SR-71'],
  cost: 15,

  mass: 6,
  thrustIncrement: 0.6,
  maxSpeed: 4,
  turnRate: Math.PI / 8,

  turnWait: 4,
  thrustWait: 0,
  weaponWait: 0,
  specialWait: 0,

  maxEnergy: 24,
  energyRegeneration: 1,
  energyWait: 8,
  weaponEnergyCost: 3,
  specialEnergyCost: 2,

  maxCrew: 20,

  spritePrefix: 'androsynth-guardian',
  color: 0xffffff,
  size: 16,
};
