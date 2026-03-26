// Ship stat definitions ported from SC2 source
// These control game logic (cooldowns, energy, costs) while matter-rs handles physics

export interface ShipStats {
  raceName: string;       // "Earthling" — shown in HUD
  shipClass: string;      // "Cruiser"
  captainNames: string[]; // Pool of captain names from race_strings
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
  spritePrefix: string;   // Texture key prefix (e.g. 'human-cruiser')
  color: number;          // Ship color (hex)
  size: number;           // Ship radius for polygon body
}
