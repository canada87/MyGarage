<script lang="ts">
	import { dialog } from '#lib/state.svelte.ts';
	import ActionsSheet from './dialogs/ActionsSheet.svelte';
	import ConfirmDialog from './dialogs/ConfirmDialog.svelte';
	import ContactDialog from './dialogs/ContactDialog.svelte';
	import DocumentDialog from './dialogs/DocumentDialog.svelte';
	import EventDialog from './dialogs/EventDialog.svelte';
	import KmDialog from './dialogs/KmDialog.svelte';
	import MountDialog from './dialogs/MountDialog.svelte';
	import ScheduleDialog from './dialogs/ScheduleDialog.svelte';
	import TyreSetDialog from './dialogs/TyreSetDialog.svelte';
</script>

<!-- Keyed so that opening a new dialog always remounts it with fresh state. -->
{#key dialog.current}
	{#if dialog.current}
	{@const d = dialog.current}
	{#if d.kind === 'km'}
		<KmDialog vehicleId={d.vehicleId} />
	{:else if d.kind === 'event'}
		<EventDialog vehicleId={d.vehicleId} schedule={d.schedule} event={d.event} />
	{:else if d.kind === 'schedule'}
		<ScheduleDialog vehicleId={d.vehicleId} schedule={d.schedule} />
	{:else if d.kind === 'document'}
		<DocumentDialog vehicleId={d.vehicleId} eventId={d.eventId} doc={d.doc} />
	{:else if d.kind === 'contact'}
		<ContactDialog vehicleId={d.vehicleId} contact={d.contact} />
	{:else if d.kind === 'tyreSet'}
		<TyreSetDialog vehicleId={d.vehicleId} set={d.set} />
	{:else if d.kind === 'mount'}
		<MountDialog set={d.set} action={d.action} />
	{:else if d.kind === 'actions'}
		<ActionsSheet vehicleId={d.vehicleId} />
	{:else if d.kind === 'confirm'}
		<ConfirmDialog title={d.title} message={d.message} confirmLabel={d.confirmLabel} danger={d.danger} onConfirm={d.onConfirm} />
	{/if}
	{/if}
{/key}
