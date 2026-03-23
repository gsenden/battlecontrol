// Ship game state - manages cooldowns, energy, and translates input into physics actions
// The scene/entity layer applies those commands to the active physics engine.

import type { ShipStats } from './ship-stats.js';

export interface ShipInput {
  left: boolean;
  right: boolean;
  thrust: boolean;
  weapon: boolean;
  special: boolean;
}

export interface PhysicsCommand {
  type: 'addVelocity' | 'capSpeed';
  dvx?: number;
  dvy?: number;
  maxSpeed?: number;
}

export class ShipState {
  readonly stats: ShipStats;
  readonly bodyId: number;

  // Game state
  crew: number;
  energy: number;
  facing: number; // Current angle in radians

  // Cooldown counters (count down to 0)
  turnCounter: number = 0;
  thrustCounter: number = 0;
  weaponCounter: number = 0;
  specialCounter: number = 0;
  energyCounter: number = 0;

  // Status
  statusFlags: number = 0;

  constructor(stats: ShipStats, bodyId: number, startAngle: number = -Math.PI / 2) {
    this.stats = stats;
    this.bodyId = bodyId;
    this.crew = stats.maxCrew;
    this.energy = stats.maxEnergy;
    this.facing = startAngle;
  }

  // Process one physics frame of input, returns commands for the physics layer
  update(input: ShipInput, currentSpeed: number): PhysicsCommand[] {
    const commands: PhysicsCommand[] = [];

    // Energy regeneration
    if (this.energyCounter > 0) {
      this.energyCounter--;
    } else if (this.energy < this.stats.maxEnergy) {
      this.energy = Math.min(this.energy + this.stats.energyRegeneration, this.stats.maxEnergy);
      this.energyCounter = this.stats.energyWait;
    }

    // Turning
    if (this.turnCounter > 0) {
      this.turnCounter--;
    } else if (input.left || input.right) {
      if (input.left) {
        this.facing -= this.stats.turnRate;
      } else {
        this.facing += this.stats.turnRate;
      }
      this.turnCounter = this.stats.turnWait;
    }

    // Thrust — directly adds velocity increment like SC2
    if (this.thrustCounter > 0) {
      this.thrustCounter--;
    } else if (input.thrust) {
      const dvx = Math.cos(this.facing) * this.stats.thrustIncrement;
      const dvy = Math.sin(this.facing) * this.stats.thrustIncrement;
      commands.push({ type: 'addVelocity', dvx, dvy });
      this.thrustCounter = this.stats.thrustWait;
    }

    // Speed cap
    if (currentSpeed > this.stats.maxSpeed) {
      commands.push({ type: 'capSpeed', maxSpeed: this.stats.maxSpeed });
    }

    // Weapon
    if (this.weaponCounter > 0) {
      this.weaponCounter--;
    } else if (input.weapon && this.energy >= this.stats.weaponEnergyCost) {
      this.energy -= this.stats.weaponEnergyCost;
      this.weaponCounter = this.stats.weaponWait;
      // TODO: spawn projectile
    }

    // Special
    if (this.specialCounter > 0) {
      this.specialCounter--;
    } else if (input.special && this.energy >= this.stats.specialEnergyCost) {
      this.energy -= this.stats.specialEnergyCost;
      this.specialCounter = this.stats.specialWait;
      // TODO: activate special ability
    }

    return commands;
  }

  takeDamage(amount: number): boolean {
    this.crew = Math.max(0, this.crew - amount);
    return this.crew <= 0;
  }
}
