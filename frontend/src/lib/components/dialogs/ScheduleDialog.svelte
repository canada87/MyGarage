<script lang="ts">
	import { untrack } from 'svelte';
	import Modal from '../Modal.svelte';
	import Icon from '../Icon.svelte';
	import DueStateFields from './DueStateFields.svelte';
	import { api } from '#lib/api.ts';
	import { categories, categoryIcon, categoryLabel, parseIntOrNull, proposeFirstDue } from '#lib/format.ts';
	import { app, closeDialog, deleteWithUndo, refresh, toast } from '#lib/state.svelte.ts';
	import type { Category, DueMode, ScheduleView, TaskType } from '#lib/types.ts';

	const props: { vehicleId: number; schedule?: ScheduleView } = $props();
	// Each dialog is mounted fresh for a single request: read the props once.
	const { vehicleId, schedule } = untrack(() => props);

	const editing = !!schedule;
	const vehicle = $derived(app.vehicles.find((v) => v.id === vehicleId));

	let types = $state<TaskType[]>([]);
	let templateId = $state<string>('');
	let title = $state(schedule?.title ?? '');
	let category = $state<Category>(schedule?.category ?? 'manutenzione');
	let icon = $state(schedule?.icon ?? 'wrench');
	let recurrence = $state<'interval' | 'annual'>(schedule?.anchor_month ? 'annual' : 'interval');
	let months = $state(schedule?.interval_months != null ? String(schedule.interval_months) : '');
	let kmInterval = $state(schedule?.interval_km != null ? String(schedule.interval_km) : '');
	let anchorDay = $state(schedule?.anchor_day != null ? String(schedule.anchor_day) : '');
	let anchorMonth = $state(schedule?.anchor_month != null ? String(schedule.anchor_month) : '');
	let mode = $state<DueMode>(editing ? (schedule!.status === 'setup' ? 'last' : 'keep') : 'last');
	let date = $state('');
	let km = $state('');
	let warnDays = $state(String(schedule?.warn_days ?? ''));
	let warnKm = $state(String(schedule?.warn_km ?? ''));
	let emailEnabled = $state(schedule?.email_enabled ?? true);
	let active = $state(schedule?.active ?? true);
	let notes = $state(schedule?.notes ?? '');
	let error = $state('');
	let saving = $state(false);

	$effect(() => {
		if (!editing) api.get<TaskType[]>('/task-types').then((t) => (types = t));
	});

	function applyTemplate(id: string) {
		templateId = id;
		const t = types.find((x) => String(x.id) === id);
		if (!t) return;
		title = t.name;
		category = t.category;
		icon = t.icon;
		recurrence = t.anchor_month ? 'annual' : 'interval';
		months = t.interval_months != null ? String(t.interval_months) : '';
		kmInterval = t.interval_km != null ? String(t.interval_km) : '';
		anchorDay = t.anchor_day != null ? String(t.anchor_day) : '';
		anchorMonth = t.anchor_month != null ? String(t.anchor_month) : '';
		const proposal = proposeFirstDue(t, vehicle?.registration_date ?? null);
		if (proposal) {
			mode = 'next';
			date = proposal;
		}
	}

	async function save(e: SubmitEvent) {
		e.preventDefault();
		error = '';
		if (!title.trim()) return (error = 'Dai un nome alla scadenza');
		const annual = recurrence === 'annual';
		const body = {
			vehicle_id: vehicleId,
			task_type_id: schedule?.task_type_id ?? (templateId ? Number(templateId) : null),
			title,
			category,
			icon,
			interval_months: annual ? null : parseIntOrNull(months),
			interval_km: annual ? null : parseIntOrNull(kmInterval),
			anchor_month: annual ? parseIntOrNull(anchorMonth) : null,
			anchor_day: annual ? parseIntOrNull(anchorDay) : null,
			warn_days: parseIntOrNull(warnDays),
			warn_km: parseIntOrNull(warnKm),
			email_enabled: emailEnabled,
			active,
			notes,
			mode,
			date: date || null,
			km: parseIntOrNull(km)
		};
		if (!annual && body.interval_months == null && body.interval_km == null && mode === 'last') {
			return (error = 'Senza intervallo non posso calcolare la prossima scadenza: indica ogni quanto si fa, oppure la prossima scadenza');
		}
		if (annual && (body.anchor_day == null || body.anchor_month == null)) {
			return (error = 'Indica giorno e mese della scadenza annuale');
		}
		saving = true;
		try {
			if (editing) await api.put(`/schedules/${schedule!.id}`, body);
			else await api.post('/schedules', body);
			toast(editing ? 'Scadenza aggiornata' : 'Scadenza aggiunta');
			refresh();
			closeDialog();
		} catch (err) {
			error = (err as Error).message;
		} finally {
			saving = false;
		}
	}

	function remove() {
		deleteWithUndo('schedule', schedule!.id, 'Scadenza', `/schedules/${schedule!.id}`);
		closeDialog();
	}

	const monthNames = ['gen', 'feb', 'mar', 'apr', 'mag', 'giu', 'lug', 'ago', 'set', 'ott', 'nov', 'dic'];
</script>

<Modal title={editing ? `Modifica ${schedule!.title}` : 'Nuova scadenza'} onclose={closeDialog} wide>
	<form class="form" onsubmit={save} id="sched-form">
		{#if !editing}
			<div class="field">
				<label for="sc-tpl">Parti da un modello</label>
				<select id="sc-tpl" class="input" value={templateId} onchange={(e) => applyTemplate(e.currentTarget.value)}>
					<option value="">Personalizzata</option>
					{#each types as t (t.id)}<option value={t.id}>{t.name}</option>{/each}
				</select>
			</div>
		{/if}
		<div class="form-grid cols-2">
			<div class="field">
				<label for="sc-title">Nome <span class="req">*</span></label>
				<input id="sc-title" class="input" bind:value={title} required placeholder="Es. Tagliando" />
			</div>
			<div class="field">
				<label for="sc-cat">Categoria</label>
				<select
					id="sc-cat"
					class="input"
					bind:value={category}
					onchange={() => {
						if (!templateId && !editing) icon = categoryIcon[category];
					}}
				>
					{#each categories as c (c)}<option value={c}>{categoryLabel[c]}</option>{/each}
				</select>
			</div>
		</div>

		<div class="field">
			<span class="label">Ogni quanto</span>
			<div class="segmented" role="group" aria-label="Tipo di ricorrenza">
				<button type="button" aria-pressed={recurrence === 'interval'} onclick={() => (recurrence = 'interval')}>Intervallo</button>
				<button type="button" aria-pressed={recurrence === 'annual'} onclick={() => (recurrence = 'annual')}>Data fissa ogni anno</button>
			</div>
		</div>
		{#if recurrence === 'interval'}
			<div class="form-grid cols-2">
				<div class="field">
					<label for="sc-months">Ogni (mesi)</label>
					<div class="input-affix">
						<input id="sc-months" class="input num" type="text" inputmode="numeric" bind:value={months} placeholder="Es. 12" />
						<span class="affix">mesi</span>
					</div>
				</div>
				<div class="field">
					<label for="sc-km">Oppure ogni (km)</label>
					<div class="input-affix">
						<input id="sc-km" class="input num" type="text" inputmode="numeric" bind:value={kmInterval} placeholder="Es. 15000" />
						<span class="affix">km</span>
					</div>
				</div>
			</div>
			<span class="subtle">Puoi indicare mesi, km o entrambi: vale quello che arriva prima.</span>
		{:else}
			<div class="form-grid cols-2">
				<div class="field">
					<label for="sc-day">Entro il giorno</label>
					<input id="sc-day" class="input num" type="number" min="1" max="31" bind:value={anchorDay} />
				</div>
				<div class="field">
					<label for="sc-month">Mese</label>
					<select id="sc-month" class="input" bind:value={anchorMonth}>
						<option value="">—</option>
						{#each monthNames as m, i (m)}<option value={String(i + 1)}>{m}</option>{/each}
					</select>
				</div>
			</div>
		{/if}

		<DueStateFields
			bind:mode
			bind:date
			bind:km
			allowKeep={editing && schedule!.status !== 'setup'}
			usesDate={recurrence === 'annual' || !!parseIntOrNull(months)}
			usesKm={recurrence === 'interval' && !!parseIntOrNull(kmInterval)}
			annual={recurrence === 'annual'}
		/>

		<details class="more">
			<summary>Preavviso, email e altre opzioni</summary>
			<div class="form" style="margin-top:12px">
				<div class="form-grid cols-2">
					<div class="field">
						<label for="sc-wd">Avvisa giorni prima</label>
						<div class="input-affix">
							<input id="sc-wd" class="input num" type="text" inputmode="numeric" bind:value={warnDays} placeholder="30" />
							<span class="affix">gg</span>
						</div>
					</div>
					<div class="field">
						<label for="sc-wk">Avvisa km prima</label>
						<div class="input-affix">
							<input id="sc-wk" class="input num" type="text" inputmode="numeric" bind:value={warnKm} placeholder="2000" />
							<span class="affix">km</span>
						</div>
					</div>
				</div>
				<label class="check"><input type="checkbox" bind:checked={emailEnabled} />Invia promemoria via email</label>
				<label class="check"><input type="checkbox" bind:checked={active} />Scadenza attiva</label>
				<div class="field">
					<label for="sc-notes">Note</label>
					<textarea id="sc-notes" class="input" bind:value={notes} placeholder="Es. numero polizza, olio consigliato…"></textarea>
				</div>
			</div>
		</details>
		{#if error}<div class="alert danger" role="alert">{error}</div>{/if}
	</form>
	{#snippet footer()}
		<div class="form-actions">
			{#if editing}
				<button type="button" class="btn btn-danger" onclick={remove}><Icon name="trash" size={18} />Elimina</button>
				<span class="spacer"></span>
			{/if}
			<button type="button" class="btn" onclick={closeDialog}>Annulla</button>
			<button type="submit" form="sched-form" class="btn btn-primary" disabled={saving}>{saving ? 'Salvataggio…' : 'Salva'}</button>
		</div>
	{/snippet}
</Modal>

<style>
	.more summary {
		cursor: pointer;
		font-weight: 600;
		color: var(--primary);
		min-height: 44px;
		display: flex;
		align-items: center;
	}
</style>
