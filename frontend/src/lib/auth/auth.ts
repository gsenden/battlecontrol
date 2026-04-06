import { translate } from '$lib/i18n/i18n.js';
import type { I18nKey } from '$lib/i18n/translations.js';

const AUTH_USER_STORAGE_KEY = 'battlecontrol.auth.user';
const AUTH_SERVER_LABEL = 'localhost:3000';

export interface UserDto {
	id: number;
	name: string;
}

interface LoginRequestDto {
	name: string;
}

interface PasskeyOptionsDto {
	publicKey: unknown;
}

interface PasskeyStartRequestDto {
	name: string;
}

interface PasskeyFinishRequestDto {
	name: string;
	credential: unknown;
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

export async function loginUser(name: string): Promise<UserDto> {
	const body: LoginRequestDto = { name };
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

export async function registerWithPasskey(name: string): Promise<UserDto> {
	ensurePasskeySupport();
	const body: PasskeyStartRequestDto = { name };
	const startResponse = await fetch('/auth/passkey/register/start', {
		method: 'POST',
		headers: {
			'content-type': 'application/json',
		},
		body: JSON.stringify(body),
	});

	if (!startResponse.ok) {
		throw await parseApiError(startResponse);
	}

	const options = await startResponse.json() as PasskeyOptionsDto;
	const credential = await navigator.credentials.create({
		publicKey: parseCreationOptions(options.publicKey),
	});

	if (!credential) {
		throw new Error(translate('AUTHENTICATION_FAILED'));
	}

	const finishResponse = await fetch('/auth/passkey/register/finish', {
		method: 'POST',
		headers: {
			'content-type': 'application/json',
		},
		body: JSON.stringify({
			name,
			credential: credential.toJSON(),
		} satisfies PasskeyFinishRequestDto),
	});

	if (!finishResponse.ok) {
		throw await parseApiError(finishResponse);
	}

	return finishResponse.json() as Promise<UserDto>;
}

export async function loginWithPasskey(name: string): Promise<UserDto> {
	ensurePasskeySupport();
	const body: PasskeyStartRequestDto = { name };
	const startResponse = await fetch('/auth/passkey/login/start', {
		method: 'POST',
		headers: {
			'content-type': 'application/json',
		},
		body: JSON.stringify(body),
	});

	if (!startResponse.ok) {
		throw await parseApiError(startResponse);
	}

	const options = await startResponse.json() as PasskeyOptionsDto;
	const credential = await navigator.credentials.get({
		publicKey: parseRequestOptions(options.publicKey),
	});

	if (!credential) {
		throw new Error(translate('AUTHENTICATION_FAILED'));
	}

	const finishResponse = await fetch('/auth/passkey/login/finish', {
		method: 'POST',
		headers: {
			'content-type': 'application/json',
		},
		body: JSON.stringify({
			name,
			credential: credential.toJSON(),
		} satisfies PasskeyFinishRequestDto),
	});

	if (!finishResponse.ok) {
		throw await parseApiError(finishResponse);
	}

	return finishResponse.json() as Promise<UserDto>;
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

	if (error instanceof Error && error.message) {
		return error.message;
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
	AuthenticationFailed: 'AUTHENTICATION_FAILED',
	UserAlreadyExists: 'USER_ALREADY_EXISTS',
	UserNotFound: 'USER_NOT_FOUND',
	ServerOffline: 'SERVER_OFFLINE',
	RequestTimeout: 'REQUEST_TIMEOUT',
	RequestFailed: 'REQUEST_FAILED',
	RoomNotFound: 'ROOM_NOT_FOUND',
	DatabaseError: 'DATABASE_ERROR',
};

type PublicKeyCredentialWithJsonParsers = typeof PublicKeyCredential & {
	parseCreationOptionsFromJSON?: (value: unknown) => CredentialCreationOptions['publicKey'];
	parseRequestOptionsFromJSON?: (value: unknown) => CredentialRequestOptions['publicKey'];
};

function ensurePasskeySupport() {
	if (
		typeof window === 'undefined'
		|| typeof navigator === 'undefined'
		|| typeof PublicKeyCredential === 'undefined'
		|| typeof navigator.credentials === 'undefined'
	) {
		throw new Error(translate('PASSKEY_UNAVAILABLE'));
	}
}

function parseCreationOptions(value: unknown): CredentialCreationOptions['publicKey'] {
	const parser = PublicKeyCredential as PublicKeyCredentialWithJsonParsers;
	if (!parser.parseCreationOptionsFromJSON) {
		throw new Error(translate('PASSKEY_UNAVAILABLE'));
	}
	return parser.parseCreationOptionsFromJSON(unwrapPublicKey(value));
}

function parseRequestOptions(value: unknown): CredentialRequestOptions['publicKey'] {
	const parser = PublicKeyCredential as PublicKeyCredentialWithJsonParsers;
	if (!parser.parseRequestOptionsFromJSON) {
		throw new Error(translate('PASSKEY_UNAVAILABLE'));
	}
	return parser.parseRequestOptionsFromJSON(unwrapPublicKey(value));
}

function unwrapPublicKey(value: unknown): unknown {
	if (
		typeof value === 'object'
		&& value !== null
		&& 'publicKey' in value
	) {
		return Reflect.get(value, 'publicKey');
	}
	return value;
}
