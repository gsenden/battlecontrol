import Phaser from 'phaser';
import { BATTLE_HEIGHT, BATTLE_WIDTH, NUM_FACINGS } from '../constants.js';
import type { ShipStats } from '../ships/ship-stats.js';
import { getShipRenderScale } from '../ui/hud-state.svelte.js';
import type { BattleShipSnapshot } from '../workers/battle-worker-types.js';

const ION_COLORS = [
  0xffaa00,
  0xff8800,
  0xff7000,
  0xff5000,
  0xff3800,
  0xff1800,
  0xff0000,
  0xd80000,
  0xb80000,
  0x980000,
  0x780000,
  0x580000,
];

interface IonParticle {
  sprite: Phaser.GameObjects.Image;
  colorIndex: number;
}

export class Ship {
  readonly stats: ShipStats;

  private scene: Phaser.Scene;
  private sprite: Phaser.GameObjects.Image;
  private ghostSprites: Phaser.GameObjects.Image[] = [];
  private ionTrail: IonParticle[] = [];
  private spritePrefix: string;
  private readonly renderScaleMultiplier: number;

  crew: number;
  energy: number;
  facing: number;
  x: number;
  y: number;
  vx: number;
  vy: number;
  dead: boolean;

  constructor(scene: Phaser.Scene, x: number, y: number, stats: ShipStats) {
    this.scene = scene;
    this.stats = stats;
    this.spritePrefix = stats.spritePrefix;
    this.renderScaleMultiplier = getShipRenderScale(stats.size);

    this.crew = stats.maxCrew;
    this.energy = stats.maxEnergy;
    this.facing = -Math.PI / 2;
    this.x = x;
    this.y = y;
    this.vx = 0;
    this.vy = 0;
    this.dead = false;

    const defaultTexture = `${this.spritePrefix}-big-000`;
    this.sprite = scene.add.image(x, y, defaultTexture);
    for (let i = 0; i < 8; i++) {
      const ghost = scene.add.image(x, y, defaultTexture);
      ghost.setVisible(false);
      this.ghostSprites.push(ghost);
    }
  }

  applySnapshot(snapshot: BattleShipSnapshot) {
    this.x = snapshot.x;
    this.y = snapshot.y;
    this.vx = snapshot.vx;
    this.vy = snapshot.vy;
    this.crew = snapshot.crew;
    this.energy = snapshot.energy;
    this.facing = snapshot.facing;
    this.dead = snapshot.dead;
    this.spritePrefix = snapshot.texturePrefix;

    if (!snapshot.dead && snapshot.thrusting) {
      this.spawnIonParticle();
    }

    this.updateIonTrail();
  }

  setTint(color: number) {
    this.sprite.setTint(color);
    for (const ghost of this.ghostSprites) {
      ghost.setTint(color);
    }
  }

  renderUpdate(scale: number = 1) {
    if (this.dead) {
      this.sprite.setVisible(false);
      for (const ghost of this.ghostSprites) {
        ghost.setVisible(false);
      }
      return;
    }

    const frameIndex = this.facingToFrame();
    const texture = `${this.spritePrefix}-big-${String(frameIndex).padStart(3, '0')}`;
    const x = this.x;
    const y = this.y;

    this.sprite.setPosition(x, y);
    this.sprite.setTexture(texture);
    this.sprite.setScale(scale * this.renderScaleMultiplier);
    const margin = Math.max(this.sprite.displayWidth, this.sprite.displayHeight) * 0.5;

    const xOffsets = [0];
    const yOffsets = [0];

    if (x < margin) {
      xOffsets.push(BATTLE_WIDTH);
    }
    if (x > BATTLE_WIDTH - margin) {
      xOffsets.push(-BATTLE_WIDTH);
    }
    if (y < margin) {
      yOffsets.push(BATTLE_HEIGHT);
    }
    if (y > BATTLE_HEIGHT - margin) {
      yOffsets.push(-BATTLE_HEIGHT);
    }

    let ghostIndex = 0;
    for (const xOffset of xOffsets) {
      for (const yOffset of yOffsets) {
        if (xOffset === 0 && yOffset === 0) {
          continue;
        }

        const ghost = this.ghostSprites[ghostIndex++];
        ghost.setTexture(texture);
        ghost.setPosition(x + xOffset, y + yOffset);
        ghost.setScale(scale * this.renderScaleMultiplier);
        ghost.setVisible(true);
      }
    }

    for (; ghostIndex < this.ghostSprites.length; ghostIndex++) {
      this.ghostSprites[ghostIndex].setVisible(false);
    }
  }

  getCurrentTextureKey(): string {
    const frameIndex = this.facingToFrame();
    return `${this.spritePrefix}-big-${String(frameIndex).padStart(3, '0')}`;
  }

  getSpeed(): number {
    return Math.sqrt((this.vx * this.vx) + (this.vy * this.vy));
  }

  containsPoint(x: number, y: number): boolean {
    if (this.dead) {
      return false;
    }
    const radius = Math.max(this.sprite.displayWidth, this.sprite.displayHeight) * 0.5;
    return Phaser.Math.Distance.Between(this.x, this.y, x, y) <= radius;
  }

  destroy() {
    for (const particle of this.ionTrail) {
      particle.sprite.destroy();
    }
    this.ionTrail = [];

    this.sprite.destroy();
    for (const ghost of this.ghostSprites) {
      ghost.destroy();
    }
  }

  private spawnIonParticle() {
    const angle = this.facing + Math.PI;
    const dist = 72;
    const x = this.x + Math.cos(angle) * dist;
    const y = this.y + Math.sin(angle) * dist;

    const sprite = this.scene.add.image(x, y, 'ion-particle');
    sprite.setTint(ION_COLORS[0]);
    sprite.setDepth(-1);

    this.ionTrail.push({ sprite, colorIndex: 0 });
  }

  private updateIonTrail() {
    for (let i = this.ionTrail.length - 1; i >= 0; i--) {
      const particle = this.ionTrail[i];
      particle.colorIndex++;

      if (particle.colorIndex >= ION_COLORS.length) {
        particle.sprite.destroy();
        this.ionTrail.splice(i, 1);
      } else {
        particle.sprite.setTint(ION_COLORS[particle.colorIndex]);
      }
    }
  }

  private facingToFrame(): number {
    let angle = this.facing + Math.PI / 2;
    angle = ((angle % (2 * Math.PI)) + (2 * Math.PI)) % (2 * Math.PI);
    return Math.round(angle / (2 * Math.PI / NUM_FACINGS)) % NUM_FACINGS;
  }
}
