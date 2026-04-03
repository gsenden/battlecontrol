import Phaser from 'phaser';
import { BattleScene } from './scenes/BattleScene.js';
import { initGameLogic } from './game-logic.js';

const gameElement = document.getElementById('game');

if (!gameElement) {
  throw new Error('Missing #game mount point');
}

initGameLogic().then(() => {
  const config: Phaser.Types.Core.GameConfig = {
    type: Phaser.AUTO,
    width: gameElement.clientWidth,
    height: gameElement.clientHeight,
    parent: 'game',
    backgroundColor: '#000000',
    scene: [BattleScene],
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
