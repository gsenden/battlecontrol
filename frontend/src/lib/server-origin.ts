declare const __SERVER_HOST__: string;
declare const __SERVER_PORT__: string;

export function serverOrigin(): string {
	if (!import.meta.env.DEV) {
		return window.location.origin;
	}

	const host = window.location.hostname;
	return `http://${host}:${__SERVER_PORT__}`;
}

export function serverUrl(path: string): string {
	return `${serverOrigin()}${path}`;
}
