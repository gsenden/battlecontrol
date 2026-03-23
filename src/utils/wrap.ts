export interface WrappedPoint {
  x: number;
  y: number;
}

function wrapAxis(value: number, size: number): number {
  return ((value % size) + size) % size;
}

export function wrapPoint(x: number, y: number, width: number, height: number): WrappedPoint {
  return {
    x: wrapAxis(x, width),
    y: wrapAxis(y, height),
  };
}

export function shortestWrappedDelta(from: number, to: number, size: number): number {
  let delta = to - from;

  if (delta > size / 2) {
    delta -= size;
  } else if (delta < -size / 2) {
    delta += size;
  }

  return delta;
}
