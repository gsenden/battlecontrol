import Phaser from 'phaser';
import { BATTLE_WIDTH, BATTLE_HEIGHT, PHYSICS_DELTA } from '../constants.js';
import { Ship } from '../entities/Ship.js';
import { HUMAN_CRUISER } from '../ships/ship-stats.js';

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

const shipModules = import.meta.glob('../assets/ships/human/cruiser-big-*.png', { eager: true, import: 'default' }) as Record<string, string>;
const planetModules = import.meta.glob('../assets/planets/*.png', { eager: true, import: 'default' }) as Record<string, string>;
const planetUrls = Object.values(planetModules);

const BIG_STAR_COUNT = 200;
const MED_STAR_COUNT = 400;
const SML_STAR_COUNT = 600;

const BIG_SCROLL_FACTOR = 0.25;
const MED_SCROLL_FACTOR = 0.125;
const SML_SCROLL_FACTOR = 0.0625;

// Gravity well constants (tuned from SC2 reference data)
const GRAVITY_THRESHOLD = 400;  // Distance in world units where gravity kicks in
const GRAVITY_FORCE = 0.0003;   // Force magnitude per frame

export class BattleScene extends Phaser.Scene {
  private ship!: Ship;
  private cursors!: Phaser.Types.Input.Keyboard.CursorKeys;
  private cameraTarget!: Phaser.GameObjects.Container;
  private planetX = BATTLE_WIDTH / 2;
  private planetY = BATTLE_HEIGHT / 2;
  private physicsAccumulator = 0;

  constructor() {
    super('BattleScene');
  }

  preload() {
    // Stars
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

    // Ship frames (16 rotations)
    const shipEntries = Object.entries(shipModules).sort(([a], [b]) => a.localeCompare(b));
    shipEntries.forEach(([, url], i) => {
      this.load.image(`human-cruiser-${i}`, url);
    });

    // Planets
    planetUrls.forEach((url, i) => {
      this.load.image(`planet-${i}`, url);
    });
  }

  create() {
    // Stars
    this.createStarLayer(
      ['star-big', 'star-misc-big-0', 'star-misc-big-1'],
      BIG_STAR_COUNT, BIG_SCROLL_FACTOR,
    );
    this.createStarLayer(
      ['star-med', 'star-misc-med-0', 'star-misc-med-1'],
      MED_STAR_COUNT, MED_SCROLL_FACTOR,
    );
    this.createStarLayer(
      ['star-sml', 'star-sml2', 'star-sml3', 'star-misc-sml-0', 'star-misc-sml-1'],
      SML_STAR_COUNT, SML_SCROLL_FACTOR,
    );

    // Planet in center
    const planetIndex = Math.floor(Math.random() * planetUrls.length);
    this.add.image(this.planetX, this.planetY, `planet-${planetIndex}`);

    // Ion trail particle texture (small orange circle, generated)
    const gfx = this.add.graphics();
    gfx.fillStyle(0xffffff);
    gfx.fillCircle(2, 2, 2);
    gfx.generateTexture('ion-particle', 4, 4);
    gfx.destroy();

    // Ship — spawn offset from center so it's not on the planet
    this.ship = new Ship(this, this.planetX + 300, this.planetY, HUMAN_CRUISER);

    // Camera follows an invisible container that tracks the ship position
    this.cameraTarget = this.add.container(this.ship.x, this.ship.y);
    this.cameras.main.setBounds(0, 0, BATTLE_WIDTH, BATTLE_HEIGHT);
    this.cameras.main.startFollow(this.cameraTarget);

    // Input
    this.cursors = this.input.keyboard!.createCursorKeys();
  }

  update(_time: number, delta: number) {
    const input = {
      left: this.cursors.left.isDown,
      right: this.cursors.right.isDown,
      thrust: this.cursors.up.isDown,
      weapon: false,
      special: false,
    };

    // Fixed timestep physics at 24fps
    this.physicsAccumulator += delta;
    while (this.physicsAccumulator >= PHYSICS_DELTA) {
      this.applyGravity();
      this.ship.physicsUpdate(input);
      this.physicsAccumulator -= PHYSICS_DELTA;
    }

    // Visual update every render frame
    this.ship.renderUpdate();
    this.cameraTarget.setPosition(this.ship.x, this.ship.y);
  }

  private applyGravity() {
    const dx = this.planetX - this.ship.x;
    const dy = this.planetY - this.ship.y;
    const dist = Math.sqrt(dx * dx + dy * dy);

    if (dist < GRAVITY_THRESHOLD && dist > 0) {
      const fx = (dx / dist) * GRAVITY_FORCE;
      const fy = (dy / dist) * GRAVITY_FORCE;
      this.matter.body.applyForce(this.ship.body, this.ship.body.position, { x: fx, y: fy });
    }
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
