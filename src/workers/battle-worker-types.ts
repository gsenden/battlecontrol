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
}

export interface BattleSnapshot {
  player: BattleShipSnapshot;
  target: BattleShipSnapshot;
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
  | { type: 'switchPlayerShip'; shipType: string };

export type BattleWorkerResponse =
  | { type: 'snapshot'; snapshot: BattleSnapshot };
