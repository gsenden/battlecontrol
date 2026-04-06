import { readFileSync, readdirSync, writeFileSync, mkdirSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { parse } from 'yaml';

const __dirname = dirname(fileURLToPath(import.meta.url));
const i18nDir = join(__dirname, '..', '..', 'shared', 'i18n');
const outFile = join(__dirname, '..', 'src', 'lib', 'i18n', 'translations.ts');

export function generateI18n() {
	const config = parse(readFileSync(join(i18nDir, 'config.yaml'), 'utf-8'));
	const defaultLanguage = config.default;

	const languages = {};
	for (const file of readdirSync(i18nDir).sort()) {
		if (!file.endsWith('.yaml') || file === 'config.yaml') continue;
		const lang = file.replace('.yaml', '');
		languages[lang] = parse(readFileSync(join(i18nDir, file), 'utf-8'));
	}

	const keys = Object.keys(Object.values(languages)[0]).sort();
	const langCodes = Object.keys(languages).sort();

	let output = '// Generated from shared/i18n/*.yaml — do not edit manually\n\n';

	output += `export const defaultLanguage = '${defaultLanguage}';\n\n`;
	output += `export type Language = ${langCodes.map((lang) => `'${lang}'`).join(' | ')};\n\n`;
	output += `export type I18nKey = ${keys.map((key) => `'${key}'`).join(' | ')};\n\n`;

	output += 'export const translations: Record<Language, Record<I18nKey, string>> = {\n';
	for (const [lang, strings] of Object.entries(languages)) {
		output += `\t'${lang}': {\n`;
		for (const key of keys) {
			const value = strings[key].replace(/'/g, "\\'");
			output += `\t\t'${key}': '${value}',\n`;
		}
		output += '\t},\n';
	}
	output += '};\n\n';

	output += `export function t(key: I18nKey, lang: Language = defaultLanguage, params?: Record<string, string>): string {\n`;
	output += `\tlet text = translations[lang][key];\n`;
	output += `\tif (params) {\n`;
	output += `\t\tfor (const [k, v] of Object.entries(params)) {\n`;
	output += `\t\t\ttext = text.replace(\`{\${k}}\`, v);\n`;
	output += `\t\t}\n`;
	output += `\t}\n`;
	output += `\treturn text;\n`;
	output += `}\n`;

	mkdirSync(dirname(outFile), { recursive: true });
	writeFileSync(outFile, output);

	return {
		defaultLanguage,
		keys,
		langCodes,
		outFile,
	};
}

export function getI18nWatchFiles() {
	return [
		join(i18nDir, 'config.yaml'),
		...readdirSync(i18nDir)
			.filter((file) => file.endsWith('.yaml') && file !== 'config.yaml')
			.map((file) => join(i18nDir, file)),
	];
}
