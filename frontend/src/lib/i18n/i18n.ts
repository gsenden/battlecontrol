import { browser } from '$app/environment';
import { get, writable } from 'svelte/store';
import { defaultLanguage, t, type I18nKey, type Language } from '$lib/i18n/translations.js';

const LANGUAGE_STORAGE_KEY = 'battlecontrol-language';
const SUPPORTED_LANGUAGES = new Set<Language>([
	'nl-NL',
	'en-GB',
	'de-DE',
	'it-IT',
	'pl-PL',
	'fr-FR',
	'es-ES',
	'pt-PT',
]);

export const languageOptions: Array<{ code: Language; label: string }> = [
	{ code: 'nl-NL', label: 'NL' },
	{ code: 'en-GB', label: 'EN' },
	{ code: 'de-DE', label: 'DE' },
	{ code: 'it-IT', label: 'IT' },
	{ code: 'pl-PL', label: 'PL' },
	{ code: 'fr-FR', label: 'FR' },
	{ code: 'es-ES', label: 'ES' },
	{ code: 'pt-PT', label: 'PT' },
];

function loadInitialLanguage(): Language {
	if (!browser) {
		return defaultLanguage;
	}

	const stored = localStorage.getItem(LANGUAGE_STORAGE_KEY) as Language | null;
	return stored && SUPPORTED_LANGUAGES.has(stored) ? stored : defaultLanguage;
}

export const currentLanguage = writable<Language>(loadInitialLanguage());

export function setLanguage(language: Language): void {
	currentLanguage.set(language);
	if (browser) {
		localStorage.setItem(LANGUAGE_STORAGE_KEY, language);
	}
}

export function translate(key: I18nKey, params?: Record<string, string>): string {
	return t(key, get(currentLanguage), params);
}
