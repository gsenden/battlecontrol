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

// Human Cruiser - balanced ship, good for testing
// Original SC2 values from src/uqm/ships/human/human.c
// Captain names from content/base/ships/human/cruiser.txt
export const HUMAN_CRUISER: ShipStats = {
  raceName: 'Earthling',
  shipClass: 'Cruiser',
  captainNames: [
    'Decker', 'Trent', 'Adama', 'Spiff', 'Graeme',
    'Kirk', 'Pike', 'Halleck', 'Tuf', 'Pirx',
    'Wu', 'VanRijn', 'Ender', 'Buck', 'Solo', 'Belt',
  ],
  cost: 11,

  mass: 6,
  thrustIncrement: 0.6,    // Slightly boosted for the current world scale and gravity tuning
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

  spritePrefix: 'human-cruiser',
  color: 0x4488ff,
  size: 16,
};
