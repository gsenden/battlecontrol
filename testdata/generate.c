// SC2 Reference Data Generator
// Runs SC2 physics functions with known inputs and dumps output as JSON
// Used by BattleControl tests to verify game logic matches the original

#include "sc2_types.h"
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// From sc2_velocity.c
extern STATUS_FLAGS inertial_thrust(VELOCITY_DESC *VelocityPtr, COUNT ShipFacing,
    STATUS_FLAGS cur_status_flags, COUNT max_thrust, COUNT thrust_increment);

// --- Human Cruiser stats (from sc2/src/uqm/ships/human/human.c) ---
#define HUMAN_MAX_THRUST        24
#define HUMAN_THRUST_INCREMENT  3
#define HUMAN_TURN_WAIT         1
#define HUMAN_THRUST_WAIT       4
#define HUMAN_WEAPON_WAIT       10
#define HUMAN_SPECIAL_WAIT      9
#define HUMAN_ENERGY_WAIT       8
#define HUMAN_MAX_CREW          18
#define HUMAN_MAX_ENERGY        18
#define HUMAN_ENERGY_REGEN      1
#define HUMAN_WEAPON_COST       9
#define HUMAN_SPECIAL_COST      4
#define HUMAN_SHIP_MASS         6
#define HUMAN_OFFSET            RES_SCALE(42)
#define MIN_MISSILE_SPEED       DISPLAY_TO_WORLD(RES_SCALE(10))
#define MAX_MISSILE_SPEED       DISPLAY_TO_WORLD(RES_SCALE(20))
#define MISSILE_SPEED           (HUMAN_MAX_THRUST >= MIN_MISSILE_SPEED ? HUMAN_MAX_THRUST : MIN_MISSILE_SPEED)
#define THRUST_SCALE            DISPLAY_TO_WORLD(RES_SCALE(1))
#define MISSILE_LIFE            60
#define HUMAN_TRACK_WAIT        3

// --- Androsynth Guardian stats (from sc2/src/uqm/ships/androsyn/androsyn.c) ---
#define ANDRO_OFFSET            RES_SCALE(14)
#define ANDRO_MISSILE_SPEED     DISPLAY_TO_WORLD(RES_SCALE(8))
#define ANDRO_MISSILE_LIFE      200
#define ANDRO_TRACK_WAIT        2

typedef struct {
    VELOCITY_DESC velocity;
    COUNT facing;
    int life;
    int turn_wait;
    int pos_x, pos_y;
} SimProjectile;

// --- Ship simulation state ---
typedef struct {
    VELOCITY_DESC velocity;
    COUNT facing;          // 0-15
    STATUS_FLAGS status;
    int turn_wait;
    int thrust_wait;
    int weapon_counter;
    int special_counter;
    int energy_counter;
    int energy;
    int crew;
    int pos_x, pos_y;     // World coordinates
} SimShip;

static void sim_ship_init(SimShip *s, int x, int y, COUNT facing) {
    memset(s, 0, sizeof(*s));
    s->pos_x = x;
    s->pos_y = y;
    s->facing = facing;
    s->crew = HUMAN_MAX_CREW;
    s->energy = HUMAN_MAX_ENERGY;
}

// Process one frame of input for a ship (mirrors ship_preprocess in ship.c)
static void sim_ship_frame(SimShip *s, int input_left, int input_right,
    int input_thrust, int input_weapon, int input_special)
{
    // Energy regeneration
    if (s->energy_counter > 0) {
        s->energy_counter--;
    } else if (s->energy < HUMAN_MAX_ENERGY) {
        s->energy += HUMAN_ENERGY_REGEN;
        if (s->energy > HUMAN_MAX_ENERGY)
            s->energy = HUMAN_MAX_ENERGY;
        s->energy_counter = HUMAN_ENERGY_WAIT;
    }

    // Turning (from ship_preprocess)
    if (s->turn_wait > 0) {
        s->turn_wait--;
    } else if (input_left || input_right) {
        if (input_left)
            s->facing = NORMALIZE_FACING(s->facing - 1);
        else
            s->facing = NORMALIZE_FACING(s->facing + 1);
        s->turn_wait = HUMAN_TURN_WAIT;
    }

    // Thrust (from ship_preprocess)
    if (s->thrust_wait > 0) {
        s->thrust_wait--;
    } else if (input_thrust) {
        STATUS_FLAGS thrust_status = inertial_thrust(
            &s->velocity, s->facing, s->status,
            HUMAN_MAX_THRUST, HUMAN_THRUST_INCREMENT);
        s->status &= ~(SHIP_AT_MAX_SPEED | SHIP_BEYOND_MAX_SPEED | SHIP_IN_GRAVITY_WELL);
        s->status |= thrust_status;
        s->thrust_wait = HUMAN_THRUST_WAIT;
    }

    // Weapon
    if (s->weapon_counter > 0) {
        s->weapon_counter--;
    } else if (input_weapon && s->energy >= HUMAN_WEAPON_COST) {
        s->energy -= HUMAN_WEAPON_COST;
        s->weapon_counter = HUMAN_WEAPON_WAIT;
    }

    // Special
    if (s->special_counter > 0) {
        s->special_counter--;
    } else if (input_special && s->energy >= HUMAN_SPECIAL_COST) {
        s->energy -= HUMAN_SPECIAL_COST;
        s->special_counter = HUMAN_SPECIAL_WAIT;
    }

    // Apply velocity to position (from PreProcess in process.c)
    {
        SIZE dx, dy;
        GetNextVelocityComponents(&s->velocity, &dx, &dy, 1);
        s->pos_x += dx;
        s->pos_y += dy;
    }
}

// Apply gravity from a planet at (px, py) to ship s
// Exact port of CalculateGravity from sc2/src/uqm/gravity.c
#define GRAVITY_THRESHOLD (COUNT)RES_SCALE(255)

static void sim_apply_gravity(SimShip *s, int planet_x, int planet_y) {
    // In SC2 gravity.c: dx = planet.x - ship.x (delta pointing toward planet)
    SIZE dx = planet_x - s->pos_x;
    SIZE dy = planet_y - s->pos_y;

    // Wrap delta (shortest path across toroidal space)
    // Simplified: no wrapping in our test scenarios (positions stay close)
    COUNT abs_dx = dx >= 0 ? dx : -dx;
    COUNT abs_dy = dy >= 0 ? dy : -dy;
    abs_dx = WORLD_TO_DISPLAY(abs_dx);
    abs_dy = WORLD_TO_DISPLAY(abs_dy);

    if (abs_dx <= GRAVITY_THRESHOLD && abs_dy <= GRAVITY_THRESHOLD) {
        DWORD dist_squared = (DWORD)(abs_dx * abs_dx) + (DWORD)(abs_dy * abs_dy);
        if (dist_squared <= (DWORD)(GRAVITY_THRESHOLD * GRAVITY_THRESHOLD)) {
            COUNT angle = ARCTAN(dx, dy);
            DeltaVelocityComponents(&s->velocity,
                COSINE(angle, WORLD_TO_VELOCITY(RES_SCALE(1))),
                SINE(angle, WORLD_TO_VELOCITY(RES_SCALE(1))));
            s->status &= ~SHIP_AT_MAX_SPEED;
            s->status |= SHIP_IN_GRAVITY_WELL;
        }
    }
}

static void sim_human_nuke_init(SimProjectile *p, int ship_x, int ship_y, COUNT facing) {
    SIZE dx, dy;
    COUNT angle;
    memset(p, 0, sizeof(*p));
    p->facing = facing;
    p->life = MISSILE_LIFE;
    p->turn_wait = HUMAN_TRACK_WAIT;
    angle = FACING_TO_ANGLE(facing);
    p->pos_x = ship_x + COSINE(angle, DISPLAY_TO_WORLD(HUMAN_OFFSET));
    p->pos_y = ship_y + SINE(angle, DISPLAY_TO_WORLD(HUMAN_OFFSET));
    dx = COSINE(angle, WORLD_TO_VELOCITY(MISSILE_SPEED));
    dy = SINE(angle, WORLD_TO_VELOCITY(MISSILE_SPEED));
    SetVelocityComponents(&p->velocity, dx, dy);
    GetCurrentVelocityComponents(&p->velocity, &dx, &dy);
    p->pos_x -= VELOCITY_TO_WORLD(dx);
    p->pos_y -= VELOCITY_TO_WORLD(dy);
}

static void sim_track_target(COUNT *facing, int tracker_x, int tracker_y, int target_x, int target_y) {
    SIZE delta_x = target_x - tracker_x;
    SIZE delta_y = target_y - tracker_y;
    SIZE delta_facing = NORMALIZE_FACING(ANGLE_TO_FACING(ARCTAN(delta_x, delta_y)) - *facing);

    if (delta_facing > 0) {
        if (delta_facing < ANGLE_TO_FACING(HALF_CIRCLE))
            *facing = NORMALIZE_FACING(*facing + 1);
        else
            *facing = NORMALIZE_FACING(*facing - 1);
    }
}

static void sim_human_nuke_frame(SimProjectile *p) {
    SIZE dx, dy;
    SDWORD speed = MISSILE_SPEED + ((MISSILE_LIFE - p->life) * THRUST_SCALE);
    COUNT angle = FACING_TO_ANGLE(p->facing);
    if (speed > MAX_MISSILE_SPEED)
        speed = MAX_MISSILE_SPEED;

    SetVelocityComponents(
        &p->velocity,
        COSINE(angle, WORLD_TO_VELOCITY(speed)),
        SINE(angle, WORLD_TO_VELOCITY(speed))
    );
    GetNextVelocityComponents(&p->velocity, &dx, &dy, 1);
    p->pos_x += dx;
    p->pos_y += dy;
    p->life--;
}

static void sim_human_nuke_homing_frame(SimProjectile *p, int target_x, int target_y) {
    if (p->turn_wait > 0) {
        p->turn_wait--;
    } else {
        sim_track_target(&p->facing, p->pos_x, p->pos_y, target_x, target_y);
        p->turn_wait = HUMAN_TRACK_WAIT;
    }

    sim_human_nuke_frame(p);
}

static uint32 andro_rng_state = 0x00C0FFEEu;

static COUNT sim_andro_random(void) {
    andro_rng_state = (andro_rng_state * 1103515245u) + 12345u;
    return (COUNT)((andro_rng_state >> 16) & 0x7FFFu);
}

static void sim_androsynth_bubble_init(SimProjectile *p, int ship_x, int ship_y, COUNT facing) {
    SIZE dx, dy;
    COUNT angle;
    memset(p, 0, sizeof(*p));
    p->facing = facing;
    p->life = ANDRO_MISSILE_LIFE;
    p->turn_wait = 0;
    angle = FACING_TO_ANGLE(facing);
    p->pos_x = ship_x + COSINE(angle, DISPLAY_TO_WORLD(ANDRO_OFFSET));
    p->pos_y = ship_y + SINE(angle, DISPLAY_TO_WORLD(ANDRO_OFFSET));
    SetVelocityVector(&p->velocity, ANDRO_MISSILE_SPEED, facing);
    GetCurrentVelocityComponents(&p->velocity, &dx, &dy);
    p->pos_x -= VELOCITY_TO_WORLD(dx);
    p->pos_y -= VELOCITY_TO_WORLD(dy);
}

static void sim_androsynth_bubble_frame(SimProjectile *p, int target_x, int target_y) {
    SIZE dx, dy;
    BYTE thrust_wait = HIBYTE(p->turn_wait);
    BYTE turn_wait = LOBYTE(p->turn_wait);

    if (thrust_wait > 0) {
        --thrust_wait;
    } else {
        thrust_wait = (BYTE)(sim_andro_random() & 3);
    }

    if (turn_wait > 0) {
        --turn_wait;
    } else {
        COUNT facing = NORMALIZE_FACING(ANGLE_TO_FACING(GetVelocityTravelAngle(&p->velocity)));
        SIZE delta_facing = NORMALIZE_FACING(ANGLE_TO_FACING(ARCTAN(target_x - p->pos_x, target_y - p->pos_y)) - facing);

        if (delta_facing <= ANGLE_TO_FACING(HALF_CIRCLE))
            facing = NORMALIZE_FACING(facing + (sim_andro_random() & (ANGLE_TO_FACING(HALF_CIRCLE) - 1)));
        else
            facing = NORMALIZE_FACING(facing - (sim_andro_random() & (ANGLE_TO_FACING(HALF_CIRCLE) - 1)));

        p->facing = facing;
        SetVelocityVector(&p->velocity, ANDRO_MISSILE_SPEED, facing);
        turn_wait = ANDRO_TRACK_WAIT;
    }

    p->turn_wait = MAKE_WORD(thrust_wait, turn_wait);
    GetNextVelocityComponents(&p->velocity, &dx, &dy, 1);
    p->pos_x += dx;
    p->pos_y += dy;
    p->life--;
}

// --- JSON output helpers ---

static FILE *out;

static void json_ship_state(SimShip *s, const char *indent) {
    SIZE vx, vy;
    GetCurrentVelocityComponents(&s->velocity, &vx, &vy);
    fprintf(out, "%s\"x\": %d, \"y\": %d,\n", indent, s->pos_x, s->pos_y);
    fprintf(out, "%s\"vx\": %d, \"vy\": %d,\n", indent, (int)vx, (int)vy);
    fprintf(out, "%s\"facing\": %d,\n", indent, (int)s->facing);
    fprintf(out, "%s\"crew\": %d, \"energy\": %d,\n", indent, s->crew, s->energy);
    fprintf(out, "%s\"statusFlags\": %d,\n", indent, (int)s->status);
    fprintf(out, "%s\"turnWait\": %d, \"thrustWait\": %d,\n", indent, s->turn_wait, s->thrust_wait);
    fprintf(out, "%s\"weaponCounter\": %d, \"specialCounter\": %d,\n", indent, s->weapon_counter, s->special_counter);
    fprintf(out, "%s\"energyCounter\": %d\n", indent, s->energy_counter);
}

static void json_projectile_state(SimProjectile *p, const char *indent) {
    SIZE vx, vy;
    GetCurrentVelocityComponents(&p->velocity, &vx, &vy);
    fprintf(out, "%s\"x\": %d, \"y\": %d,\n", indent, p->pos_x, p->pos_y);
    fprintf(out, "%s\"vx\": %d, \"vy\": %d,\n", indent, (int)vx, (int)vy);
    fprintf(out, "%s\"life\": %d\n", indent, p->life);
}

static void sim_apply_collision_cooldowns(SimShip *s) {
    if (s->turn_wait < 1)
        s->turn_wait += 1;
    if (s->thrust_wait < 3)
        s->thrust_wait += 3;
}

static SIZE sim_square_root(long value) {
    return (SIZE)sqrt((double)value);
}

static void sim_collide_ships(SimShip *ship0, SimShip *ship1) {
    SIZE speed;
    SIZE dx0, dy0, dx1, dy1, dx_rel, dy_rel;
    SIZE travel_angle0, travel_angle1, impact_angle0, impact_angle1;
    SIZE rel_travel_angle, directness;
    SIZE mass0, mass1;
    long scalar;

    dx_rel = ship0->pos_x - ship1->pos_x;
    dy_rel = ship0->pos_y - ship1->pos_y;
    impact_angle0 = ARCTAN(dx_rel, dy_rel);
    impact_angle1 = NORMALIZE_ANGLE(impact_angle0 + HALF_CIRCLE);

    GetCurrentVelocityComponents(&ship0->velocity, &dx0, &dy0);
    travel_angle0 = GetVelocityTravelAngle(&ship0->velocity);
    GetCurrentVelocityComponents(&ship1->velocity, &dx1, &dy1);
    travel_angle1 = GetVelocityTravelAngle(&ship1->velocity);
    dx_rel = dx0 - dx1;
    dy_rel = dy0 - dy1;
    rel_travel_angle = ARCTAN(dx_rel, dy_rel);
    speed = sim_square_root((long)dx_rel * dx_rel + (long)dy_rel * dy_rel);

    directness = NORMALIZE_ANGLE(rel_travel_angle - impact_angle0);
    if (directness <= QUADRANT || directness >= HALF_CIRCLE + QUADRANT) {
        directness = HALF_CIRCLE;
        impact_angle0 = travel_angle0 + HALF_CIRCLE;
        impact_angle1 = travel_angle1 + HALF_CIRCLE;
    }

    mass0 = HUMAN_SHIP_MASS;
    mass1 = HUMAN_SHIP_MASS;
    scalar = (long)SINE(directness, speed << 1) * (mass0 * mass1);

    sim_apply_collision_cooldowns(ship0);
    sim_apply_collision_cooldowns(ship1);

    speed = (SIZE)(scalar / ((long)mass0 * (mass0 + mass1)));
    DeltaVelocityComponents(&ship0->velocity, COSINE(impact_angle0, speed), SINE(impact_angle0, speed));

    GetCurrentVelocityComponents(&ship0->velocity, &dx0, &dy0);
    if (dx0 < 0) dx0 = -dx0;
    if (dy0 < 0) dy0 = -dy0;
    if (VELOCITY_TO_WORLD(dx0 + dy0) < SCALED_ONE) {
        SetVelocityComponents(
            &ship0->velocity,
            COSINE(impact_angle0, WORLD_TO_VELOCITY(SCALED_ONE) - 1),
            SINE(impact_angle0, WORLD_TO_VELOCITY(SCALED_ONE) - 1)
        );
    }

    speed = (SIZE)(scalar / ((long)mass1 * (mass0 + mass1)));
    DeltaVelocityComponents(&ship1->velocity, COSINE(impact_angle1, speed), SINE(impact_angle1, speed));

    GetCurrentVelocityComponents(&ship1->velocity, &dx1, &dy1);
    if (dx1 < 0) dx1 = -dx1;
    if (dy1 < 0) dy1 = -dy1;
    if (VELOCITY_TO_WORLD(dx1 + dy1) < SCALED_ONE) {
        SetVelocityComponents(
            &ship1->velocity,
            COSINE(impact_angle1, WORLD_TO_VELOCITY(SCALED_ONE) - 1),
            SINE(impact_angle1, WORLD_TO_VELOCITY(SCALED_ONE) - 1)
        );
    }
}

// --- Scenarios ---

// Scenario: pure thrust in one direction for N frames
static void scenario_collision_cooldowns(void) {
    SimShip ship;
    sim_ship_init(&ship, 5000, 5000, 0);

    fprintf(out, "  \"collision_cooldowns\": {\n");
    fprintf(out, "    \"description\": \"Ship collision applies the SC2 collision turn and thrust wait values\",\n");
    fprintf(out, "    \"ship\": \"human_cruiser\",\n");
    sim_apply_collision_cooldowns(&ship);
    fprintf(out, "    \"turnWait\": %d,\n", ship.turn_wait);
    fprintf(out, "    \"thrustWait\": %d\n", ship.thrust_wait);
    fprintf(out, "  }");
}

static void scenario_collision_existing_cooldowns(void) {
    SimShip ship;
    sim_ship_init(&ship, 5000, 5000, 0);
    ship.turn_wait = 2;
    ship.thrust_wait = 4;

    fprintf(out, "  \"collision_existing_cooldowns\": {\n");
    fprintf(out, "    \"description\": \"Ship collision does not reduce existing cooldowns when they are already higher\",\n");
    fprintf(out, "    \"ship\": \"human_cruiser\",\n");
    sim_apply_collision_cooldowns(&ship);
    fprintf(out, "    \"turnWait\": %d,\n", ship.turn_wait);
    fprintf(out, "    \"thrustWait\": %d\n", ship.thrust_wait);
    fprintf(out, "  }");
}

static void scenario_collision_head_on(void) {
    SimShip ship0;
    SimShip ship1;
    SIZE vx0_before, vy0_before, vx1_before, vy1_before;
    SIZE vx0_after, vy0_after, vx1_after, vy1_after;

    sim_ship_init(&ship0, 5000, 5000, 4);
    sim_ship_init(&ship1, 5010, 5000, 12);
    SetVelocityComponents(&ship0.velocity, 96, 0);
    SetVelocityComponents(&ship1.velocity, -96, 0);
    GetCurrentVelocityComponents(&ship0.velocity, &vx0_before, &vy0_before);
    GetCurrentVelocityComponents(&ship1.velocity, &vx1_before, &vy1_before);

    sim_collide_ships(&ship0, &ship1);

    GetCurrentVelocityComponents(&ship0.velocity, &vx0_after, &vy0_after);
    GetCurrentVelocityComponents(&ship1.velocity, &vx1_after, &vy1_after);

    fprintf(out, "  \"collision_head_on\": {\n");
    fprintf(out, "    \"description\": \"Two equal-mass human ships collide head-on\",\n");
    fprintf(out, "    \"before\": {\n");
    fprintf(out, "      \"ship0\": { \"vx\": %d, \"vy\": %d },\n", (int)vx0_before, (int)vy0_before);
    fprintf(out, "      \"ship1\": { \"vx\": %d, \"vy\": %d }\n", (int)vx1_before, (int)vy1_before);
    fprintf(out, "    },\n");
    fprintf(out, "    \"after\": {\n");
    fprintf(out, "      \"ship0\": { \"vx\": %d, \"vy\": %d, \"turnWait\": %d, \"thrustWait\": %d },\n",
        (int)vx0_after, (int)vy0_after, ship0.turn_wait, ship0.thrust_wait);
    fprintf(out, "      \"ship1\": { \"vx\": %d, \"vy\": %d, \"turnWait\": %d, \"thrustWait\": %d }\n",
        (int)vx1_after, (int)vy1_after, ship1.turn_wait, ship1.thrust_wait);
    fprintf(out, "    }\n");
    fprintf(out, "  }");
}

static void scenario_collision_moving_into_stationary(void) {
    SimShip ship0;
    SimShip ship1;
    SIZE vx0_after, vy0_after, vx1_after, vy1_after;

    sim_ship_init(&ship0, 5000, 5000, 4);
    sim_ship_init(&ship1, 5010, 5000, 12);
    SetVelocityComponents(&ship0.velocity, 96, 0);
    SetVelocityComponents(&ship1.velocity, 0, 0);

    sim_collide_ships(&ship0, &ship1);

    GetCurrentVelocityComponents(&ship0.velocity, &vx0_after, &vy0_after);
    GetCurrentVelocityComponents(&ship1.velocity, &vx1_after, &vy1_after);

    fprintf(out, "  \"collision_moving_into_stationary\": {\n");
    fprintf(out, "    \"description\": \"One human ship collides head-on with a stationary equal-mass ship\",\n");
    fprintf(out, "    \"after\": {\n");
    fprintf(out, "      \"ship0\": { \"vx\": %d, \"vy\": %d, \"turnWait\": %d, \"thrustWait\": %d },\n",
        (int)vx0_after, (int)vy0_after, ship0.turn_wait, ship0.thrust_wait);
    fprintf(out, "      \"ship1\": { \"vx\": %d, \"vy\": %d, \"turnWait\": %d, \"thrustWait\": %d }\n",
        (int)vx1_after, (int)vy1_after, ship1.turn_wait, ship1.thrust_wait);
    fprintf(out, "    }\n");
    fprintf(out, "  }");
}

static void scenario_collision_asymmetric_head_on(void) {
    SimShip ship0;
    SimShip ship1;
    SIZE vx0_after, vy0_after, vx1_after, vy1_after;

    sim_ship_init(&ship0, 5000, 5000, 4);
    sim_ship_init(&ship1, 5010, 5000, 12);
    SetVelocityComponents(&ship0.velocity, 192, 0);
    SetVelocityComponents(&ship1.velocity, -96, 0);

    sim_collide_ships(&ship0, &ship1);

    GetCurrentVelocityComponents(&ship0.velocity, &vx0_after, &vy0_after);
    GetCurrentVelocityComponents(&ship1.velocity, &vx1_after, &vy1_after);

    fprintf(out, "  \"collision_asymmetric_head_on\": {\n");
    fprintf(out, "    \"description\": \"Two equal-mass human ships collide head-on with different speeds\",\n");
    fprintf(out, "    \"after\": {\n");
    fprintf(out, "      \"ship0\": { \"vx\": %d, \"vy\": %d, \"turnWait\": %d, \"thrustWait\": %d },\n",
        (int)vx0_after, (int)vy0_after, ship0.turn_wait, ship0.thrust_wait);
    fprintf(out, "      \"ship1\": { \"vx\": %d, \"vy\": %d, \"turnWait\": %d, \"thrustWait\": %d }\n",
        (int)vx1_after, (int)vy1_after, ship1.turn_wait, ship1.thrust_wait);
    fprintf(out, "    }\n");
    fprintf(out, "  }");
}

// Scenario: pure thrust in one direction for N frames
static void scenario_thrust_straight(void) {
    SimShip ship;
    sim_ship_init(&ship, 5000, 5000, 0); // facing 0 = right

    fprintf(out, "  \"thrust_straight\": {\n");
    fprintf(out, "    \"description\": \"Human Cruiser facing right, thrust every frame for 60 frames\",\n");
    fprintf(out, "    \"ship\": \"human_cruiser\",\n");
    fprintf(out, "    \"initial_facing\": 0,\n");
    fprintf(out, "    \"frames\": [\n");

    for (int f = 0; f < 60; f++) {
        fprintf(out, "      {\n");
        fprintf(out, "        \"frame\": %d,\n", f);
        // Thrust every frame (thrustWait will throttle it)
        sim_ship_frame(&ship, 0, 0, 1, 0, 0);
        json_ship_state(&ship, "        ");
        fprintf(out, "      }%s\n", f < 59 ? "," : "");
    }

    fprintf(out, "    ]\n");
    fprintf(out, "  }");
}

// Scenario: turning left, then thrusting
static void scenario_turn_and_thrust(void) {
    SimShip ship;
    sim_ship_init(&ship, 5000, 5000, 0);

    fprintf(out, "  \"turn_and_thrust\": {\n");
    fprintf(out, "    \"description\": \"Turn left 4 steps then thrust for 30 frames\",\n");
    fprintf(out, "    \"ship\": \"human_cruiser\",\n");
    fprintf(out, "    \"frames\": [\n");

    int total = 40;
    for (int f = 0; f < total; f++) {
        fprintf(out, "      {\n");
        fprintf(out, "        \"frame\": %d,\n", f);
        if (f < 10) {
            // Turn left for 10 frames (turnWait=1 so ~5 actual turns)
            sim_ship_frame(&ship, 1, 0, 0, 0, 0);
        } else {
            // Then thrust
            sim_ship_frame(&ship, 0, 0, 1, 0, 0);
        }
        json_ship_state(&ship, "        ");
        fprintf(out, "      }%s\n", f < total - 1 ? "," : "");
    }

    fprintf(out, "    ]\n");
    fprintf(out, "  }");
}

// Scenario: thrust to max speed, then keep thrusting (tests speed cap)
static void scenario_max_speed(void) {
    SimShip ship;
    sim_ship_init(&ship, 5000, 5000, 0);

    fprintf(out, "  \"max_speed\": {\n");
    fprintf(out, "    \"description\": \"Thrust for 120 frames to reach and stay at max speed\",\n");
    fprintf(out, "    \"ship\": \"human_cruiser\",\n");
    fprintf(out, "    \"frames\": [\n");

    int total = 120;
    for (int f = 0; f < total; f++) {
        fprintf(out, "      {\n");
        fprintf(out, "        \"frame\": %d,\n", f);
        sim_ship_frame(&ship, 0, 0, 1, 0, 0);
        json_ship_state(&ship, "        ");
        fprintf(out, "      }%s\n", f < total - 1 ? "," : "");
    }

    fprintf(out, "    ]\n");
    fprintf(out, "  }");
}

// Scenario: thrust at angle while at max speed (direction change)
static void scenario_thrust_at_angle(void) {
    SimShip ship;
    sim_ship_init(&ship, 5000, 5000, 0);

    fprintf(out, "  \"thrust_at_angle\": {\n");
    fprintf(out, "    \"description\": \"Reach max speed facing right, then turn and thrust at angle\",\n");
    fprintf(out, "    \"ship\": \"human_cruiser\",\n");
    fprintf(out, "    \"frames\": [\n");

    int total = 80;
    for (int f = 0; f < total; f++) {
        fprintf(out, "      {\n");
        fprintf(out, "        \"frame\": %d,\n", f);
        if (f < 40) {
            // Build up to max speed facing right
            sim_ship_frame(&ship, 0, 0, 1, 0, 0);
        } else if (f < 50) {
            // Turn right (toward down)
            sim_ship_frame(&ship, 0, 1, 1, 0, 0);
        } else {
            // Keep thrusting in new direction
            sim_ship_frame(&ship, 0, 0, 1, 0, 0);
        }
        json_ship_state(&ship, "        ");
        fprintf(out, "      }%s\n", f < total - 1 ? "," : "");
    }

    fprintf(out, "    ]\n");
    fprintf(out, "  }");
}

// Scenario: energy drain and regeneration
static void scenario_energy(void) {
    SimShip ship;
    sim_ship_init(&ship, 5000, 5000, 0);

    fprintf(out, "  \"energy\": {\n");
    fprintf(out, "    \"description\": \"Fire weapon twice, then wait for energy regen\",\n");
    fprintf(out, "    \"ship\": \"human_cruiser\",\n");
    fprintf(out, "    \"frames\": [\n");

    int total = 80;
    for (int f = 0; f < total; f++) {
        fprintf(out, "      {\n");
        fprintf(out, "        \"frame\": %d,\n", f);
        if (f == 0 || f == 15) {
            // Fire weapon at frame 0 and 15
            sim_ship_frame(&ship, 0, 0, 0, 1, 0);
        } else {
            // Wait
            sim_ship_frame(&ship, 0, 0, 0, 0, 0);
        }
        json_ship_state(&ship, "        ");
        fprintf(out, "      }%s\n", f < total - 1 ? "," : "");
    }

    fprintf(out, "    ]\n");
    fprintf(out, "  }");
}

// Scenario: all 16 facings - set velocity in each direction
static void scenario_all_facings(void) {
    fprintf(out, "  \"all_facings\": {\n");
    fprintf(out, "    \"description\": \"Velocity vector for each of the 16 ship facings at max thrust\",\n");
    fprintf(out, "    \"ship\": \"human_cruiser\",\n");
    fprintf(out, "    \"facings\": [\n");

    for (int f = 0; f < 16; f++) {
        VELOCITY_DESC vel;
        ZeroVelocityComponents(&vel);
        SetVelocityVector(&vel, HUMAN_MAX_THRUST, f);

        SIZE vx, vy;
        GetCurrentVelocityComponents(&vel, &vx, &vy);

        fprintf(out, "      { \"facing\": %d, \"vx\": %d, \"vy\": %d, \"angle\": %d }%s\n",
            f, (int)vx, (int)vy, (int)vel.TravelAngle, f < 15 ? "," : "");
    }

    fprintf(out, "    ]\n");
    fprintf(out, "  }");
}

// Scenario: ship drifting toward planet with gravity
static void scenario_gravity_well(void) {
    SimShip ship;
    // Planet at center (3000, 3000), ship nearby at (3400, 3000)
    // 400 world units = 100 display units, within GRAVITY_THRESHOLD (255)
    int planet_x = 3000, planet_y = 3000;
    sim_ship_init(&ship, 3400, 3000, 0);

    fprintf(out, "  \"gravity_well\": {\n");
    fprintf(out, "    \"description\": \"Ship near planet, no thrust, gravity pulls it in\",\n");
    fprintf(out, "    \"ship\": \"human_cruiser\",\n");
    fprintf(out, "    \"planet\": { \"x\": %d, \"y\": %d },\n", planet_x, planet_y);
    fprintf(out, "    \"frames\": [\n");

    int total = 80;
    for (int f = 0; f < total; f++) {
        fprintf(out, "      {\n");
        fprintf(out, "        \"frame\": %d,\n", f);
        // No player input, just gravity
        sim_ship_frame(&ship, 0, 0, 0, 0, 0);
        sim_apply_gravity(&ship, planet_x, planet_y);
        json_ship_state(&ship, "        ");
        fprintf(out, "      }%s\n", f < total - 1 ? "," : "");
    }

    fprintf(out, "    ]\n");
    fprintf(out, "  }");
}

// Scenario: ship thrusting toward planet (gravity whip attempt)
static void scenario_gravity_whip(void) {
    SimShip ship;
    // Planet at center, ship above thrusting down toward it
    int planet_x = 3000, planet_y = 3000;
    sim_ship_init(&ship, 3000, 2200, 8); // facing 8 = down

    fprintf(out, "  \"gravity_whip\": {\n");
    fprintf(out, "    \"description\": \"Ship thrusts toward planet, enters gravity well, exceeds max speed\",\n");
    fprintf(out, "    \"ship\": \"human_cruiser\",\n");
    fprintf(out, "    \"planet\": { \"x\": %d, \"y\": %d },\n", planet_x, planet_y);
    fprintf(out, "    \"frames\": [\n");

    int total = 100;
    for (int f = 0; f < total; f++) {
        fprintf(out, "      {\n");
        fprintf(out, "        \"frame\": %d,\n", f);
        // Thrust every frame toward planet
        sim_ship_frame(&ship, 0, 0, 1, 0, 0);
        sim_apply_gravity(&ship, planet_x, planet_y);
        json_ship_state(&ship, "        ");
        fprintf(out, "      }%s\n", f < total - 1 ? "," : "");
    }

    fprintf(out, "    ]\n");
    fprintf(out, "  }");
}

static void scenario_human_nuke_straight(void) {
    SimProjectile nuke;
    int total = 120;
    sim_human_nuke_init(&nuke, 5000, 5000, 0);

    fprintf(out, "  \"human_nuke_straight\": {\n");
    fprintf(out, "    \"description\": \"Human Cruiser nuke fired straight ahead without tracking\",\n");
    fprintf(out, "    \"ship\": \"human_cruiser\",\n");
    fprintf(out, "    \"frames\": [\n");

    for (int f = 0; f < total; f++) {
        fprintf(out, "      {\n");
        fprintf(out, "        \"frame\": %d,\n", f);
        sim_human_nuke_frame(&nuke);
        json_projectile_state(&nuke, "        ");
        fprintf(out, "      }%s\n", f < total - 1 ? "," : "");
    }

    fprintf(out, "    ]\n");
    fprintf(out, "  }");
}

static void scenario_human_nuke_homing(void) {
    SimProjectile nuke;
    int target_x = 5600;
    int target_y = 4100;
    int target_vx = 6;
    int total = 120;
    sim_human_nuke_init(&nuke, 5000, 5000, 0);

    fprintf(out, "  \"human_nuke_homing\": {\n");
    fprintf(out, "    \"description\": \"Human Cruiser nuke tracks a slowly moving target ship\",\n");
    fprintf(out, "    \"ship\": \"human_cruiser\",\n");
    fprintf(out, "    \"frames\": [\n");

    for (int f = 0; f < total; f++) {
        fprintf(out, "      {\n");
        fprintf(out, "        \"frame\": %d,\n", f);
        sim_human_nuke_homing_frame(&nuke, target_x, target_y);
        fprintf(out, "        \"targetX\": %d,\n", target_x);
        fprintf(out, "        \"targetY\": %d,\n", target_y);
        fprintf(out, "        \"facing\": %d,\n", (int)nuke.facing);
        json_projectile_state(&nuke, "        ");
        fprintf(out, "      }%s\n", f < total - 1 ? "," : "");
        target_x += target_vx;
    }

    fprintf(out, "    ]\n");
    fprintf(out, "  }");
}

static void scenario_androsynth_bubble_targeted(void) {
    SimProjectile bubble;
    int target_x = 5600;
    int target_y = 4100;
    int target_vx = 4;
    int total = 30;
    andro_rng_state = 0x00C0FFEEu;
    sim_androsynth_bubble_init(&bubble, 5000, 5000, 0);

    fprintf(out, "  \"androsynth_bubble_targeted\": {\n");
    fprintf(out, "    \"description\": \"Androsynth bubble tracks a slowly moving target with SC2-style spread\",\n");
    fprintf(out, "    \"ship\": \"androsynth_guardian\",\n");
    fprintf(out, "    \"frames\": [\n");

    for (int f = 0; f < total; f++) {
        fprintf(out, "      {\n");
        fprintf(out, "        \"frame\": %d,\n", f);
        sim_androsynth_bubble_frame(&bubble, target_x, target_y);
        fprintf(out, "        \"targetX\": %d,\n", target_x);
        fprintf(out, "        \"targetY\": %d,\n", target_y);
        fprintf(out, "        \"facing\": %d,\n", (int)bubble.facing);
        json_projectile_state(&bubble, "        ");
        fprintf(out, "      }%s\n", f < total - 1 ? "," : "");
        target_x += target_vx;
    }

    fprintf(out, "    ]\n");
    fprintf(out, "  }");
}

static void scenario_androsynth_bubble_two_shots(void) {
    SimProjectile first_bubble;
    SimProjectile second_bubble;
    SIZE vx, vy;
    int target_x = 5600;
    int target_y = 4100;
    int total = 8;
    andro_rng_state = 0x00C0FFEEu;
    sim_androsynth_bubble_init(&first_bubble, 5000, 5000, 0);

    for (int f = 0; f < total; f++) {
        sim_androsynth_bubble_frame(&first_bubble, target_x, target_y);
    }

    sim_androsynth_bubble_init(&second_bubble, 5000, 5000, 0);

    fprintf(out, "  \"androsynth_bubble_two_shots\": {\n");
    fprintf(out, "    \"description\": \"Two Androsynth bubbles fired in one SC2 simulation do not restart the same random path\",\n");
    fprintf(out, "    \"ship\": \"androsynth_guardian\",\n");
    GetCurrentVelocityComponents(&first_bubble.velocity, &vx, &vy);
    fprintf(out, "    \"first\": {\n");
    fprintf(out, "      \"x\": %d,\n", first_bubble.pos_x);
    fprintf(out, "      \"y\": %d,\n", first_bubble.pos_y);
    fprintf(out, "      \"vx\": %d,\n", (int)vx);
    fprintf(out, "      \"vy\": %d\n", (int)vy);
    fprintf(out, "    },\n");
    fprintf(out, "    \"secondFrames\": [\n");

    for (int f = 0; f < total; f++) {
        fprintf(out, "      {\n");
        fprintf(out, "        \"frame\": %d,\n", f);
        sim_androsynth_bubble_frame(&second_bubble, target_x, target_y);
        GetCurrentVelocityComponents(&second_bubble.velocity, &vx, &vy);
        fprintf(out, "        \"x\": %d,\n", second_bubble.pos_x);
        fprintf(out, "        \"y\": %d,\n", second_bubble.pos_y);
        fprintf(out, "        \"vx\": %d,\n", (int)vx);
        fprintf(out, "        \"vy\": %d\n", (int)vy);
        fprintf(out, "      }%s\n", f < total - 1 ? "," : "");
    }

    fprintf(out, "    ]\n");
    fprintf(out, "  }");
}

// --- Main ---

int main(int argc, char *argv[]) {
    const char *output_file = "reference.json";

    if (argc > 1)
        output_file = argv[1];

    out = fopen(output_file, "w");
    if (!out) {
        fprintf(stderr, "Error: cannot open %s for writing\n", output_file);
        return 1;
    }

    fprintf(out, "{\n");

    scenario_collision_cooldowns();
    fprintf(out, ",\n");

    scenario_collision_existing_cooldowns();
    fprintf(out, ",\n");

    scenario_collision_head_on();
    fprintf(out, ",\n");

    scenario_collision_moving_into_stationary();
    fprintf(out, ",\n");

    scenario_collision_asymmetric_head_on();
    fprintf(out, ",\n");

    scenario_thrust_straight();
    fprintf(out, ",\n");

    scenario_turn_and_thrust();
    fprintf(out, ",\n");

    scenario_max_speed();
    fprintf(out, ",\n");

    scenario_thrust_at_angle();
    fprintf(out, ",\n");

    scenario_energy();
    fprintf(out, ",\n");

    scenario_all_facings();
    fprintf(out, ",\n");

    scenario_gravity_well();
    fprintf(out, ",\n");

    scenario_gravity_whip();
    fprintf(out, ",\n");

    scenario_human_nuke_straight();
    fprintf(out, ",\n");

    scenario_human_nuke_homing();
    fprintf(out, ",\n");

    scenario_androsynth_bubble_targeted();
    fprintf(out, ",\n");

    scenario_androsynth_bubble_two_shots();
    fprintf(out, "\n");

    fprintf(out, "}\n");

    fclose(out);
    fprintf(stderr, "Reference data written to %s\n", output_file);
    return 0;
}
