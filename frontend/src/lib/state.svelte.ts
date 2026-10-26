import { SvelteSet } from 'svelte/reactivity';
import { api } from './api';
import type { CarEvent, Contact, DocumentFile, ScheduleView, TyreSetView, VehicleSummary } from './types';

/** Shared app state: bump `version` after any write so pages reload their data. */
export const app = $state({
	version: 0,
	vehicles: [] as VehicleSummary[],
	vehiclesLoaded: false
});

export async function loadVehicles() {
	try {
		app.vehicles = await api.get<VehicleSummary[]>('/vehicles');
	} finally {
		app.vehiclesLoaded = true;
	}
}

export function refresh() {
	app.version++;
	void loadVehicles();
}

/* ---------- Toasts ---------- */

export interface Toast {
	id: number;
	message: string;
	kind: 'info' | 'error' | 'success';
	action?: { label: string; run: () => void };
}

export const toasts = $state<Toast[]>([]);
let toastSeq = 0;

export function toast(message: string, opts: { kind?: Toast['kind']; action?: Toast['action']; duration?: number } = {}) {
	const id = ++toastSeq;
	toasts.push({ id, message, kind: opts.kind ?? 'success', action: opts.action });
	setTimeout(() => dismissToast(id), opts.duration ?? (opts.action ? 5000 : 3500));
	return id;
}

export function dismissToast(id: number) {
	const i = toasts.findIndex((t) => t.id === id);
	if (i >= 0) toasts.splice(i, 1);
}

export function toastError(e: unknown) {
	toast(e instanceof Error ? e.message : String(e), { kind: 'error', duration: 6000 });
}

/* ---------- Delete with undo ---------- */

/** Keys ("event:12") of items hidden while their deletion can still be undone. */
export const pendingDeletes = new SvelteSet<string>();

export function isPending(kind: string, id: number) {
	return pendingDeletes.has(`${kind}:${id}`);
}

/**
 * Hides the item at once and deletes it after a few seconds unless the user taps "Annulla".
 */
export function deleteWithUndo(kind: string, id: number, label: string, path: string) {
	const key = `${kind}:${id}`;
	pendingDeletes.add(key);
	let undone = false;
	const tid = toast(`${label} eliminato`, {
		kind: 'info',
		action: {
			label: 'Annulla',
			run: () => {
				undone = true;
				pendingDeletes.delete(key);
				dismissToast(tid);
			}
		}
	});
	setTimeout(async () => {
		if (undone) return;
		try {
			await api.del(path);
		} catch (e) {
			toastError(e);
		} finally {
			pendingDeletes.delete(key);
			refresh();
		}
	}, 5000);
}

/* ---------- Dialogs ---------- */

export type DialogRequest =
	| { kind: 'km'; vehicleId?: number }
	| { kind: 'event'; vehicleId?: number; schedule?: ScheduleView; event?: CarEvent }
	| { kind: 'schedule'; vehicleId: number; schedule?: ScheduleView }
	| { kind: 'document'; vehicleId?: number; eventId?: number; doc?: DocumentFile }
	| { kind: 'contact'; vehicleId?: number | null; contact?: Contact }
	| { kind: 'tyreSet'; vehicleId: number; set?: TyreSetView }
	| { kind: 'mount'; set: TyreSetView; action: 'mount' | 'unmount' }
	| { kind: 'actions'; vehicleId?: number }
	| {
			kind: 'confirm';
			title: string;
			message: string;
			confirmLabel: string;
			danger?: boolean;
			onConfirm: () => Promise<void> | void;
	  };

export const dialog = $state<{ current: DialogRequest | null }>({ current: null });

export function openDialog(d: DialogRequest) {
	dialog.current = d;
}

export function closeDialog() {
	dialog.current = null;
}

export function confirmDialog(opts: Omit<Extract<DialogRequest, { kind: 'confirm' }>, 'kind'>) {
	openDialog({ kind: 'confirm', ...opts });
}
