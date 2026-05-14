import type { BattleShipSnapshot } from '../workers/battle-worker-types.js';

export function closestLivingOpponentDistance(
  player: Pick<BattleShipSnapshot, 'id' | 'x' | 'y'>,
  ships: BattleShipSnapshot[],
): number | null {
  let closestDistance: number | null = null;

  for (const ship of ships) {
    if (ship.id === player.id || ship.dead) {
      continue;
    }
    const dx = ship.x - player.x;
    const dy = ship.y - player.y;
    const distance = Math.sqrt((dx * dx) + (dy * dy));
    if (closestDistance === null || distance < closestDistance) {
      closestDistance = distance;
    }
  }

  return closestDistance;
}
