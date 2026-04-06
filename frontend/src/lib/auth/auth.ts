import { translate } from '$lib/i18n/i18n.js';
import type { I18nKey } from '$lib/i18n/translations.js';

const AUTH_USER_STORAGE_KEY = 'battlecontrol.auth.user';
const AUTH_SERVER_LABEL = 'localhost:3000';

export interface UserDto {
	id: number;
	name: string;
	email: string;
}

interface LoginRequestDto {
	email: string;
}

interface ApiError {
	code: string;
	params?: Record<string, string>;
}

export async function registerUser(name: string): Promise<UserDto> {
	const response = await fetch('/auth/user', {
		method: 'POST',
		headers: {
			'content-type': 'application/json',
		},
		body: JSON.stringify({ name }),
	});

	if (!response.ok) {
		throw await parseApiError(response);
	}

	return response.json() as Promise<UserDto>;
}

export async function loginUser(email: string): Promise<UserDto> {
	const body: LoginRequestDto = { email };
	const response = await fetch('/auth/login', {
		method: 'POST',
		headers: {
			'content-type': 'application/json',
		},
		body: JSON.stringify(body),
	});

	if (!response.ok) {
		throw await parseApiError(response);
	}

	return response.json() as Promise<UserDto>;
}

export function loadStoredUser(): UserDto | null {
	if (typeof window === 'undefined') {
		return null;
	}

	const raw = window.localStorage.getItem(AUTH_USER_STORAGE_KEY);
	if (!raw) {
		return null;
	}

	try {
		return JSON.parse(raw) as UserDto;
	} catch {
		window.localStorage.removeItem(AUTH_USER_STORAGE_KEY);
		return null;
	}
}

export function storeUser(user: UserDto) {
	window.localStorage.setItem(AUTH_USER_STORAGE_KEY, JSON.stringify(user));
}

export function toReadableErrorMessage(error: unknown): string {
	if (isApiError(error)) {
		const key = ERROR_KEY_TO_I18N[error.code];
		if (key) {
			return translate(key, error.params);
		}
	}

	if (error instanceof TypeError) {
		return translate('SERVER_OFFLINE', { server: AUTH_SERVER_LABEL });
	}

	return translate('REQUEST_FAILED', { server: AUTH_SERVER_LABEL });
}

async function parseApiError(response: Response): Promise<ApiError> {
	try {
		return await response.json() as ApiError;
	} catch {
		return { code: 'REQUEST_FAILED', params: { server: AUTH_SERVER_LABEL } };
	}
}

function isApiError(error: unknown): error is ApiError {
	return typeof error === 'object'
		&& error !== null
		&& 'code' in error
		&& typeof Reflect.get(error, 'code') === 'string';
}

const ERROR_KEY_TO_I18N: Record<string, I18nKey | undefined> = {
	UserAlreadyExists: 'USER_ALREADY_EXISTS',
	UserNotFound: 'USER_NOT_FOUND',
	ServerOffline: 'SERVER_OFFLINE',
	RequestTimeout: 'REQUEST_TIMEOUT',
	RequestFailed: 'REQUEST_FAILED',
	RoomNotFound: 'ROOM_NOT_FOUND',
	DatabaseError: 'DATABASE_ERROR',
};
