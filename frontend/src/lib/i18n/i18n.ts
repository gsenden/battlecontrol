import { browser } from '$app/environment';
import { get, writable } from 'svelte/store';
import { defaultLanguage, t, type I18nKey, type Language } from '$lib/i18n/translations.js';

const LANGUAGE_STORAGE_KEY = 'battlecontrol-language';

function loadInitialLanguage(): Language {
	if (!browser) {
		return defaultLanguage;
	}

	const stored = localStorage.getItem(LANGUAGE_STORAGE_KEY);
	return stored === 'en-GB' || stored === 'nl-NL' ? stored : defaultLanguage;
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
