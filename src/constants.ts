// SC2 game logic constants
// These define ship behavior rules, NOT physics math (matter-rs handles that)

// Ship facings: SC2 uses 16 discrete rotations
export const NUM_FACINGS = 16;
export const FACING_ANGLE_STEP = (2 * Math.PI) / NUM_FACINGS; // ~22.5 degrees

export function facingToRadians(facing: number): number {
  // SC2 angle 0 = "up" (north), increases clockwise
  // Phaser/Matter: 0 = right, increases clockwise
  // SC2 facing 0 → -PI/2 radians (up)
  return ((facing * FACING_ANGLE_STEP) - Math.PI / 2);
}

export function normalizeFacing(f: number): number {
  return ((f % NUM_FACINGS) + NUM_FACINGS) % NUM_FACINGS;
}

// Status flags (game logic, not physics)
export const LEFT = 1 << 0;
export const RIGHT = 1 << 1;
export const THRUST = 1 << 2;
export const WEAPON = 1 << 3;
export const SPECIAL = 1 << 4;
export const SHIP_AT_MAX_SPEED = 1 << 7;
export const SHIP_BEYOND_MAX_SPEED = 1 << 6;
export const SHIP_IN_GRAVITY_WELL = 1 << 8;

// Battle space (world units for matter-rs)
// We use a 800x600 world for the web version, scaled from SC2's proportions
export const BATTLE_WIDTH = 800;
export const BATTLE_HEIGHT = 600;

// Physics frame rate (SC2 runs at 24fps)
export const PHYSICS_FPS = 24;
export const PHYSICS_DELTA = 1000 / PHYSICS_FPS;
