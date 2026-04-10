// Generated from shared/i18n/*.yaml — do not edit manually

export const defaultLanguage = 'nl-NL';

export type Language = 'en-GB' | 'nl-NL';

export type I18nKey = 'APP_NAME' | 'AUTHENTICATION_FAILED' | 'AUTH_SUBTITLE' | 'AUTH_SUCCESS' | 'CANCEL' | 'DATABASE_ERROR' | 'EMAIL' | 'HOME_SUBTITLE' | 'INVALID_URL' | 'LOBBY' | 'LOBBY_EMPTY_GAMES' | 'LOBBY_EMPTY_PLAYERS' | 'LOBBY_GAMES' | 'LOBBY_GAMES_SUBTITLE' | 'LOBBY_ONLINE' | 'LOBBY_PLAYERS' | 'LOBBY_PLAYERS_SUBTITLE' | 'LOGIN' | 'LOGIN_SUBTITLE' | 'LOGIN_WITH_ONE_TIME_CODE' | 'LOGOUT' | 'PASSKEY_CANCELLED' | 'PASSKEY_UNAVAILABLE' | 'PLAYER_NAME' | 'REGISTER' | 'REQUEST_FAILED' | 'REQUEST_TIMEOUT' | 'ROOM_NOT_FOUND' | 'SERVER_OFFLINE' | 'SIGN_IN' | 'SIGN_UP' | 'START_BATTLE' | 'USER_ALREADY_EXISTS' | 'USER_NOT_FOUND' | 'WELCOME';

export const translations: Record<Language, Record<I18nKey, string>> = {
	'en-GB': {
		'APP_NAME': 'Battle Control',
		'AUTHENTICATION_FAILED': 'Verification failed',
		'AUTH_SUBTITLE': 'Register a pilot to continue',
		'AUTH_SUCCESS': 'Pilot ready',
		'CANCEL': 'Cancel',
		'DATABASE_ERROR': 'A database error occurred',
		'EMAIL': 'Email address',
		'HOME_SUBTITLE': 'In a universe where the Androsynth, VUX, Thraddash, Ur-Quan, Melnorme, Humans, and other races fight for Star Control, you and your mates find victory through:',
		'INVALID_URL': 'Cannot build URL from: {host}, {port}, {path}',
		'LOBBY': 'Lobby',
		'LOBBY_EMPTY_GAMES': 'There are no open games yet.',
		'LOBBY_EMPTY_PLAYERS': 'There are no other pilots online yet.',
		'LOBBY_GAMES': 'Games',
		'LOBBY_GAMES_SUBTITLE': 'Open battles and waiting commanders',
		'LOBBY_ONLINE': 'Online',
		'LOBBY_PLAYERS': 'Players',
		'LOBBY_PLAYERS_SUBTITLE': 'Pilots currently present in the star system',
		'LOGIN': 'Log in',
		'LOGIN_SUBTITLE': 'Resume with an existing pilot',
		'LOGIN_WITH_ONE_TIME_CODE': 'Log in with one-time code',
		'LOGOUT': 'Log out',
		'PASSKEY_CANCELLED': 'The passkey action was cancelled or denied',
		'PASSKEY_UNAVAILABLE': 'Passkeys are not supported in this browser',
		'PLAYER_NAME': 'Name',
		'REGISTER': 'Register',
		'REQUEST_FAILED': 'Request to {server} failed',
		'REQUEST_TIMEOUT': 'Request to {server} timed out',
		'ROOM_NOT_FOUND': 'Room {room_name} not found',
		'SERVER_OFFLINE': 'Could not connect to server {server}',
		'SIGN_IN': 'Sign in',
		'SIGN_UP': 'Sign up',
		'START_BATTLE': 'Start battle',
		'USER_ALREADY_EXISTS': 'User {email} already exists',
		'USER_NOT_FOUND': 'No pilot found for {email}',
		'WELCOME': 'Welcome',
	},
	'nl-NL': {
		'APP_NAME': 'Battle Control',
		'AUTHENTICATION_FAILED': 'Verificatie is mislukt',
		'AUTH_SUBTITLE': 'Registreer een piloot om door te gaan',
		'AUTH_SUCCESS': 'Piloot gereed',
		'CANCEL': 'Annuleren',
		'DATABASE_ERROR': 'Er is een databasefout opgetreden',
		'EMAIL': 'E-mailadres',
		'HOME_SUBTITLE': 'In ons universum, waar de Androsynth, VUX, Thraddash, Ur-Quan, Melnorme, Mensen en andere rassen strijden om Star Control, ben jij de piloot die samen met je maten telkens weer de overwinning behaalt door:',
		'INVALID_URL': 'Kan geen URL maken van: {host}, {port}, {path}',
		'LOBBY': 'Lobby',
		'LOBBY_EMPTY_GAMES': 'Er zijn nog geen open games.',
		'LOBBY_EMPTY_PLAYERS': 'Er zijn nog geen andere piloten online.',
		'LOBBY_GAMES': 'Games',
		'LOBBY_GAMES_SUBTITLE': 'Open gevechten en wachtende commandanten',
		'LOBBY_ONLINE': 'Online',
		'LOBBY_PLAYERS': 'Spelers',
		'LOBBY_PLAYERS_SUBTITLE': 'Piloten die nu in het sterrenstelsel aanwezig zijn',
		'LOGIN': 'Inloggen',
		'LOGIN_SUBTITLE': 'Ga verder met een bestaande piloot',
		'LOGIN_WITH_ONE_TIME_CODE': 'Inloggen met eenmalige code',
		'LOGOUT': 'Uitloggen',
		'PASSKEY_CANCELLED': 'De passkey-actie is geannuleerd of geweigerd',
		'PASSKEY_UNAVAILABLE': 'Passkeys worden niet ondersteund in deze browser',
		'PLAYER_NAME': 'Naam',
		'REGISTER': 'Registreren',
		'REQUEST_FAILED': 'Verzoek naar {server} is mislukt',
		'REQUEST_TIMEOUT': 'Verzoek naar {server} duurde te lang',
		'ROOM_NOT_FOUND': 'Kamer {room_name} niet gevonden',
		'SERVER_OFFLINE': 'Verbinding met de server {server} is mislukt',
		'SIGN_IN': 'Inloggen',
		'SIGN_UP': 'Registreren',
		'START_BATTLE': 'Start gevecht',
		'USER_ALREADY_EXISTS': 'Gebruiker {email} bestaat al',
		'USER_NOT_FOUND': 'Geen piloot gevonden voor {email}',
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
