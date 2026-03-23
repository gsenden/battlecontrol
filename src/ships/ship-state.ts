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

export interface VelocityVector {
  x: number;
  y: number;
}

export interface PhysicsCommand {
  type: 'addVelocity' | 'setVelocity';
  dvx?: number;
  dvy?: number;
  vx?: number;
  vy?: number;
}

const TRAVEL_ALIGNMENT_EPSILON = 0.0001;
const GRAVITY_WELL_SPEED_MULTIPLIER = 1.75;

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
  update(input: ShipInput, currentVelocity: VelocityVector, allowBeyondMaxSpeed: boolean = false): PhysicsCommand[] {
    const commands: PhysicsCommand[] = [];
    const currentSpeed = Math.hypot(currentVelocity.x, currentVelocity.y);

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
      const nextVelocity = this.getThrustVelocity(currentVelocity, dvx, dvy, currentSpeed, allowBeyondMaxSpeed);
      commands.push({ type: 'setVelocity', vx: nextVelocity.x, vy: nextVelocity.y });
      this.thrustCounter = this.stats.thrustWait;
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

  private getThrustVelocity(
    currentVelocity: VelocityVector,
    dvx: number,
    dvy: number,
    currentSpeed: number,
    allowBeyondMaxSpeed: boolean,
  ): VelocityVector {
    const normalMaxSpeed = this.stats.maxSpeed;
    const gravityWellMaxSpeed = normalMaxSpeed * GRAVITY_WELL_SPEED_MULTIPLIER;
    const travelAligned = currentSpeed <= TRAVEL_ALIGNMENT_EPSILON || this.isTravelAligned(currentVelocity);

    if (!allowBeyondMaxSpeed && travelAligned && currentSpeed > normalMaxSpeed) {
      return currentVelocity;
    }

    const desiredVelocity = {
      x: currentVelocity.x + dvx,
      y: currentVelocity.y + dvy,
    };
    const desiredSpeed = Math.hypot(desiredVelocity.x, desiredVelocity.y);

    if (desiredSpeed <= normalMaxSpeed) {
      return desiredVelocity;
    }

    if (!travelAligned && currentSpeed >= normalMaxSpeed) {
      const travelAngle = Math.atan2(currentVelocity.y, currentVelocity.x);
      const rotatedVelocity = {
        x: currentVelocity.x + (dvx * 0.5) - (Math.cos(travelAngle) * this.stats.thrustIncrement),
        y: currentVelocity.y + (dvy * 0.5) - (Math.sin(travelAngle) * this.stats.thrustIncrement),
      };
      const rotatedSpeed = Math.hypot(rotatedVelocity.x, rotatedVelocity.y);

      if (rotatedSpeed <= gravityWellMaxSpeed || rotatedSpeed < currentSpeed) {
        return rotatedVelocity;
      }
    }

    if (((allowBeyondMaxSpeed && desiredSpeed <= gravityWellMaxSpeed) || desiredSpeed < currentSpeed)) {
      return desiredVelocity;
    }

    if (travelAligned) {
      const limitedSpeed = Math.min(
        allowBeyondMaxSpeed ? gravityWellMaxSpeed : normalMaxSpeed,
        desiredSpeed,
      );
      return {
        x: Math.cos(this.facing) * limitedSpeed,
        y: Math.sin(this.facing) * limitedSpeed,
      };
    }

    return currentVelocity;
  }

  private isTravelAligned(currentVelocity: VelocityVector): boolean {
    const travelAngle = Math.atan2(currentVelocity.y, currentVelocity.x);
    const facingDelta = Math.atan2(
      Math.sin(this.facing - travelAngle),
      Math.cos(this.facing - travelAngle),
    );
    return Math.abs(facingDelta) <= TRAVEL_ALIGNMENT_EPSILON;
  }
}
