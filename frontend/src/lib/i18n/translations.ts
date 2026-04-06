// Generated from shared/i18n/*.yaml — do not edit manually

export const defaultLanguage = 'nl-NL';

export type Language = 'en-GB' | 'nl-NL';

export type I18nKey = 'APP_NAME' | 'DATABASE_ERROR' | 'EMAIL' | 'INVALID_URL' | 'PLAYER_NAME' | 'REGISTER' | 'REQUEST_FAILED' | 'REQUEST_TIMEOUT' | 'ROOM_NOT_FOUND' | 'SERVER_OFFLINE' | 'SIGN_IN' | 'SIGN_UP' | 'USER_ALREADY_EXISTS' | 'WELCOME';

export const translations: Record<Language, Record<I18nKey, string>> = {
	'en-GB': {
		'APP_NAME': 'Matter-rs Game Server',
		'DATABASE_ERROR': 'A database error occurred',
		'EMAIL': 'Email address',
		'INVALID_URL': 'Cannot build URL from: {host}, {port}, {path}',
		'PLAYER_NAME': 'Name',
		'REGISTER': 'Register',
		'REQUEST_FAILED': 'Request to {server} failed',
		'REQUEST_TIMEOUT': 'Request to {server} timed out',
		'ROOM_NOT_FOUND': 'Room {room_name} not found',
		'SERVER_OFFLINE': 'Could not connect to server {server}',
		'SIGN_IN': 'Sign in',
		'SIGN_UP': 'Sign up',
		'USER_ALREADY_EXISTS': 'User with email address {email} already exists',
		'WELCOME': 'Welcome',
	},
	'nl-NL': {
		'APP_NAME': 'Matter-rs Game Server',
		'DATABASE_ERROR': 'Er is een databasefout opgetreden',
		'EMAIL': 'E-mailadres',
		'INVALID_URL': 'Kan geen URL maken van: {host}, {port}, {path}',
		'PLAYER_NAME': 'Naam',
		'REGISTER': 'Registreren',
		'REQUEST_FAILED': 'Verzoek naar {server} is mislukt',
		'REQUEST_TIMEOUT': 'Verzoek naar {server} duurde te lang',
		'ROOM_NOT_FOUND': 'Kamer {room_name} niet gevonden',
		'SERVER_OFFLINE': 'Verbinding met de server {server} is mislukt',
		'SIGN_IN': 'Inloggen',
		'SIGN_UP': 'Registreren',
		'USER_ALREADY_EXISTS': 'Gebruiker met e-mailadres {email} bestaat al',
		'WELCOME': 'Welkom',
	},
};

export function t(key: I18nKey, lang: Language = defaultLanguage, params?: Record<string, string>): string {
	let text = translations[lang][key];
	if (params) {
		for (const [k, v] of Object.entries(params)) {
			text = text.replace(`{${k}}`, v);
		}
	}
	return text;
}
