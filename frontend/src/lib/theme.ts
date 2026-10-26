export type Theme = 'system' | 'light' | 'dark';
const KEY = 'mygarage-theme';

export function getTheme(): Theme {
	try {
		const t = localStorage.getItem(KEY);
		if (t === 'light' || t === 'dark') return t;
	} catch {
		/* storage unavailable */
	}
	return 'system';
}

export function setTheme(t: Theme) {
	try {
		if (t === 'system') localStorage.removeItem(KEY);
		else localStorage.setItem(KEY, t);
	} catch {
		/* storage unavailable */
	}
	if (t === 'system') delete document.documentElement.dataset.theme;
	else document.documentElement.dataset.theme = t;
}

export function isDark(t: Theme): boolean {
	return t === 'dark' || (t === 'system' && matchMedia('(prefers-color-scheme: dark)').matches);
}
