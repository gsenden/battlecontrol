import init, { GameLogic } from 'game-logic-wasm';

let instance: GameLogic;

export async function initGameLogic(): Promise<GameLogic> {
  await init();
  instance = new GameLogic();
  return instance;
}

export function getGameLogic(): GameLogic {
  return instance;
}
