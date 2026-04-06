// Exact copy of SC2 velocity functions and trig tables
// Source files: sc2/src/uqm/velocity.c, sc2/src/uqm/trans.c
// We copy instead of include because the SC2 headers have deep dependency chains

#include "sc2_types.h"

// --- Sine table (from trans.c) ---

SDWORD sinetab[] = {
    -FLT_ADJUST(1.000000),
    -FLT_ADJUST(0.995185),
    -FLT_ADJUST(0.980785),
    -FLT_ADJUST(0.956940),
    -FLT_ADJUST(0.923880),
    -FLT_ADJUST(0.881921),
    -FLT_ADJUST(0.831470),
    -FLT_ADJUST(0.773010),
    -FLT_ADJUST(0.707107),
    -FLT_ADJUST(0.634393),
    -FLT_ADJUST(0.555570),
    -FLT_ADJUST(0.471397),
    -FLT_ADJUST(0.382683),
    -FLT_ADJUST(0.290285),
    -FLT_ADJUST(0.195090),
    -FLT_ADJUST(0.098017),
    FLT_ADJUST(0.000000),
    FLT_ADJUST(0.098017),
    FLT_ADJUST(0.195090),
    FLT_ADJUST(0.290285),
    FLT_ADJUST(0.382683),
    FLT_ADJUST(0.471397),
    FLT_ADJUST(0.555570),
    FLT_ADJUST(0.634393),
    FLT_ADJUST(0.707107),
    FLT_ADJUST(0.773010),
    FLT_ADJUST(0.831470),
    FLT_ADJUST(0.881921),
    FLT_ADJUST(0.923880),
    FLT_ADJUST(0.956940),
    FLT_ADJUST(0.980785),
    FLT_ADJUST(0.995185),
    FLT_ADJUST(1.000000),
    FLT_ADJUST(0.995185),
    FLT_ADJUST(0.980785),
    FLT_ADJUST(0.956940),
    FLT_ADJUST(0.923880),
    FLT_ADJUST(0.881921),
    FLT_ADJUST(0.831470),
    FLT_ADJUST(0.773010),
    FLT_ADJUST(0.707107),
    FLT_ADJUST(0.634393),
    FLT_ADJUST(0.555570),
    FLT_ADJUST(0.471397),
    FLT_ADJUST(0.382683),
    FLT_ADJUST(0.290285),
    FLT_ADJUST(0.195090),
    FLT_ADJUST(0.098017),
    FLT_ADJUST(0.000000),
    -FLT_ADJUST(0.098017),
    -FLT_ADJUST(0.195090),
    -FLT_ADJUST(0.290285),
    -FLT_ADJUST(0.382683),
    -FLT_ADJUST(0.471397),
    -FLT_ADJUST(0.555570),
    -FLT_ADJUST(0.634393),
    -FLT_ADJUST(0.707107),
    -FLT_ADJUST(0.773010),
    -FLT_ADJUST(0.831470),
    -FLT_ADJUST(0.881921),
    -FLT_ADJUST(0.923880),
    -FLT_ADJUST(0.956940),
    -FLT_ADJUST(0.980785),
    -FLT_ADJUST(0.995185),
};

// --- ARCTAN (from trans.c) ---

COUNT
ARCTAN (SDWORD delta_x, SDWORD delta_y)
{
    SDWORD v1, v2;
    static COUNT atantab[] = {
        0,0,1,1,1,2,2,2,2,3,3,3,4,4,4,4,5,5,5,5,6,6,6,6,7,7,7,7,7,7,8,8,8
    };

    v1 = delta_x;
    v2 = delta_y;
    if (v1 == 0 && v2 == 0)
        return (FULL_CIRCLE);

    if (v1 < 0) v1 = -v1;
    if (v2 < 0) v2 = -v2;

    if (v1 > v2)
        v1 = QUADRANT - atantab[(((DWORD)v2 << 5) + (v1 >> 1)) / v1];
    else
        v1 = atantab[(((DWORD)v1 << 5) + (v2 >> 1)) / v2];

    if (delta_x < 0) v1 = FULL_CIRCLE - v1;
    if (delta_y > 0) v1 = HALF_CIRCLE - v1;

    return (NORMALIZE_ANGLE (v1));
}

// --- Velocity functions (from velocity.c) ---

#define VELOCITY_REMAINDER(v) ((v) & (VELOCITY_SCALE - 1))

void
GetCurrentVelocityComponents (VELOCITY_DESC *velocityptr, SIZE *pdx, SIZE *pdy)
{
    *pdx = WORLD_TO_VELOCITY (velocityptr->vector.width)
        + (velocityptr->fract.width - (SIZE)HIBYTE (velocityptr->incr.width));
    *pdy = WORLD_TO_VELOCITY (velocityptr->vector.height)
        + (velocityptr->fract.height - (SIZE)HIBYTE (velocityptr->incr.height));
}

void
GetNextVelocityComponents (VELOCITY_DESC *velocityptr, SIZE *pdx, SIZE *pdy, COUNT num_frames)
{
    COUNT e;

    e = (COUNT)((COUNT)velocityptr->error.width +
                ((COUNT)velocityptr->fract.width * num_frames));

    *pdx = (velocityptr->vector.width * num_frames)
        + ((SIZE)((SBYTE)LOBYTE (velocityptr->incr.width))
           * (e >> VELOCITY_SHIFT));

    velocityptr->error.width = VELOCITY_REMAINDER (e);

    e = (COUNT)((COUNT)velocityptr->error.height +
                ((COUNT)velocityptr->fract.height * num_frames));

    *pdy = (velocityptr->vector.height * num_frames)
        + ((SIZE)((SBYTE)LOBYTE (velocityptr->incr.height))
           * (e >> VELOCITY_SHIFT));

    velocityptr->error.height = VELOCITY_REMAINDER (e);
}

void
SetVelocityVector (VELOCITY_DESC *velocityptr, SDWORD magnitude, COUNT facing)
{
    COUNT angle;
    SIZE dx, dy;

    angle = velocityptr->TravelAngle =
        FACING_TO_ANGLE (NORMALIZE_FACING (facing));
    magnitude = WORLD_TO_VELOCITY (magnitude);
    dx = COSINE (angle, magnitude);
    dy = SINE (angle, magnitude);
    if (dx >= 0)
    {
        velocityptr->vector.width = VELOCITY_TO_WORLD (dx);
        velocityptr->incr.width = MAKE_WORD ((BYTE)1, (BYTE)0);
    }
    else
    {
        dx = -dx;
        velocityptr->vector.width = -VELOCITY_TO_WORLD (dx);
        velocityptr->incr.width =
            MAKE_WORD ((BYTE)0xFF, (BYTE)(VELOCITY_REMAINDER (dx) << 1));
    }
    if (dy >= 0)
    {
        velocityptr->vector.height = VELOCITY_TO_WORLD (dy);
        velocityptr->incr.height = MAKE_WORD ((BYTE)1, (BYTE)0);
    }
    else
    {
        dy = -dy;
        velocityptr->vector.height = -VELOCITY_TO_WORLD (dy);
        velocityptr->incr.height =
            MAKE_WORD ((BYTE)0xFF, (BYTE)(VELOCITY_REMAINDER (dy) << 1));
    }

    velocityptr->fract.width = VELOCITY_REMAINDER (dx);
    velocityptr->fract.height = VELOCITY_REMAINDER (dy);
    velocityptr->error.width = velocityptr->error.height = 0;
}

void
SetVelocityComponents (VELOCITY_DESC *velocityptr, SDWORD dx, SDWORD dy)
{
    COUNT angle;

    if ((angle = ARCTAN (dx, dy)) == FULL_CIRCLE)
    {
        ZeroVelocityComponents (velocityptr);
    }
    else
    {
        if (dx >= 0)
        {
            velocityptr->vector.width = VELOCITY_TO_WORLD (dx);
            velocityptr->incr.width = MAKE_WORD ((BYTE)1, (BYTE)0);
        }
        else
        {
            dx = -dx;
            velocityptr->vector.width = -VELOCITY_TO_WORLD (dx);
            velocityptr->incr.width =
                MAKE_WORD ((BYTE)0xFF, (BYTE)(VELOCITY_REMAINDER (dx) << 1));
        }
        if (dy >= 0)
        {
            velocityptr->vector.height = VELOCITY_TO_WORLD (dy);
            velocityptr->incr.height = MAKE_WORD ((BYTE)1, (BYTE)0);
        }
        else
        {
            dy = -dy;
            velocityptr->vector.height = -VELOCITY_TO_WORLD (dy);
            velocityptr->incr.height =
                MAKE_WORD ((BYTE)0xFF, (BYTE)(VELOCITY_REMAINDER (dy) << 1));
        }

        velocityptr->fract.width = VELOCITY_REMAINDER (dx);
        velocityptr->fract.height = VELOCITY_REMAINDER (dy);
        velocityptr->error.width = velocityptr->error.height = 0;
    }

    velocityptr->TravelAngle = angle;
}

void
DeltaVelocityComponents (VELOCITY_DESC *velocityptr, SDWORD dx, SDWORD dy)
{
    dx += WORLD_TO_VELOCITY (velocityptr->vector.width)
        + (velocityptr->fract.width - (SIZE)HIBYTE (velocityptr->incr.width));
    dy += WORLD_TO_VELOCITY (velocityptr->vector.height)
        + (velocityptr->fract.height - (SIZE)HIBYTE (velocityptr->incr.height));

    SetVelocityComponents (velocityptr, dx, dy);
}

// --- inertial_thrust (from ship.c) ---

// Simplified version without ELEMENT/STARSHIP dependencies
// Takes parameters directly instead of reading from game state

STATUS_FLAGS
inertial_thrust (VELOCITY_DESC *VelocityPtr, COUNT ShipFacing,
    STATUS_FLAGS cur_status_flags,
    COUNT max_thrust, COUNT thrust_increment)
{
#define MAX_ALLOWED_SPEED     WORLD_TO_VELOCITY (DISPLAY_TO_WORLD (RES_SCALE (18)))
#define MAX_ALLOWED_SPEED_SQR ((DWORD)MAX_ALLOWED_SPEED * MAX_ALLOWED_SPEED)

    COUNT CurrentAngle, TravelAngle;

    CurrentAngle = FACING_TO_ANGLE (ShipFacing);
    TravelAngle = GetVelocityTravelAngle (VelocityPtr);

    if (thrust_increment == max_thrust)
    {   // inertialess (Skiff)
        SetVelocityVector (VelocityPtr, max_thrust, ShipFacing);
        return (SHIP_AT_MAX_SPEED);
    }
    else if (TravelAngle == CurrentAngle
            && (cur_status_flags & (SHIP_AT_MAX_SPEED | SHIP_BEYOND_MAX_SPEED))
            && !(cur_status_flags & SHIP_IN_GRAVITY_WELL))
    {   // already at max
        return (cur_status_flags & (SHIP_AT_MAX_SPEED | SHIP_BEYOND_MAX_SPEED));
    }
    else
    {
        SIZE delta_x, delta_y;
        SIZE cur_delta_x, cur_delta_y;
        DWORD desired_speed, max_speed;
        DWORD current_speed;

        thrust_increment = WORLD_TO_VELOCITY (thrust_increment);
        GetCurrentVelocityComponents (VelocityPtr, &cur_delta_x, &cur_delta_y);
        current_speed = VelocitySquared (cur_delta_x, cur_delta_y);
        delta_x = cur_delta_x + COSINE (CurrentAngle, thrust_increment);
        delta_y = cur_delta_y + SINE (CurrentAngle, thrust_increment);
        desired_speed = VelocitySquared (delta_x, delta_y);
        max_speed = VelocitySquared (WORLD_TO_VELOCITY (max_thrust), 0);

        if (desired_speed <= max_speed)
        {   // normal acceleration
            SetVelocityComponents (VelocityPtr, delta_x, delta_y);
        }
        else if (((cur_status_flags & SHIP_IN_GRAVITY_WELL)
                && desired_speed <= MAX_ALLOWED_SPEED_SQR)
                || desired_speed < current_speed)
        {   // gravity well or decelerating
            SetVelocityComponents (VelocityPtr, delta_x, delta_y);
            return (SHIP_AT_MAX_SPEED | SHIP_BEYOND_MAX_SPEED);
        }
        else if (TravelAngle == CurrentAngle)
        {   // at max, same direction
            if (current_speed <= max_speed)
                SetVelocityVector (VelocityPtr, max_thrust, ShipFacing);
            return (SHIP_AT_MAX_SPEED);
        }
        else
        {   // at max, thrusting at angle
            VELOCITY_DESC v = *VelocityPtr;

            DeltaVelocityComponents (&v,
                    COSINE (CurrentAngle, thrust_increment >> 1)
                    - COSINE (TravelAngle, thrust_increment),
                    SINE (CurrentAngle, thrust_increment >> 1)
                    - SINE (TravelAngle, thrust_increment));
            GetCurrentVelocityComponents (&v, &cur_delta_x, &cur_delta_y);
            desired_speed = VelocitySquared (cur_delta_x, cur_delta_y);
            if (desired_speed > max_speed)
            {
                if (desired_speed < current_speed)
                    *VelocityPtr = v;
                return (SHIP_AT_MAX_SPEED | SHIP_BEYOND_MAX_SPEED);
            }

            *VelocityPtr = v;
        }

        return (0);
    }
}
