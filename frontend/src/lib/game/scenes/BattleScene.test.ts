import { describe, expect, it } from 'vitest';
import { closestLivingOpponentDistance } from './closest-opponent.js';
import { reconcileBattleSnapshot } from './snapshot-reconciliation.js';
import type {
  BattleShipSnapshot,
  BattleSnapshot,
  MeteorSnapshot,
  ProjectileSnapshot,
} from '../workers/battle-worker-types.js';

describe('reconcileBattleSnapshot', () => {
  it('zooms against the closest living opponent', () => {
    const distance = closestLivingOpponentDistance(
      ship(2, { x: 100, y: 100 }),
      [
        ship(1, { x: 400, y: 100 }),
        ship(2, { x: 100, y: 100 }),
        ship(3, { x: 180, y: 100 }),
        ship(4, { x: 120, y: 100, dead: true }),
      ],
    );

    expect(distance).toBe(80);
  });

  it('keeps authoritative-only ships visible', () => {
    const result = reconcileBattleSnapshot(
      battleSnapshot({
        ships: [ship(1), ship(2)],
      }),
      battleSnapshot({
        ships: [ship(1), ship(2), ship(3)],
      }),
    );

    expect(result.ships.map((ship) => ship.id)).toEqual([1, 2, 3]);
  });

  it('applies authoritative target positions immediately', () => {
    const result = reconcileBattleSnapshot(
      battleSnapshot({
        target: ship(2, { x: 0 }),
      }),
      battleSnapshot({
        target: ship(2, { x: 100 }),
      }),
    );

    expect(result.target.x).toBe(100);
  });

  it('keeps authoritative-only projectiles visible', () => {
    const result = reconcileBattleSnapshot(
      battleSnapshot({
        projectiles: [
          projectile(1),
          projectile(2),
        ],
      }),
      battleSnapshot({
        projectiles: [
          projectile(1),
          projectile(3),
        ],
      }),
    );

    expect(result.projectiles.map((projectile) => projectile.id)).toEqual([1, 2, 3]);
  });

  it('keeps authoritative-only meteors visible', () => {
    const result = reconcileBattleSnapshot(
      battleSnapshot({
        meteors: [
          meteor(1),
          meteor(2),
        ],
      }),
      battleSnapshot({
        meteors: [
          meteor(1),
          meteor(3),
        ],
      }),
    );

    expect(result.meteors.map((meteor) => meteor.id)).toEqual([1, 2, 3]);
  });

  it('does not replay authoritative audio without pending audio events', () => {
    const result = reconcileBattleSnapshot(
      battleSnapshot(),
      battleSnapshot({ audioEvents: [{ key: 'battle-boom' }] }),
    );

    expect(result.audioEvents).toEqual([]);
  });

  it('plays pending authoritative audio once through the reconciled snapshot', () => {
    const result = reconcileBattleSnapshot(
      battleSnapshot({ audioEvents: [{ key: 'human-primary' }] }),
      battleSnapshot(),
      [{ key: 'battle-boom' }],
    );

    expect(result.audioEvents.map((audioEvent) => audioEvent.key)).toEqual(['human-primary', 'battle-boom']);
  });
});

function battleSnapshot(overrides: Partial<BattleSnapshot> = {}): BattleSnapshot {
  return {
    ships: [ship(1), ship(2)],
    player: ship(1),
    target: ship(2),
    meteors: [],
    projectiles: [],
    explosions: [],
    lasers: [],
    audioEvents: [],
    ...overrides,
  };
}

function ship(id: number, overrides: Partial<BattleShipSnapshot> = {}): BattleShipSnapshot {
  return {
    id,
    x: 0,
    y: 0,
    vx: 0,
    vy: 0,
    crew: 1,
    energy: 1,
    facing: 0,
    turretFacing: 0,
    thrusting: false,
    dead: false,
    cloaked: false,
    texturePrefix: 'human-cruiser',
    ...overrides,
  };
}

function projectile(id: number, overrides: Partial<ProjectileSnapshot> = {}): ProjectileSnapshot {
  return {
    id,
    x: 0,
    y: 0,
    vx: 0,
    vy: 0,
    life: 1,
    texturePrefix: 'missile',
    ...overrides,
  };
}

function meteor(id: number, overrides: Partial<MeteorSnapshot> = {}): MeteorSnapshot {
  return {
    id,
    x: 0,
    y: 0,
    vx: 0,
    vy: 0,
    frameIndex: 0,
    texturePrefix: 'battle-asteroid',
    ...overrides,
  };
}
