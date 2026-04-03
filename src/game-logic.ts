import init, { GameLogic, MatterWorld } from 'game-logic-wasm';

let instance: GameLogic;

export async function initGameLogic(): Promise<GameLogic> {
  await init();
  instance = new GameLogic();
  return instance;
}

export function getGameLogic(): GameLogic {
  return instance;
}

export function createMatterWorld(): MatterWorld {
  return new MatterWorld();
}
