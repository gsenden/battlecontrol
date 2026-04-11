import { translate } from '$lib/i18n/i18n.js';
import type { I18nKey } from '$lib/i18n/translations.js';

const AUTH_SERVER_LABEL = '127.0.0.1:3000';

export interface UserDto {
	id: number;
	name: string;
	profile_image_url?: string | null;
}

export interface UserSettingsDto {
	turn_left_key: string;
	turn_right_key: string;
	thrust_key: string;
	music_enabled: boolean;
	music_volume: number;
	sound_effects_enabled: boolean;
	sound_effects_volume: number;
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

interface UpdateUserProfileRequestDto {
	name: string;
	profile_image_url: string;
}

interface ProfileImageUploadDto {
	profile_image_url: string;
}

interface ApiError {
	code: string;
	params?: Record<string, string>;
}

export async function registerUser(name: string): Promise<UserDto> {
	const response = await fetch('/auth/user', {
		method: 'POST',
		credentials: 'same-origin',
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
		credentials: 'same-origin',
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
		credentials: 'same-origin',
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
		credentials: 'same-origin',
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
		credentials: 'same-origin',
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
		credentials: 'same-origin',
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

export async function getCurrentUser(): Promise<UserDto | null> {
	const response = await fetch('/auth/me', {
		method: 'GET',
		credentials: 'same-origin',
	});

	if (response.status === 401) {
		return null;
	}

	if (!response.ok) {
		throw await parseApiError(response);
	}

	return response.json() as Promise<UserDto>;
}

export async function logoutUser(): Promise<void> {
	const response = await fetch('/auth/logout', {
		method: 'POST',
		credentials: 'same-origin',
	});

	if (!response.ok) {
		throw await parseApiError(response);
	}
}

export async function updateUserProfile(name: string, profileImageUrl: string): Promise<UserDto> {
	const response = await fetch('/auth/profile', {
		method: 'PUT',
		credentials: 'same-origin',
		headers: {
			'content-type': 'application/json',
		},
		body: JSON.stringify({
			name,
			profile_image_url: profileImageUrl,
		} satisfies UpdateUserProfileRequestDto),
	});

	if (!response.ok) {
		throw await parseApiError(response);
	}

	return response.json() as Promise<UserDto>;
}

export async function uploadProfileImage(image: Blob): Promise<string> {
	const formData = new FormData();
	formData.append('image', image, 'profile-image.webp');

	const response = await fetch('/auth/profile-image', {
		method: 'POST',
		credentials: 'same-origin',
		body: formData,
	});

	if (!response.ok) {
		throw await parseApiError(response);
	}

	const body = await response.json() as ProfileImageUploadDto;
	return body.profile_image_url;
}

export async function getUserSettings(): Promise<UserSettingsDto> {
	const response = await fetch('/auth/settings', {
		method: 'GET',
		credentials: 'same-origin',
	});

	if (!response.ok) {
		throw await parseApiError(response);
	}

	return response.json() as Promise<UserSettingsDto>;
}

export async function saveUserSettings(settings: UserSettingsDto): Promise<UserSettingsDto> {
	const response = await fetch('/auth/settings', {
		method: 'PUT',
		credentials: 'same-origin',
		headers: {
			'content-type': 'application/json',
		},
		body: JSON.stringify(settings),
	});

	if (!response.ok) {
		throw await parseApiError(response);
	}

	return response.json() as Promise<UserSettingsDto>;
}

export function toReadableErrorMessage(error: unknown): string {
	if (isApiError(error)) {
		const key = ERROR_KEY_TO_I18N[error.code];
		if (key) {
			return translate(key, error.params);
		}
	}

	if (isPasskeyCancelledError(error)) {
		return translate('PASSKEY_CANCELLED');
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

function isPasskeyCancelledError(error: unknown): boolean {
	return error instanceof DOMException && error.name === 'NotAllowedError';
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
