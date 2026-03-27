import Phaser from 'phaser';
import { BATTLE_HEIGHT, BATTLE_WIDTH } from '../constants.js';
import { ShipState } from '../ships/ship-state.js';
import type { ShipStats } from '../ships/ship-stats.js';
import type { ShipInput } from '../ships/ship-state.js';
import { NUM_FACINGS } from '../constants.js';
import { wrapPoint } from '../utils/wrap.js';
import { getShipRenderScale } from '../ui/hud-state.svelte.js';

// SC2 ion trail color cycle: 12 steps from bright orange → dark red → gone
// Converted from MAKE_RGB15 (5-bit per channel) to 8-bit hex
const ION_COLORS = [
  0xffaa00,  // 1F,15,00
  0xff8800,  // 1F,11,00
  0xff7000,  // 1F,0E,00
  0xff5000,  // 1F,0A,00
  0xff3800,  // 1F,07,00
  0xff1800,  // 1F,03,00
  0xff0000,  // 1F,00,00
  0xd80000,  // 1B,00,00
  0xb80000,  // 17,00,00
  0x980000,  // 13,00,00
  0x780000,  // 0F,00,00
  0x580000,  // 0B,00,00
];

interface IonParticle {
  sprite: Phaser.GameObjects.Image;
  colorIndex: number;
}

export class Ship {
  readonly state: ShipState;
  readonly body: MatterJS.BodyType;
  private sprite: Phaser.GameObjects.Image;
  private ghostSprites: Phaser.GameObjects.Image[] = [];
  private scene: Phaser.Scene;
  private ionTrail: IonParticle[] = [];
  private readonly spritePrefix: string;
  private readonly renderScaleMultiplier: number;

  constructor(scene: Phaser.Scene, x: number, y: number, stats: ShipStats) {
    this.scene = scene;
    this.state = new ShipState(stats, 0);
    this.spritePrefix = stats.spritePrefix;
    this.renderScaleMultiplier = stats.renderScale ?? getShipRenderScale(stats.size);

    // Create Matter.js circle body (simple hitbox for now)
    this.body = scene.matter.add.circle(x, y, stats.size, {
      mass: stats.mass,
      frictionAir: 0,
      friction: 0,
      frictionStatic: 0,
      restitution: 0.8,
    });
    scene.matter.body.setInertia(this.body, Infinity);

    // Ship sprite — frame 0 = facing up
    const defaultTexture = `${this.spritePrefix}-big-000`;
    this.sprite = scene.add.image(x, y, defaultTexture);
    for (let i = 0; i < 8; i++) {
      const ghost = scene.add.image(x, y, defaultTexture);
      ghost.setVisible(false);
      this.ghostSprites.push(ghost);
    }
  }

  /** Called at physics rate (24fps) — processes game logic */
  physicsUpdate(input: ShipInput, allowBeyondMaxSpeed: boolean = false) {
    const commands = this.state.update(input, this.body.velocity, allowBeyondMaxSpeed);
    let didThrust = false;

    for (const cmd of commands) {
      switch (cmd.type) {
        case 'addVelocity': {
          didThrust = true;
          // Directly add velocity increment — no force, no mass dependency
          const vel = this.body.velocity;
          this.scene.matter.body.setVelocity(this.body, {
            x: vel.x + cmd.dvx!,
            y: vel.y + cmd.dvy!,
          });
          break;
        }
        case 'setVelocity': {
          didThrust = true;
          this.scene.matter.body.setVelocity(this.body, {
            x: cmd.vx!,
            y: cmd.vy!,
          });
          break;
        }
      }
    }

    // Spawn ion particle only when a real thrust impulse was applied
    if (didThrust) {
      this.spawnIonParticle();
    }

    // Age existing ion particles
    this.updateIonTrail();
  }

  addVelocity(dvx: number, dvy: number) {
    const vel = this.body.velocity;
    this.scene.matter.body.setVelocity(this.body, {
      x: vel.x + dvx,
      y: vel.y + dvy,
    });
  }

  setTint(color: number) {
    this.sprite.setTint(color);
    for (const ghost of this.ghostSprites) {
      ghost.setTint(color);
    }
  }

  /** Called every render frame — updates visuals only */
  renderUpdate(scale: number = 1) {
    const frameIndex = this.facingToFrame();
    const texture = `${this.spritePrefix}-big-${String(frameIndex).padStart(3, '0')}`;
    const x = this.body.position.x;
    const y = this.body.position.y;

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

  wrapPosition() {
    const wrapped = wrapPoint(this.body.position.x, this.body.position.y, BATTLE_WIDTH, BATTLE_HEIGHT);
    if (wrapped.x === this.body.position.x && wrapped.y === this.body.position.y) {
      return;
    }

    this.scene.matter.body.setPosition(this.body, wrapped);
  }

  private spawnIonParticle() {
    const angle = this.state.facing + Math.PI; // opposite of facing
    const dist = 72;
    const x = this.body.position.x + Math.cos(angle) * dist;
    const y = this.body.position.y + Math.sin(angle) * dist;

    const sprite = this.scene.add.image(x, y, 'ion-particle');
    sprite.setTint(ION_COLORS[0]);
    sprite.setDepth(-1); // behind the ship

    this.ionTrail.push({ sprite, colorIndex: 0 });
  }

  private updateIonTrail() {
    for (let i = this.ionTrail.length - 1; i >= 0; i--) {
      const particle = this.ionTrail[i];
      particle.colorIndex++;

      if (particle.colorIndex >= ION_COLORS.length) {
        // Particle expired
        particle.sprite.destroy();
        this.ionTrail.splice(i, 1);
      } else {
        particle.sprite.setTint(ION_COLORS[particle.colorIndex]);
      }
    }
  }

  getSpeed(): number {
    const vel = this.body.velocity;
    return Math.sqrt(vel.x * vel.x + vel.y * vel.y);
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

    this.scene.matter.world.remove(this.body);
  }

  private facingToFrame(): number {
    let angle = this.state.facing + Math.PI / 2;
    angle = ((angle % (2 * Math.PI)) + 2 * Math.PI) % (2 * Math.PI);
    return Math.round(angle / (2 * Math.PI / NUM_FACINGS)) % NUM_FACINGS;
  }

  get x() { return this.body.position.x; }
  get y() { return this.body.position.y; }
}
