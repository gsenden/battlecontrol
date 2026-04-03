import Phaser from 'phaser';
import { createMatterWorld } from '../game-logic.js';
import type { MatterWorld } from 'game-logic-wasm';

const shipModules = import.meta.glob('../assets/ships/*/*-big-000.png', { eager: true, import: 'default' }) as Record<string, string>;

const MATTER_SCENE_WIDTH = 1280;
const MATTER_SCENE_HEIGHT = 720;

export class MatterScene extends Phaser.Scene {
  private matterWorld!: MatterWorld;
  private shipSprites: Phaser.GameObjects.Image[] = [];
  private label!: Phaser.GameObjects.Text;
  private cursors!: Phaser.Types.Input.Keyboard.CursorKeys;
  private bodyAngles = new Map<number, number>();

  constructor() {
    super('MatterScene');
  }

  preload() {
    for (const [path, url] of Object.entries(shipModules)) {
      const file = path.split('/').pop()!.replace('.png', '');
      const folder = path.split('/').at(-2)!;
      this.load.image(`${folder}-${file}`, url);
    }
  }

  create() {
    this.cameras.main.setBackgroundColor('#02060c');
    this.matterWorld = createMatterWorld();
    this.matterWorld.setupDemo();

    this.add.rectangle(MATTER_SCENE_WIDTH / 2, MATTER_SCENE_HEIGHT / 2, 420, 220)
      .setStrokeStyle(2, 0x6f89a6, 0.9);

    this.label = this.add.text(32, 24, 'matter-js-rs scene', {
      color: '#d7e3ff',
      fontFamily: 'monospace',
      fontSize: '18px',
    });

    this.shipSprites = [
      this.add.image(0, 0, 'human-cruiser-big-000'),
      this.add.image(0, 0, 'chmmr-avatar-big-000'),
    ];

    this.shipSprites[0].setScale(0.8);
    this.shipSprites[1].setScale(0.8);

    const hint = this.add.text(32, 56, 'Open met #matter om deze scene direct te starten', {
      color: '#8ea3bf',
      fontFamily: 'monospace',
      fontSize: '12px',
    });
    hint.setAlpha(0.9);

    this.cursors = this.input.keyboard!.createCursorKeys();
  }

  update(_time: number, delta: number) {
    if (this.cursors.left.isDown) {
      this.matterWorld.rotateBody(0, -0.06);
    }

    if (this.cursors.right.isDown) {
      this.matterWorld.rotateBody(0, 0.06);
    }

    if (this.cursors.up.isDown) {
      const angle = this.bodyAngles.get(0) ?? 0;
      this.matterWorld.applyThrust(0, Math.cos(angle) * 0.005, Math.sin(angle) * 0.005);
    }

    const state = this.matterWorld.step(delta) as {
      bodies: Array<{ id: number; x: number; y: number; vx: number; vy: number; angle: number }>;
      collisions: Array<{ bodyA: number; bodyB: number }>;
    };

    state.bodies.forEach((body, index) => {
      this.bodyAngles.set(body.id, body.angle);

      const sprite = this.shipSprites[index];
      if (!sprite) {
        return;
      }

      sprite.setPosition(120 + body.x, 160 + body.y);
      sprite.setRotation(body.angle);
    });

    this.label.setText(
      `matter-js-rs scene\n${state.collisions.length > 0 ? 'collision' : 'no collision'}\nleft/right = turn, up = thrust`,
    );
  }
}
