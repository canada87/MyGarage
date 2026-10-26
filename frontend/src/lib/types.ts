export type Status = 'overdue' | 'soon' | 'setup' | 'ok';
export type DueMode = 'last' | 'next' | 'unknown' | 'keep';

export type Category = 'assicurazione' | 'bollo' | 'revisione' | 'manutenzione' | 'gomme' | 'abbonamento' | 'altro';

export interface Vehicle {
	id: number;
	name: string;
	make: string | null;
	model: string | null;
	plate: string | null;
	vin: string | null;
	year: number | null;
	registration_date: string | null;
	purchase_date: string | null;
	purchased_used: boolean;
	initial_km: number;
	color: string;
	notes: string | null;
	archived: boolean;
}

export interface Counts {
	overdue: number;
	soon: number;
	ok: number;
	setup: number;
}

export interface TaskType {
	id: number;
	name: string;
	category: Category;
	icon: string;
	interval_months: number | null;
	interval_km: number | null;
	first_interval_months: number | null;
	anchor_month: number | null;
	anchor_day: number | null;
	suggested: boolean;
	sort_order: number;
}

export interface Schedule {
	id: number;
	vehicle_id: number;
	task_type_id: number | null;
	title: string;
	category: Category;
	icon: string;
	interval_months: number | null;
	interval_km: number | null;
	anchor_month: number | null;
	anchor_day: number | null;
	baseline_date: string | null;
	baseline_km: number | null;
	manual_due_date: string | null;
	manual_due_km: number | null;
	manual_set_on: string | null;
	last_done_date: string | null;
	last_done_km: number | null;
	next_due_date: string | null;
	next_due_km: number | null;
	warn_days: number;
	warn_km: number;
	email_enabled: boolean;
	active: boolean;
	notes: string | null;
}

export interface ScheduleView extends Schedule {
	vehicle_name: string;
	vehicle_color: string;
	current_km: number | null;
	current_km_date: string | null;
	status: Status;
	days_left: number | null;
	km_left: number | null;
	km_stale: boolean;
}

export interface VehicleSummary extends Vehicle {
	current_km: number | null;
	current_km_date: string | null;
	counts: Counts;
	next: ScheduleView | null;
}

export interface VehicleDetail extends Vehicle {
	current_km: number | null;
	current_km_date: string | null;
	counts: Counts;
	schedules: ScheduleView[];
}

export interface CarEvent {
	id: number;
	vehicle_id: number;
	schedule_id: number | null;
	title: string;
	category: Category;
	date: string;
	km: number | null;
	cost_cents: number | null;
	provider: string | null;
	notes: string | null;
}

export interface OdometerReading {
	id: number;
	vehicle_id: number;
	event_id: number | null;
	date: string;
	km: number;
	source: string;
	note: string | null;
}

export interface DocumentFile {
	id: number;
	vehicle_id: number | null;
	event_id: number | null;
	title: string;
	kind: 'libretto' | 'fattura' | 'polizza' | 'foto' | 'altro';
	stored_name: string;
	original_name: string;
	mime: string;
	size: number;
	uploaded_at: string;
}

export interface Contact {
	id: number;
	vehicle_id: number | null;
	name: string;
	category: string;
	phone: string | null;
	email: string | null;
	reference: string | null;
	notes: string | null;
}

export interface TyreSet {
	id: number;
	vehicle_id: number;
	name: string;
	season: string | null;
	brand: string | null;
	model: string | null;
	size: string | null;
	dot: string | null;
	purchase_date: string | null;
	initial_km: number | null;
	notes: string | null;
	retired: boolean;
}

export interface TyreMountView {
	id: number;
	tyre_set_id: number;
	mounted_on: string;
	mounted_km: number | null;
	removed_on: string | null;
	removed_km: number | null;
	km_start: number | null;
	km_end: number | null;
	distance: number | null;
	estimated: boolean;
}

export interface TyreSetView extends TyreSet {
	mounted: boolean;
	total_km: number;
	estimated: boolean;
	mounts: TyreMountView[];
}

export interface CategoryTotal {
	category: Category;
	total_cents: number;
}

export interface Dashboard {
	today: string;
	counts: Counts;
	upcoming: ScheduleView[];
	vehicles: VehicleSummary[];
	year: number;
	year_total_cents: number;
	year_by_category: CategoryTotal[];
}

export interface Expenses {
	year: number;
	years: number[];
	total_cents: number;
	by_category: CategoryTotal[];
	by_vehicle: { vehicle_id: number; name: string; color: string; total_cents: number }[];
	events: CarEvent[];
}

export interface Settings {
	email_enabled: boolean;
	smtp_host: string;
	smtp_port: number;
	smtp_username: string;
	email_from: string;
	email_to: string;
	send_hour: number;
	default_warn_days: number;
	default_warn_km: number;
	password_set: boolean;
}
