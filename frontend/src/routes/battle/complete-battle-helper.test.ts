import { describe, it, expect, vi } from 'vitest';

describe('completeBattleAndNavigate', () => {
	it('awaits completeGame before navigating to battle-result', async () => {
		const callOrder: string[] = [];
		const completeGame = vi.fn().mockImplementation(async () => {
			callOrder.push('completeGame');
		});
		const goto = vi.fn().mockImplementation(() => {
			callOrder.push('goto');
		});
		const gameId = 'game-123';
		const winner = 'Captain Win';

		const { completeBattleAndNavigate } = await import('./complete-battle-helper.js');

		await completeBattleAndNavigate(completeGame, goto, gameId, winner);

		expect(callOrder).toEqual(['completeGame', 'goto']);
	});
});
