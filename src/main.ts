import Phaser from 'phaser';
import { BattleScene } from './scenes/BattleScene.js';
import { MatterScene } from './scenes/MatterScene.js';
import { assertVersionSync, initGameLogic } from './game-logic.js';
import { APP_VERSION } from './version.js';

const gameElement = document.getElementById('game');

if (!gameElement) {
  throw new Error('Missing #game mount point');
}

function mountVersionBadge() {
  const badge = document.createElement('div');
  badge.id = 'version-badge';
  badge.textContent = `v${APP_VERSION}`;
  document.body.appendChild(badge);
}

initGameLogic().then(() => {
  assertVersionSync();
  mountVersionBadge();

  const initialScene = window.location.hash === '#matter' ? MatterScene : BattleScene;
  const config: Phaser.Types.Core.GameConfig = {
    type: Phaser.AUTO,
    width: gameElement.clientWidth,
    height: gameElement.clientHeight,
    parent: 'game',
    backgroundColor: '#000000',
    scene: [initialScene],
    scale: {
      mode: Phaser.Scale.RESIZE,
      autoCenter: Phaser.Scale.CENTER_BOTH,
    },
    physics: {
      default: 'matter',
      matter: {
        gravity: { x: 0, y: 0 },
        debug: false,
      },
    },
  };

  new Phaser.Game(config);
});
