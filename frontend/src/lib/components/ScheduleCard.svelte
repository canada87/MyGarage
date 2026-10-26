<script lang="ts">
	import Icon from './Icon.svelte';
	import StatusBadge from './StatusBadge.svelte';
	import { dueBadge, fmtDate, fmtKm, intervalText, progressKm, progressTime } from '#lib/format.ts';
	import { openDialog } from '#lib/state.svelte.ts';
	import type { ScheduleView } from '#lib/types.ts';

	let { s }: { s: ScheduleView } = $props();

	const pt = $derived(progressTime(s));
	const pk = $derived(progressKm(s));
	const timeStatus = $derived(
		s.days_left == null ? 'ok' : s.days_left < 0 ? 'overdue' : s.days_left <= s.warn_days ? 'soon' : 'ok'
	);
	const kmStatus = $derived(
		s.km_left == null ? 'ok' : s.km_left <= 0 ? 'overdue' : s.km_left <= s.warn_km ? 'soon' : 'ok'
	);
</script>

<article class="card sched" id="s{s.id}" class:inactive={!s.active}>
	<div class="head">
		<span class="tile-ico lg"><Icon name={s.icon} /></span>
		<div class="ttl">
			<h3>{s.title}</h3>
			<div class="subtle">{intervalText(s)}</div>
		</div>
		{#if s.active}
			<StatusBadge status={s.status} text={dueBadge(s)} />
		{:else}
			<span class="badge neutral">Disattivata</span>
		{/if}
	</div>

	{#if s.status === 'setup'}
		<p class="subtle setup">Non so ancora quando scade: indica quando è stata fatta l’ultima volta o la prossima scadenza.</p>
	{:else}
		<div class="progress">
			{#if s.next_due_km != null}
				<div class="prow">
					<span>Km</span><span class="num">{fmtKm(s.current_km)} / {fmtKm(s.next_due_km)}</span>
				</div>
				{#if pk != null}<div class="bar {kmStatus}"><span style="width:{Math.max(4, pk * 100)}%"></span></div>{/if}
				{#if s.km_stale}<div class="stale">Km non aggiornati di recente</div>{/if}
			{/if}
			{#if s.next_due_date}
				<div class="prow">
					<span>Tempo</span><span class="num">entro {fmtDate(s.next_due_date)}</span>
				</div>
				{#if pt != null}<div class="bar {timeStatus}"><span style="width:{Math.max(4, pt * 100)}%"></span></div>{/if}
			{/if}
		</div>
	{/if}
	{#if s.last_done_date || s.last_done_km != null}
		<div class="subtle">
			Ultima volta: {s.last_done_date ? fmtDate(s.last_done_date) : ''}{s.last_done_date && s.last_done_km != null ? ' · ' : ''}{s.last_done_km != null ? fmtKm(s.last_done_km) : ''}
		</div>
	{/if}

	<div class="foot">
		{#if s.status === 'setup'}
			<button class="btn btn-primary btn-sm" onclick={() => openDialog({ kind: 'schedule', vehicleId: s.vehicle_id, schedule: s })}>
				<Icon name="pencil" size={16} />Completa
			</button>
		{:else}
			<button class="btn btn-primary btn-sm" onclick={() => openDialog({ kind: 'event', vehicleId: s.vehicle_id, schedule: s })}>
				<Icon name="check" size={16} />Segna fatto
			</button>
		{/if}
		<button class="btn btn-sm" onclick={() => openDialog({ kind: 'schedule', vehicleId: s.vehicle_id, schedule: s })}>Modifica</button>
		{#if s.next_due_date}
			<a class="icon-btn" href="/api/schedules/{s.id}/calendar.ics" download data-sveltekit-reload aria-label="Aggiungi al calendario" title="Aggiungi al calendario">
				<Icon name="calendar-plus" size={18} />
			</a>
		{/if}
	</div>
</article>

<style>
	.sched {
		padding: 16px;
		display: grid;
		gap: 12px;
		scroll-margin-top: 80px;
	}
	.sched:target {
		outline: 2px solid var(--primary);
	}
	.inactive {
		opacity: 0.65;
	}
	.head {
		display: flex;
		align-items: center;
		gap: 8px 12px;
		flex-wrap: wrap;
	}
	.ttl {
		flex: 1;
		min-width: 140px;
	}
	.progress {
		display: grid;
		gap: 6px;
	}
	.prow {
		display: flex;
		justify-content: space-between;
		flex-wrap: wrap;
		gap: 0 8px;
		font-size: 13px;
		color: var(--fg-muted);
		font-weight: 500;
	}
	.stale {
		font-size: 13px;
		color: var(--warn);
		font-weight: 600;
	}
	.setup {
		margin: 0;
	}
	.foot {
		display: flex;
		gap: 8px;
		flex-wrap: wrap;
	}
</style>
