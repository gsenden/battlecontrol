/// <reference lib="webworker" />

import init, { Battle } from 'game-logic-wasm';
import { PHYSICS_DELTA } from '../constants.js';
import type { BattleWorkerMessage, BattleWorkerResponse } from './battle-worker-types.js';

let battle: Battle | null = null;
let initPromise: Promise<unknown> | null = null;
let timerId: number | null = null;

function ensureInit() {
  if (!initPromise) {
    initPromise = init();
  }

  return initPromise;
}

function postSnapshot() {
  if (!battle) {
    return;
  }

  const message: BattleWorkerResponse = {
    type: 'snapshot',
    snapshot: battle.getSnapshot(),
  };
  self.postMessage(message);
}

function startTicking() {
  if (timerId !== null) {
    clearInterval(timerId);
  }

  timerId = self.setInterval(() => {
    battle?.tick(PHYSICS_DELTA);
    postSnapshot();
  }, PHYSICS_DELTA);
}

self.onmessage = async (event: MessageEvent<BattleWorkerMessage>) => {
  const message = event.data;

  if (message.type === 'initBattle') {
    await ensureInit();
    battle = new Battle(
      message.playerShipType,
      message.targetShipType,
      message.playerX,
      message.playerY,
      message.targetX,
      message.targetY,
      message.planetX,
      message.planetY,
      message.width,
      message.height,
    );
    startTicking();
    postSnapshot();
    return;
  }

  if (!battle) {
    return;
  }

  if (message.type === 'setPlayerInput') {
    battle.setPlayerInput(
      message.input.left,
      message.input.right,
      message.input.thrust,
      message.input.weapon,
      message.input.special,
    );
  } else if (message.type === 'setTargetInput') {
    battle.setTargetInput(
      message.input.left,
      message.input.right,
      message.input.thrust,
      message.input.weapon,
      message.input.special,
    );
  } else if (message.type === 'triggerTargetWeapon') {
    battle.triggerTargetWeapon();
  } else if (message.type === 'setPlayerWeaponTargetPoint') {
    battle.setPlayerWeaponTargetPoint(message.x, message.y);
  } else if (message.type === 'setPlayerWeaponTargetShip') {
    battle.setPlayerWeaponTargetShip();
  } else if (message.type === 'setPlayerSpecialTargetPoint') {
    battle.setPlayerSpecialTargetPoint(message.x, message.y);
  } else if (message.type === 'clearPlayerWeaponTarget') {
    battle.clearPlayerWeaponTarget();
  } else if (message.type === 'clearPlayerSpecialTarget') {
    battle.clearPlayerSpecialTarget();
  } else if (message.type === 'switchPlayerShip') {
    battle.switchPlayerShip(message.shipType);
    postSnapshot();
  }
};

export {};
