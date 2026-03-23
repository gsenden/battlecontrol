// Ship stat definitions ported from SC2 source
// These control game logic (cooldowns, energy, costs) while matter-rs handles physics

export interface ShipStats {
  name: string;
  cost: number;

  // Physics properties
  mass: number;             // Body mass
  thrustIncrement: number;  // Velocity added per thrust (direct, like SC2)
  maxSpeed: number;         // Speed cap
  turnRate: number;         // Radians per turn step

  // Cooldown timers (in physics frames at 24fps)
  turnWait: number;       // Frames between turns
  thrustWait: number;     // Frames between thrust applications
  weaponWait: number;     // Frames between weapon fires
  specialWait: number;    // Frames between special uses

  // Energy system
  maxEnergy: number;
  energyRegeneration: number;  // Energy per regen tick
  energyWait: number;          // Frames between regen ticks
  weaponEnergyCost: number;
  specialEnergyCost: number;

  // Health
  maxCrew: number;        // Crew = health points

  // Visual
  color: number;          // Ship color (hex)
  size: number;           // Ship radius for polygon body
}

// Human Cruiser - balanced ship, good for testing
// Original SC2 values from src/uqm/ships/human/human.c
export const HUMAN_CRUISER: ShipStats = {
  name: 'Earthling Cruiser',
  cost: 11,

  mass: 6,
  thrustIncrement: 0.5,    // Velocity added per thrust step (SC2: 3 display units → 96 internal)
  maxSpeed: 4.0,           // Max velocity magnitude (SC2: 24 display units → 768 internal)
  turnRate: Math.PI / 8,   // 22.5 degrees per step (1/16 of circle)

  turnWait: 1,
  thrustWait: 4,
  weaponWait: 10,
  specialWait: 9,

  maxEnergy: 18,
  energyRegeneration: 1,
  energyWait: 8,
  weaponEnergyCost: 9,
  specialEnergyCost: 4,

  maxCrew: 18,

  color: 0x4488ff,
  size: 16,
};
