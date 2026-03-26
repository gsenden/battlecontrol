import type { ShipStats } from '../ship-stats.js';

export const MMRNMHRM_XFORM: ShipStats = {
  raceName: 'Mmrnmhrm',
  shipClass: 'X-Form',
  captainNames: ['Qir-nha', 'Jhe-qir', 'Qua-rhna', 'Mn-quah', 'Nrna-mha', 'Um-hrh', 'Hm-nhuh', 'Rrma-hrn', 'Jra-nr', 'Ur-mfrs', 'Qua-qir', 'Mrm-na', 'Jhe-mhr', 'Hmr-hun', 'Nhuh-na', 'Hrnm-hm'],
  cost: 19,

  mass: 3,
  thrustIncrement: 1,
  maxSpeed: 3.3,
  turnRate: Math.PI / 8,

  turnWait: 2,
  thrustWait: 1,
  weaponWait: 0,
  specialWait: 0,

  maxEnergy: 10,
  energyRegeneration: 2,
  energyWait: 6,
  weaponEnergyCost: 1,
  specialEnergyCost: 10,

  maxCrew: 20,

  spritePrefix: 'mmrnmhrm-xform',
  color: 0xffffff,
  size: 12,
};
