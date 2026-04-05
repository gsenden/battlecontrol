import { describe, expect, it, vi } from 'vitest';
import { buildDebugLogSnapshot } from './debug-overlay.js';

describe('buildDebugLogSnapshot', () => {
  it('includes the current status and logs', () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date('2026-04-05T12:00:00.000Z'));

    expect(buildDebugLogSnapshot('rocket life=42', ['hit x=10 y=20'], ['frame=0 x=1 y=2'])).toEqual({
      status: 'rocket life=42',
      logs: ['hit x=10 y=20'],
      trace: ['frame=0 x=1 y=2'],
      updatedAt: '2026-04-05T12:00:00.000Z',
    });
  });
});
