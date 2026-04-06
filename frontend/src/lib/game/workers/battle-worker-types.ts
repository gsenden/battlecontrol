import type { ShipInput } from '../ships/ship-stats.js';

export interface BattleShipSnapshot {
  id: number;
  x: number;
  y: number;
  vx: number;
  vy: number;
  crew: number;
  energy: number;
  facing: number;
  thrusting: boolean;
  dead: boolean;
  cloaked: boolean;
  texturePrefix: string;
}

export interface ProjectileSnapshot {
  id: number;
  x: number;
  y: number;
  vx: number;
  vy: number;
  life: number;
  texturePrefix: string;
}

export interface ExplosionSnapshot {
  id: number;
  x: number;
  y: number;
  frameIndex: number;
  texturePrefix: string;
}

export interface MeteorSnapshot {
  id: number;
  x: number;
  y: number;
  vx: number;
  vy: number;
  frameIndex: number;
  texturePrefix: string;
}

export interface LaserSnapshot {
  id: number;
  startX: number;
  startY: number;
  endX: number;
  endY: number;
  color: number;
  width: number;
}

export interface AudioEventSnapshot {
  key: string;
}

export interface BattleSnapshot {
  player: BattleShipSnapshot;
  target: BattleShipSnapshot;
  meteors: MeteorSnapshot[];
  projectiles: ProjectileSnapshot[];
  explosions: ExplosionSnapshot[];
  lasers: LaserSnapshot[];
  audioEvents: AudioEventSnapshot[];
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
  | { type: 'triggerTargetWeapon' }
  | { type: 'setPlayerWeaponTargetPoint'; x: number; y: number }
  | { type: 'setPlayerWeaponTargetShip' }
  | { type: 'setPlayerSpecialTargetPoint'; x: number; y: number }
  | { type: 'clearPlayerWeaponTarget' }
  | { type: 'clearPlayerSpecialTarget' }
  | { type: 'switchPlayerShip'; shipType: string };

export type BattleWorkerResponse =
  | { type: 'snapshot'; snapshot: BattleSnapshot };
