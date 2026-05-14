import type { AudioEventSnapshot, BattleSnapshot } from '../workers/battle-worker-types.js';

const POSITION_RECONCILE_ALPHA = 0.35;
const VELOCITY_RECONCILE_ALPHA = 0.2;
const FACING_RECONCILE_STEP = 0.25;

export function reconcileBattleSnapshot(
  predicted: BattleSnapshot,
  authoritative: BattleSnapshot | null,
  authoritativeAudioEvents: AudioEventSnapshot[] = [],
): BattleSnapshot {
  if (!authoritative) {
    return predicted;
  }

  const snapshot: BattleSnapshot = {
    ...predicted,
    ships: predicted.ships.map((ship) => ({ ...ship })),
    player: { ...predicted.player },
    target: { ...predicted.target },
    projectiles: predicted.projectiles.map((projectile) => ({ ...projectile })),
    meteors: predicted.meteors.map((meteor) => ({ ...meteor })),
    explosions: authoritative.explosions.map((explosion) => ({ ...explosion })),
    lasers: authoritative.lasers.map((laser) => ({ ...laser })),
    audioEvents: [
      ...predicted.audioEvents.map((event) => ({ ...event })),
      ...authoritativeAudioEvents.map((event) => ({ ...event })),
    ],
  };

  reconcileShipSnapshot(snapshot.player, authoritative.player, POSITION_RECONCILE_ALPHA, VELOCITY_RECONCILE_ALPHA);
  reconcileShipSnapshot(snapshot.target, authoritative.target, 1, 1);
  snapshot.ships = reconcileShips(snapshot.ships, authoritative.ships);

  const authProjectiles = new Map(authoritative.projectiles.map((projectile) => [projectile.id, projectile]));
  snapshot.projectiles = snapshot.projectiles.map((projectile) => {
    const auth = authProjectiles.get(projectile.id);
    if (!auth) {
      return projectile;
    }
    return {
      ...projectile,
      x: linear(projectile.x, auth.x, POSITION_RECONCILE_ALPHA),
      y: linear(projectile.y, auth.y, POSITION_RECONCILE_ALPHA),
      vx: linear(projectile.vx, auth.vx, VELOCITY_RECONCILE_ALPHA),
      vy: linear(projectile.vy, auth.vy, VELOCITY_RECONCILE_ALPHA),
      life: auth.life,
      texturePrefix: auth.texturePrefix,
    };
  });

  for (const authProjectile of authoritative.projectiles) {
    if (!snapshot.projectiles.some((projectile) => projectile.id === authProjectile.id)) {
      snapshot.projectiles.push({ ...authProjectile });
    }
  }

  const authMeteors = new Map(authoritative.meteors.map((meteor) => [meteor.id, meteor]));
  snapshot.meteors = snapshot.meteors.map((meteor) => {
    const auth = authMeteors.get(meteor.id);
    if (!auth) {
      return meteor;
    }
    return {
      ...meteor,
      x: linear(meteor.x, auth.x, POSITION_RECONCILE_ALPHA),
      y: linear(meteor.y, auth.y, POSITION_RECONCILE_ALPHA),
      vx: linear(meteor.vx, auth.vx, VELOCITY_RECONCILE_ALPHA),
      vy: linear(meteor.vy, auth.vy, VELOCITY_RECONCILE_ALPHA),
      frameIndex: auth.frameIndex,
    };
  });

  for (const authMeteor of authoritative.meteors) {
    if (!snapshot.meteors.some((meteor) => meteor.id === authMeteor.id)) {
      snapshot.meteors.push({ ...authMeteor });
    }
  }

  return snapshot;
}

function reconcileShips(
  predictedShips: BattleSnapshot['ships'],
  authoritativeShips: BattleSnapshot['ships'],
) {
  const ships = predictedShips.map((ship) => ({ ...ship }));
  const authShips = new Map(authoritativeShips.map((ship) => [ship.id, ship]));
  for (const ship of ships) {
    const authoritative = authShips.get(ship.id);
    if (!authoritative) {
      continue;
    }
    reconcileShipSnapshot(ship, authoritative, 1, 1);
  }
  for (const authoritative of authoritativeShips) {
    if (!ships.some((ship) => ship.id === authoritative.id)) {
      ships.push({ ...authoritative });
    }
  }
  return ships;
}

function reconcileShipSnapshot(
  ship: BattleSnapshot['player'],
  authoritative: BattleSnapshot['player'],
  positionAlpha: number,
  velocityAlpha: number,
) {
  ship.x = linear(ship.x, authoritative.x, positionAlpha);
  ship.y = linear(ship.y, authoritative.y, positionAlpha);
  ship.vx = linear(ship.vx, authoritative.vx, velocityAlpha);
  ship.vy = linear(ship.vy, authoritative.vy, velocityAlpha);
  ship.facing = rotateTo(ship.facing, authoritative.facing, FACING_RECONCILE_STEP);
  ship.turretFacing = rotateTo(ship.turretFacing, authoritative.turretFacing, FACING_RECONCILE_STEP);
  ship.crew = authoritative.crew;
  ship.energy = authoritative.energy;
  ship.dead = authoritative.dead;
  ship.cloaked = authoritative.cloaked;
  ship.thrusting = authoritative.thrusting;
  ship.texturePrefix = authoritative.texturePrefix;
}

function linear(start: number, end: number, amount: number) {
  return start + ((end - start) * amount);
}

function rotateTo(current: number, target: number, step: number) {
  const delta = shortestAngleDelta(current, target);
  if (Math.abs(delta) <= step) {
    return target;
  }
  return current + (Math.sign(delta) * step);
}

function shortestAngleDelta(current: number, target: number) {
  let delta = target - current;
  while (delta <= -Math.PI) {
    delta += Math.PI * 2;
  }
  while (delta > Math.PI) {
    delta -= Math.PI * 2;
  }
  return delta;
}
