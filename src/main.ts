import Phaser from 'phaser';
import { BattleScene } from './scenes/BattleScene.js';
import { MatterScene } from './scenes/MatterScene.js';
import { assertVersionSync, initGameLogic } from './game-logic.js';
import { APP_VERSION } from './version.js';
import { mountDebugOverlay, toggleDebugUi } from './debug-overlay.js';

const gameElement = document.getElementById('game');

if (!gameElement) {
  throw new Error('Missing #game mount point');
}

document.title = import.meta.env.DEV ? 'Battle Control DEV' : 'Battle Control';

function mountVersionBadge() {
  const badge = document.createElement('div');
  badge.id = 'version-badge';
  badge.textContent = `v${APP_VERSION}`;
  document.body.appendChild(badge);
}

function mountStartOverlay() {
  const overlay = document.createElement('div');
  overlay.id = 'start-overlay';

  const button = document.createElement('button');
  button.id = 'start-game-button';
  button.type = 'button';
  button.textContent = 'Loading...';
  button.disabled = true;

  const onSceneReady = () => {
    button.disabled = false;
    button.textContent = 'Start Game';
  };

  window.addEventListener('battlecontrol:scene-ready', onSceneReady, { once: true });
  button.addEventListener('pointerdown', (event) => {
    event.preventDefault();
    event.stopPropagation();
  });
  button.addEventListener('click', (event) => {
    event.preventDefault();
    event.stopPropagation();

    if (button.disabled) {
      return;
    }

    window.removeEventListener('battlecontrol:scene-ready', onSceneReady);
    window.setTimeout(() => {
      overlay.remove();
      window.dispatchEvent(new Event('battlecontrol:start-game'));
    }, 0);
  });

  overlay.appendChild(button);
  document.body.appendChild(overlay);
}

initGameLogic().then(() => {
  assertVersionSync();
  mountVersionBadge();
  mountDebugOverlay();
  window.addEventListener('keydown', (event) => {
    if (event.code !== 'Backquote') {
      return;
    }

    event.preventDefault();
    toggleDebugUi();
  });

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

  mountStartOverlay();
  new Phaser.Game(config);
});
