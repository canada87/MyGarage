import type { Category, ScheduleView, Status } from './types';

const nf = new Intl.NumberFormat('it-IT');
const eur = new Intl.NumberFormat('it-IT', { style: 'currency', currency: 'EUR', maximumFractionDigits: 2 });
const eurRound = new Intl.NumberFormat('it-IT', { style: 'currency', currency: 'EUR', maximumFractionDigits: 0 });

/** Parses a YYYY-MM-DD string as a local date (not UTC). */
export function parseDate(s: string): Date {
	const [y, m, d] = s.split('-').map(Number);
	return new Date(y, m - 1, d);
}

export function todayIso(): string {
	const d = new Date();
	return toIso(d);
}

export function toIso(d: Date): string {
	const p = (n: number) => String(n).padStart(2, '0');
	return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}`;
}

export function addMonthsIso(iso: string, months: number): string {
	const d = parseDate(iso);
	const day = d.getDate();
	d.setDate(1);
	d.setMonth(d.getMonth() + months);
	const last = new Date(d.getFullYear(), d.getMonth() + 1, 0).getDate();
	d.setDate(Math.min(day, last));
	return toIso(d);
}

export const fmtKm = (km: number | null | undefined) => (km == null ? '—' : `${nf.format(km)} km`);
export const fmtNum = (n: number) => nf.format(n);
export const fmtEuro = (cents: number | null | undefined) => (cents == null ? '—' : eur.format(cents / 100));
export const fmtEuroRound = (cents: number) => eurRound.format(cents / 100);

export function fmtDate(iso: string | null | undefined): string {
	if (!iso) return '—';
	return parseDate(iso).toLocaleDateString('it-IT', { day: 'numeric', month: 'short', year: 'numeric' });
}

export function fmtDateLong(iso: string): string {
	const s = parseDate(iso).toLocaleDateString('it-IT', { weekday: 'long', day: 'numeric', month: 'long', year: 'numeric' });
	return s.charAt(0).toUpperCase() + s.slice(1);
}

export function fmtDayMonth(iso: string): string {
	return parseDate(iso).toLocaleDateString('it-IT', { weekday: 'long', day: 'numeric', month: 'long' });
}

export function daysAgo(iso: string | null): string {
	if (!iso) return 'mai';
	const diff = Math.round((parseDate(todayIso()).getTime() - parseDate(iso).getTime()) / 86400000);
	if (diff <= 0) return 'oggi';
	if (diff === 1) return 'ieri';
	if (diff < 60) return `${diff} giorni fa`;
	const months = Math.round(diff / 30);
	return months < 18 ? `${months} mesi fa` : `${Math.round(diff / 365)} anni fa`;
}

export function fmtBytes(n: number): string {
	if (n < 1024) return `${n} B`;
	if (n < 1024 * 1024) return `${Math.round(n / 1024)} KB`;
	return `${(n / 1024 / 1024).toFixed(1).replace('.', ',')} MB`;
}

export const statusLabel: Record<Status, string> = {
	overdue: 'Scaduta',
	soon: 'In arrivo',
	ok: 'In regola',
	setup: 'Da completare'
};

export const categoryLabel: Record<Category, string> = {
	assicurazione: 'Assicurazione',
	bollo: 'Bollo',
	revisione: 'Revisione',
	manutenzione: 'Manutenzione',
	gomme: 'Gomme',
	abbonamento: 'Abbonamenti',
	altro: 'Altro'
};

export const categoryIcon: Record<Category, string> = {
	assicurazione: 'shield',
	bollo: 'receipt',
	revisione: 'clipboard',
	manutenzione: 'wrench',
	gomme: 'tyre',
	abbonamento: 'truck',
	altro: 'wrench'
};

export const categories = Object.keys(categoryLabel) as Category[];

export const contactCategories: Record<string, string> = {
	assicurazione: 'Assicurazione',
	soccorso: 'Soccorso stradale',
	officina: 'Officina',
	gommista: 'Gommista',
	carrozzeria: 'Carrozzeria',
	concessionario: 'Concessionario',
	altro: 'Altro'
};

export const documentKinds: Record<string, string> = {
	libretto: 'Libretto',
	fattura: 'Fattura',
	polizza: 'Polizza',
	foto: 'Foto',
	altro: 'Altro'
};

/** Short badge text for a deadline, e.g. "Tra 5 giorni" or "Superato di 420 km". */
export function dueBadge(s: ScheduleView): string {
	if (s.status === 'setup') return 'Da completare';
	const kmOver = s.km_left != null && s.km_left <= 0;
	const dateOver = s.days_left != null && s.days_left < 0;
	if (s.status === 'overdue') {
		if (kmOver && (!dateOver || s.days_left == null)) return `Superato di ${fmtKm(-s.km_left!)}`;
		if (dateOver) return s.days_left === -1 ? 'Scaduta ieri' : `Scaduta da ${-s.days_left!} giorni`;
	}
	// Show whichever limit is closer to triggering the warning.
	const kmClose = s.km_left != null && s.km_left <= s.warn_km;
	if (s.days_left != null && !(kmClose && (s.days_left > s.warn_days))) {
		if (s.days_left === 0) return 'Scade oggi';
		if (s.days_left === 1) return 'Domani';
		if (s.days_left < 60) return `Tra ${s.days_left} giorni`;
		const months = Math.round(s.days_left / 30.4);
		return months < 24 ? `Tra ${months} mesi` : `Tra ${Math.round(s.days_left / 365)} anni`;
	}
	if (s.km_left != null) return `Mancano ${fmtKm(s.km_left)}`;
	return statusLabel[s.status];
}

/** Longer description of the target, e.g. "Scade sabato 31 ottobre · o a 30.000 km". */
export function dueDetail(s: ScheduleView): string {
	const parts: string[] = [];
	if (s.next_due_date) parts.push(`Entro ${fmtDate(s.next_due_date)}`);
	if (s.next_due_km != null) {
		const now = s.current_km != null ? ` · ora ${fmtKm(s.current_km)}` : '';
		parts.push(`${parts.length ? 'o a' : 'A'} ${fmtKm(s.next_due_km)}${now}`);
	}
	if (!parts.length) return 'Indica quando è stata fatta l’ultima volta';
	return parts.join(' ');
}

export function intervalText(s: {
	interval_months: number | null;
	interval_km: number | null;
	anchor_month: number | null;
	anchor_day: number | null;
}): string {
	if (s.anchor_month && s.anchor_day) {
		return `Ogni anno entro il ${String(s.anchor_day).padStart(2, '0')}/${String(s.anchor_month).padStart(2, '0')}`;
	}
	const parts: string[] = [];
	if (s.interval_months) {
		parts.push(
			s.interval_months % 12 === 0
				? s.interval_months === 12
					? 'ogni anno'
					: `ogni ${s.interval_months / 12} anni`
				: `ogni ${s.interval_months} mesi`
		);
	}
	if (s.interval_km) parts.push(`${parts.length ? 'o ' : 'ogni '}${fmtKm(s.interval_km)}`);
	if (!parts.length) return 'Una tantum';
	const t = parts.join(' ');
	return t.charAt(0).toUpperCase() + t.slice(1);
}

/** Share of the interval already elapsed (0–1), for progress bars. */
export function progressTime(s: ScheduleView): number | null {
	if (!s.next_due_date || s.days_left == null) return null;
	let total: number | null = null;
	if (s.last_done_date) {
		total = (parseDate(s.next_due_date).getTime() - parseDate(s.last_done_date).getTime()) / 86400000;
	} else if (s.interval_months) {
		total = s.interval_months * 30.4;
	} else if (s.anchor_month) {
		total = 365;
	}
	if (!total || total <= 0) return null;
	return Math.min(1, Math.max(0, 1 - s.days_left / total));
}

export function progressKm(s: ScheduleView): number | null {
	if (s.next_due_km == null || s.km_left == null) return null;
	const total = s.last_done_km != null ? s.next_due_km - s.last_done_km : s.interval_km;
	if (!total || total <= 0) return null;
	return Math.min(1, Math.max(0, 1 - s.km_left / total));
}

/** Converts an input string like "12,50" into cents. */
export function parseEuro(v: string | number | null | undefined): number | null {
	if (v === null || v === undefined || v === '') return null;
	const n = typeof v === 'number' ? v : Number(String(v).replace(/\./g, '').replace(',', '.'));
	return Number.isFinite(n) ? Math.round(n * 100) : null;
}

export function centsToInput(c: number | null): string {
	return c == null ? '' : (c / 100).toFixed(2).replace('.', ',');
}

export function parseIntOrNull(v: string | number | null | undefined): number | null {
	if (v === null || v === undefined || v === '') return null;
	const n = typeof v === 'number' ? v : Number(String(v).replace(/[.\s]/g, ''));
	return Number.isFinite(n) ? Math.round(n) : null;
}

/**
 * Proposes the next due date of tasks counted from registration (e.g. revisione: after 4 years,
 * then every 2). Returns null when the task or the registration date doesn't allow it.
 */
export function proposeFirstDue(
	t: { first_interval_months: number | null; interval_months: number | null },
	registrationDate: string | null
): string | null {
	if (!t.first_interval_months || !registrationDate) return null;
	const today = todayIso();
	let due = addMonthsIso(registrationDate, t.first_interval_months);
	let guard = 0;
	while (due < today && t.interval_months && guard++ < 50) due = addMonthsIso(due, t.interval_months);
	return due;
}

export const vehicleColors =['#2563EB', '#E11D48', '#059669', '#D97706', '#7C3AED', '#0891B2', '#475569', '#DB2777'];
