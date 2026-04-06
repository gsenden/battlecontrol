import init, { Battle, GameLogic, MatterWorld, getVersion } from 'game-logic-wasm';
import { APP_VERSION } from './version.js';

let instance: GameLogic;

export async function initGameLogic(): Promise<GameLogic> {
  await init();
  instance = new GameLogic();
  return instance;
}

export function assertVersionSync() {
  const rustVersion = getVersion();
  if (rustVersion !== APP_VERSION) {
    throw new Error(`Version mismatch: ts=${APP_VERSION}, rust=${rustVersion}`);
  }
}

export function getGameLogic(): GameLogic {
  return instance;
}

export function createMatterWorld(): MatterWorld {
  return new MatterWorld();
}

export function createBattle(
  playerShipType: string,
  targetShipType: string,
  playerX: number,
  playerY: number,
  targetX: number,
  targetY: number,
  planetX: number,
  planetY: number,
  width: number,
  height: number,
): Battle {
  return new Battle(
    playerShipType,
    targetShipType,
    playerX,
    playerY,
    targetX,
    targetY,
    planetX,
    planetY,
    width,
    height,
  );
}
