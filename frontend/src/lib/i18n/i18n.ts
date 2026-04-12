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
	'ja-JP',
	'ar-SA',
	'uk-UA',
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
	{ code: 'ja-JP', label: 'JA' },
	{ code: 'ar-SA', label: 'AR' },
	{ code: 'uk-UA', label: 'UK' },
];

const LANGUAGE_FALLBACKS: Record<string, Language> = {
	nl: 'nl-NL',
	en: 'en-GB',
	de: 'de-DE',
	it: 'it-IT',
	pl: 'pl-PL',
	fr: 'fr-FR',
	es: 'es-ES',
	pt: 'pt-PT',
	ja: 'ja-JP',
	ar: 'ar-SA',
	uk: 'uk-UA',
};

function matchSupportedLanguage(locale: string | null | undefined): Language | null {
	if (!locale) {
		return null;
	}

	const normalizedLocale = locale.trim();
	if (SUPPORTED_LANGUAGES.has(normalizedLocale as Language)) {
		return normalizedLocale as Language;
	}

	const [languageCode] = normalizedLocale.split('-');
	return LANGUAGE_FALLBACKS[languageCode.toLowerCase()] ?? null;
}

function loadInitialLanguage(): Language {
	if (!browser) {
		return defaultLanguage;
	}

	const stored = localStorage.getItem(LANGUAGE_STORAGE_KEY) as Language | null;
	if (stored && SUPPORTED_LANGUAGES.has(stored)) {
		return stored;
	}

	for (const locale of navigator.languages) {
		const matched = matchSupportedLanguage(locale);
		if (matched) {
			return matched;
		}
	}

	return matchSupportedLanguage(navigator.language) ?? defaultLanguage;
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
