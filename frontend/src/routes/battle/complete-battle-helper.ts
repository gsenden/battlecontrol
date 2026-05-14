type CompleteGame = (gameId: string) => Promise<unknown>;
type Navigate = (url: string) => Promise<unknown> | unknown;

export async function completeBattleAndNavigate(
	completeGame: CompleteGame,
	goto: Navigate,
	gameId: string,
	winner: string,
) {
	await completeGame(gameId);
	await goto(`/battle-result?winner=${encodeURIComponent(winner)}`);
}
