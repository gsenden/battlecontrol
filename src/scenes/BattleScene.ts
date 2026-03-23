import Phaser from 'phaser';
import { BATTLE_WIDTH, BATTLE_HEIGHT } from '../constants.js';

import starBig from '../assets/stars/stars-000.png';
import starMed from '../assets/stars/stars-001.png';
import starSml from '../assets/stars/stars-002.png';
import starSml2 from '../assets/stars/stars-003.png';
import starSml3 from '../assets/stars/stars-004.png';
import starMiscBig0 from '../assets/stars/stars-misc-big-000.png';
import starMiscBig1 from '../assets/stars/stars-misc-big-001.png';
import starMiscMed0 from '../assets/stars/stars-misc-med-000.png';
import starMiscMed1 from '../assets/stars/stars-misc-med-001.png';
import starMiscSml0 from '../assets/stars/stars-misc-sml-000.png';
import starMiscSml1 from '../assets/stars/stars-misc-sml-001.png';

const planetModules = import.meta.glob('../assets/planets/*.png', { eager: true, import: 'default' }) as Record<string, string>;
const planetUrls = Object.values(planetModules);

const BIG_STAR_COUNT = 200;
const MED_STAR_COUNT = 400;
const SML_STAR_COUNT = 600;

// Parallax scroll factors: smaller = moves slower = feels further away
const BIG_SCROLL_FACTOR = 0.25;
const MED_SCROLL_FACTOR = 0.125;
const SML_SCROLL_FACTOR = 0.0625;

const CAMERA_SPEED = 10;

export class BattleScene extends Phaser.Scene {
  private cursors!: Phaser.Types.Input.Keyboard.CursorKeys;

  constructor() {
    super('BattleScene');
  }

  preload() {
    this.load.image('star-big', starBig);
    this.load.image('star-med', starMed);
    this.load.image('star-sml', starSml);
    this.load.image('star-sml2', starSml2);
    this.load.image('star-sml3', starSml3);
    this.load.image('star-misc-big-0', starMiscBig0);
    this.load.image('star-misc-big-1', starMiscBig1);
    this.load.image('star-misc-med-0', starMiscMed0);
    this.load.image('star-misc-med-1', starMiscMed1);
    this.load.image('star-misc-sml-0', starMiscSml0);
    this.load.image('star-misc-sml-1', starMiscSml1);

    planetUrls.forEach((url, i) => {
      this.load.image(`planet-${i}`, url);
    });
  }

  create() {
    // Stars behind everything, spread over the world
    this.createStarLayer(
      ['star-big', 'star-misc-big-0', 'star-misc-big-1'],
      BIG_STAR_COUNT,
      BIG_SCROLL_FACTOR,
    );
    this.createStarLayer(
      ['star-med', 'star-misc-med-0', 'star-misc-med-1'],
      MED_STAR_COUNT,
      MED_SCROLL_FACTOR,
    );
    this.createStarLayer(
      ['star-sml', 'star-sml2', 'star-sml3', 'star-misc-sml-0', 'star-misc-sml-1'],
      SML_STAR_COUNT,
      SML_SCROLL_FACTOR,
    );

    // Random planet in center of the world
    const planetIndex = Math.floor(Math.random() * planetUrls.length);
    this.add.image(BATTLE_WIDTH / 2, BATTLE_HEIGHT / 2, `planet-${planetIndex}`);

    // Camera: bounded to world, start centered
    this.cameras.main.setBounds(0, 0, BATTLE_WIDTH, BATTLE_HEIGHT);
    this.cameras.main.centerOn(BATTLE_WIDTH / 2, BATTLE_HEIGHT / 2);

    // Temp: arrow keys to fly camera around for testing
    this.cursors = this.input.keyboard!.createCursorKeys();
  }

  update() {
    if (this.cursors.left.isDown) this.cameras.main.scrollX -= CAMERA_SPEED;
    if (this.cursors.right.isDown) this.cameras.main.scrollX += CAMERA_SPEED;
    if (this.cursors.up.isDown) this.cameras.main.scrollY -= CAMERA_SPEED;
    if (this.cursors.down.isDown) this.cameras.main.scrollY += CAMERA_SPEED;
  }

  private createStarLayer(textures: string[], count: number, scrollFactor: number) {
    for (let i = 0; i < count; i++) {
      const texture = textures[Math.floor(Math.random() * textures.length)];
      const x = Math.random() * BATTLE_WIDTH;
      const y = Math.random() * BATTLE_HEIGHT;
      const star = this.add.image(x, y, texture);
      star.setScrollFactor(scrollFactor);
    }
  }
}
