import { readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.resolve(__dirname, '..');
const version = readFileSync(path.join(rootDir, 'VERSION'), 'utf-8').trim();

syncPackageJson(path.join(rootDir, 'package.json'));
syncPackageJson(path.join(rootDir, 'frontend/package.json'));
syncRootPackageLock(path.join(rootDir, 'package-lock.json'));
syncFrontendPackageLock(path.join(rootDir, 'frontend/package-lock.json'));
syncWorkspaceVersion(path.join(rootDir, 'Cargo.toml'));

console.log(`Synchronized project version to ${version}`);

function syncPackageJson(filePath) {
	const content = JSON.parse(readFileSync(filePath, 'utf-8'));
	content.version = version;
	writeJson(filePath, content);
}

function syncRootPackageLock(filePath) {
	const content = JSON.parse(readFileSync(filePath, 'utf-8'));
	content.version = version;
	if (content.packages?.['']) {
		content.packages[''].version = version;
	}
	writeJson(filePath, content);
}

function syncFrontendPackageLock(filePath) {
	const content = JSON.parse(readFileSync(filePath, 'utf-8'));
	content.version = version;
	if (content.packages?.['']) {
		content.packages[''].version = version;
	}
	if (content.packages?.['../pkg']) {
		content.packages['../pkg'].version = version;
	}
	writeJson(filePath, content);
}

function syncWorkspaceVersion(filePath) {
	const content = readFileSync(filePath, 'utf-8');
	const next = content.replace(
		/version = "\d+\.\d+\.\d+"/,
		`version = "${version}"`,
	);
	writeFileSync(filePath, next);
}

function writeJson(filePath, content) {
	writeFileSync(filePath, `${JSON.stringify(content, null, '\t')}\n`);
}
