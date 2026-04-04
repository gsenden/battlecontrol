import type { ShipInput } from '../ships/ship-stats.js';

export interface BattleShipSnapshot {
  x: number;
  y: number;
  vx: number;
  vy: number;
  crew: number;
  energy: number;
  facing: number;
  thrusting: boolean;
  dead: boolean;
}

export interface ProjectileSnapshot {
  x: number;
  y: number;
  vx: number;
  vy: number;
  life: number;
  texturePrefix: string;
}

export interface ExplosionSnapshot {
  x: number;
  y: number;
  frameIndex: number;
  texturePrefix: string;
}

export interface BattleSnapshot {
  player: BattleShipSnapshot;
  target: BattleShipSnapshot;
  projectiles: ProjectileSnapshot[];
  explosions: ExplosionSnapshot[];
}

export type BattleWorkerMessage =
  | {
    type: 'initBattle';
    playerShipType: string;
    targetShipType: string;
    playerX: number;
    playerY: number;
    targetX: number;
    targetY: number;
    planetX: number;
    planetY: number;
    width: number;
    height: number;
  }
  | { type: 'setPlayerInput'; input: ShipInput }
  | { type: 'setTargetInput'; input: ShipInput }
  | { type: 'setPlayerWeaponTargetPoint'; x: number; y: number }
  | { type: 'setPlayerWeaponTargetShip' }
  | { type: 'clearPlayerWeaponTarget' }
  | { type: 'switchPlayerShip'; shipType: string };

export type BattleWorkerResponse =
  | { type: 'snapshot'; snapshot: BattleSnapshot };
