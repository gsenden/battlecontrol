const SATURN_FACING_COUNT = 16;

export function velocityToSaturnFrame(vx: number, vy: number): number {
  let angle = Math.atan2(vy, vx) + Math.PI / 2;
  angle = ((angle % (2 * Math.PI)) + (2 * Math.PI)) % (2 * Math.PI);
  return Math.round(angle / (2 * Math.PI / SATURN_FACING_COUNT)) % SATURN_FACING_COUNT;
}
