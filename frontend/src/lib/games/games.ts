import { toReadableErrorMessage, type UserDto } from '$lib/auth/auth.js';

export interface GamePlayerDto {
	user: UserDto;
	selected_race?: string | null;
}

export interface GameDto {
	id: string;
	name: string;
	game_type: string;
	max_players: number;
	is_private: boolean;
	password?: string | null;
	creator: UserDto;
	players: GamePlayerDto[];
}

export interface CreateGameRequestDto {
	name: string;
	game_type: string;
	max_players: number;
	is_private: boolean;
	password?: string | null;
}

export interface JoinGameRequestDto {
	password?: string | null;
}

export interface SaveSelectedRaceRequestDto {
	selected_race: string;
}

export interface GameRoomEventDto {
	kind: 'snapshot' | 'started' | 'cancelled';
	game_id: string;
	game?: GameDto | null;
}

export async function createGame(game: CreateGameRequestDto): Promise<GameDto> {
	const response = await fetch('/games', {
		method: 'POST',
		credentials: 'same-origin',
		headers: {
			'content-type': 'application/json',
		},
		body: JSON.stringify(game),
	});

	if (!response.ok) {
		throw await parseGameError(response);
	}

	return response.json() as Promise<GameDto>;
}

export async function listGames(): Promise<GameDto[]> {
	const response = await fetch('/games', {
		method: 'GET',
		credentials: 'same-origin',
	});

	if (!response.ok) {
		throw await parseGameError(response);
	}

	return response.json() as Promise<GameDto[]>;
}

export async function getGame(gameId: string): Promise<GameDto> {
	const response = await fetch(`/games/${gameId}`, {
		method: 'GET',
		credentials: 'same-origin',
	});

	if (!response.ok) {
		throw await parseGameError(response);
	}

	return response.json() as Promise<GameDto>;
}

export async function getGameInstance(gameId: string): Promise<GameDto> {
	const response = await fetch(`/games/${gameId}/instance`, {
		method: 'GET',
		credentials: 'same-origin',
	});

	if (!response.ok) {
		throw await parseGameError(response);
	}

	return response.json() as Promise<GameDto>;
}

export async function joinGame(gameId: string, request: JoinGameRequestDto): Promise<GameDto> {
	const response = await fetch(`/games/${gameId}/join`, {
		method: 'POST',
		credentials: 'same-origin',
		headers: {
			'content-type': 'application/json',
		},
		body: JSON.stringify(request),
	});

	if (!response.ok) {
		throw await parseGameError(response);
	}

	return response.json() as Promise<GameDto>;
}

export async function saveSelectedRace(gameId: string, request: SaveSelectedRaceRequestDto): Promise<GameDto> {
	const response = await fetch(`/games/${gameId}/race`, {
		method: 'PUT',
		credentials: 'same-origin',
		headers: {
			'content-type': 'application/json',
		},
		body: JSON.stringify(request),
	});

	if (!response.ok) {
		throw await parseGameError(response);
	}

	return response.json() as Promise<GameDto>;
}

export async function leaveGame(gameId: string): Promise<void> {
	const response = await fetch(`/games/${gameId}/leave`, {
		method: 'POST',
		credentials: 'same-origin',
	});

	if (!response.ok) {
		throw await parseGameError(response);
	}
}

export async function cancelGame(gameId: string): Promise<void> {
	const response = await fetch(`/games/${gameId}/cancel`, {
		method: 'POST',
		credentials: 'same-origin',
	});

	if (!response.ok) {
		throw await parseGameError(response);
	}
}

export async function startGame(gameId: string): Promise<GameDto> {
	const response = await fetch(`/games/${gameId}/start`, {
		method: 'POST',
		credentials: 'same-origin',
	});

	if (!response.ok) {
		throw await parseGameError(response);
	}

	return response.json() as Promise<GameDto>;
}

export async function completeGame(gameId: string): Promise<void> {
	const response = await fetch(`/games/${gameId}/complete`, {
		method: 'POST',
		credentials: 'same-origin',
	});

	if (!response.ok) {
		throw await parseGameError(response);
	}
}

async function parseGameError(response: Response): Promise<Error> {
	try {
		return new Error(toReadableErrorMessage(await response.json()));
	} catch {
		return new Error(toReadableErrorMessage(new Error(response.statusText || 'Request failed')));
	}
}
