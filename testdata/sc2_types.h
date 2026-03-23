// Minimal type definitions extracted from SC2/UQM source
// This avoids needing the full SC2 include chain
#ifndef SC2_TYPES_H
#define SC2_TYPES_H

#include <stdint.h>
#include <string.h>
#include <stdbool.h>

// Base types (from sc2/src/types.h + libs/compiler.h)
typedef uint8_t   uint8;
typedef int8_t    sint8;
typedef uint16_t  uint16;
typedef int16_t   sint16;
typedef uint32_t  uint32;
typedef int32_t   sint32;
typedef uint64_t  uint64;
typedef int64_t   sint64;

typedef uint8     BYTE;
typedef uint8     UBYTE;
typedef sint8     SBYTE;
typedef uint16    UWORD;
typedef sint16    SWORD;
typedef uint32    DWORD;
typedef sint32    SDWORD;
typedef uint64    QWORD;
typedef sint64    SQWORD;

typedef UWORD     COUNT;
typedef SWORD     SIZE;
typedef SWORD     COORD;

// Macros (from libs/compiler.h)
#define MAKE_WORD(lo, hi)   ((UWORD) ((BYTE) (hi) << 8) | (BYTE) (lo))
#define LOBYTE(x)    ((BYTE) ((UWORD) (x)))
#define HIBYTE(x)    ((BYTE) ((UWORD) (x) >> 8))

// Geometry (from libs/gfxlib.h)
typedef struct { COORD width, height; } EXTENT;
typedef struct { COORD x, y; } POINT;

// Angle system (from uqm/units.h)
#define CIRCLE_SHIFT 6
#define FULL_CIRCLE (1 << CIRCLE_SHIFT)
#define HALF_CIRCLE (FULL_CIRCLE >> 1)
#define QUADRANT (FULL_CIRCLE >> 2)
#define OCTANT (FULL_CIRCLE >> 3)

#define FACING_SHIFT 4
#define ANGLE_TO_FACING(a) (((a)+(1<<(CIRCLE_SHIFT-FACING_SHIFT-1))) \
                                        >>(CIRCLE_SHIFT-FACING_SHIFT))
#define FACING_TO_ANGLE(f) ((f)<<(CIRCLE_SHIFT-FACING_SHIFT))
#define NORMALIZE_ANGLE(a) ((DWORD)((a)&(FULL_CIRCLE-1)))
#define NORMALIZE_FACING(f) ((DWORD)((f)&((1 << FACING_SHIFT)-1)))

// Trig (from uqm/units.h)
#define SIN_SHIFT 14
#define SIN_SCALE (1 << SIN_SHIFT)
#define INT_ADJUST(x) ((x)<<SIN_SHIFT)
#define FLT_ADJUST(x) (SIZE)((x)*SIN_SCALE)

extern SDWORD sinetab[];
#define SINVAL(a) sinetab[NORMALIZE_ANGLE(a)]
#define COSVAL(a) SINVAL((a)+QUADRANT)
#define SINE(a,m) ((SDWORD)((((long)SINVAL(a))*(long)(m))>>SIN_SHIFT))
#define COSINE(a,m) SINE((a)+QUADRANT,m)
extern COUNT ARCTAN (SDWORD delta_x, SDWORD delta_y);

// Coordinate system (from uqm/units.h)
#define ONE_SHIFT 2
#define SCALED_ONE (1 << ONE_SHIFT)
#define DISPLAY_TO_WORLD(x) ((x)<<ONE_SHIFT)
#define WORLD_TO_DISPLAY(x) ((x)>>ONE_SHIFT)

// Resolution (we use non-HD = 0)
#define RESOLUTION_FACTOR 0
#define RES_SCALE(a) ((a) << RESOLUTION_FACTOR)

// Velocity (from uqm/velocity.h)
#define VELOCITY_SHIFT 5
#define VELOCITY_SCALE (1<<VELOCITY_SHIFT)
#define VELOCITY_TO_WORLD(v) ((v)>>VELOCITY_SHIFT)
#define WORLD_TO_VELOCITY(l) ((l)<<VELOCITY_SHIFT)

typedef struct velocity_desc {
    COUNT TravelAngle;
    EXTENT vector;
    EXTENT fract;
    EXTENT error;
    EXTENT incr;
} VELOCITY_DESC;

#define ZeroVelocityComponents(pv) memset(pv,0,sizeof (*(pv)))
#define GetVelocityTravelAngle(pv) (pv)->TravelAngle

// Status flags (from uqm/races.h)
typedef DWORD STATUS_FLAGS;
#define LEFT                    (1 << 0)
#define RIGHT                   (1 << 1)
#define THRUST                  (1 << 2)
#define WEAPON                  (1 << 3)
#define SPECIAL                 (1 << 4)
#define SHIP_BEYOND_MAX_SPEED   (1 << 6)
#define SHIP_AT_MAX_SPEED       (1 << 7)
#define SHIP_IN_GRAVITY_WELL    (1 << 8)

static inline DWORD
VelocitySquared (SIZE dx, SIZE dy)
{
    return (DWORD)((long)dx * dx + (long)dy * dy);
}

// Velocity functions (implemented in sc2_velocity.c)
void GetCurrentVelocityComponents (VELOCITY_DESC *velocityptr, SIZE *pdx, SIZE *pdy);
void GetNextVelocityComponents (VELOCITY_DESC *velocityptr, SIZE *pdx, SIZE *pdy, COUNT num_frames);
void SetVelocityVector (VELOCITY_DESC *velocityptr, SDWORD magnitude, COUNT facing);
void SetVelocityComponents (VELOCITY_DESC *velocityptr, SDWORD dx, SDWORD dy);
void DeltaVelocityComponents (VELOCITY_DESC *velocityptr, SDWORD dx, SDWORD dy);

#endif // SC2_TYPES_H
