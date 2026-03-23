import Phaser from 'phaser';
import { ShipState } from '../ships/ship-state.js';
import type { ShipStats } from '../ships/ship-stats.js';
import type { ShipInput } from '../ships/ship-state.js';
import { NUM_FACINGS } from '../constants.js';

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
  private scene: Phaser.Scene;
  private ionTrail: IonParticle[] = [];

  constructor(scene: Phaser.Scene, x: number, y: number, stats: ShipStats) {
    this.scene = scene;
    this.state = new ShipState(stats, 0);

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
    this.sprite = scene.add.image(x, y, 'human-cruiser-0');
  }

  /** Called at physics rate (24fps) — processes game logic */
  physicsUpdate(input: ShipInput) {
    const commands = this.state.update(input, this.getSpeed());
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
        case 'capSpeed': {
          const vel = this.body.velocity;
          const currentSpeed = Math.sqrt(vel.x * vel.x + vel.y * vel.y);
          if (currentSpeed > cmd.maxSpeed!) {
            const scale = cmd.maxSpeed! / currentSpeed;
            this.scene.matter.body.setVelocity(this.body, {
              x: vel.x * scale,
              y: vel.y * scale,
            });
          }
          break;
        }
      }
    }

    if (this.getSpeed() > this.state.stats.maxSpeed) {
      const vel = this.body.velocity;
      const currentSpeed = Math.sqrt(vel.x * vel.x + vel.y * vel.y);
      const scale = this.state.stats.maxSpeed / currentSpeed;
      this.scene.matter.body.setVelocity(this.body, {
        x: vel.x * scale,
        y: vel.y * scale,
      });
    }

    // Spawn ion particle only when a real thrust impulse was applied
    if (didThrust) {
      this.spawnIonParticle();
    }

    // Age existing ion particles
    this.updateIonTrail();
  }

  /** Called every render frame — updates visuals only */
  renderUpdate() {
    this.sprite.setPosition(this.body.position.x, this.body.position.y);
    const frameIndex = this.facingToFrame();
    this.sprite.setTexture(`human-cruiser-${frameIndex}`);
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

  private facingToFrame(): number {
    let angle = this.state.facing + Math.PI / 2;
    angle = ((angle % (2 * Math.PI)) + 2 * Math.PI) % (2 * Math.PI);
    return Math.round(angle / (2 * Math.PI / NUM_FACINGS)) % NUM_FACINGS;
  }

  get x() { return this.body.position.x; }
  get y() { return this.body.position.y; }
}
