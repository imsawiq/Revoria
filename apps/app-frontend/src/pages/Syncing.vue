<script setup lang="ts">
import {
	FolderIcon,
	FolderOpenIcon,
	LinkIcon,
	PlusIcon,
	TrashIcon,
	UpdatedIcon,
} from '@modrinth/assets'
import { Button, Card, Checkbox, injectNotificationManager } from '@modrinth/ui'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { computed, onMounted, ref } from 'vue'

import { applyAllSyncTargets, getSyncState, removeSyncTarget, setSyncTarget } from '@/helpers/syncing.js'
import { openPath } from '@/helpers/utils.js'
import { useBreadcrumbs } from '@/store/breadcrumbs'

type SyncTarget = {
	path: string
	is_file: boolean
	enabled: boolean
	default_target: boolean
	sync_count: number
	cannot_sync_count: number
}

type SyncState = {
	sync_folder: string
	total_count: number
	folders: SyncTarget[]
	files: SyncTarget[]
}

const breadcrumbs = useBreadcrumbs()
const { handleError, addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	breadcrumb: {
		id: 'syncing.breadcrumb',
		defaultMessage: 'Syncing',
	},
	title: {
		id: 'syncing.title',
		defaultMessage: 'Instance syncing',
	},
	description: {
		id: 'syncing.description',
		defaultMessage:
			'Share one real folder or file across all instances. Folder targets are linked, file targets are hard-linked.',
	},
	howItWorks: {
		id: 'syncing.how-it-works',
		defaultMessage:
			'When enabled, existing data is moved into the shared sync folder and each instance path is replaced with a link.',
	},
	openFolder: {
		id: 'syncing.action.open-folder',
		defaultMessage: 'Open sync folder',
	},
	applyAll: {
		id: 'syncing.action.apply',
		defaultMessage: 'Apply to all instances',
	},
	reload: {
		id: 'syncing.action.reload',
		defaultMessage: 'Reload',
	},
	sharedFolder: {
		id: 'syncing.shared-folder',
		defaultMessage: 'Shared data folder',
	},
	sharedFolderHint: {
		id: 'syncing.shared-folder-hint',
		defaultMessage: 'This folder stores the real shared files and folders.',
	},
	totalInstances: {
		id: 'syncing.total-instances',
		defaultMessage: 'Instances',
	},
	conflictsHint: {
		id: 'syncing.conflicts-hint',
		defaultMessage: 'Conflicts mean some instances still have separate data at that path.',
	},
	defaultFolders: {
		id: 'syncing.section.default-folders',
		defaultMessage: 'Default folders',
	},
	customFolders: {
		id: 'syncing.section.custom-folders',
		defaultMessage: 'Custom folders',
	},
	customFiles: {
		id: 'syncing.section.custom-files',
		defaultMessage: 'Custom files',
	},
	addTarget: {
		id: 'syncing.add-target',
		defaultMessage: 'Add sync target',
	},
	addPlaceholder: {
		id: 'syncing.add-placeholder',
		defaultMessage: 'resourcepacks or options.txt',
	},
	targetTypeFolder: {
		id: 'syncing.target-type.folder',
		defaultMessage: 'Folder',
	},
	targetTypeFile: {
		id: 'syncing.target-type.file',
		defaultMessage: 'File',
	},
	add: {
		id: 'syncing.action.add',
		defaultMessage: 'Add target',
	},
	emptyCustomFolders: {
		id: 'syncing.empty.custom-folders',
		defaultMessage: 'No custom folder targets yet.',
	},
	emptyCustomFiles: {
		id: 'syncing.empty.custom-files',
		defaultMessage: 'No custom file targets yet.',
	},
	synced: {
		id: 'syncing.target.synced',
		defaultMessage: '{synced}/{total} synced',
	},
	conflicts: {
		id: 'syncing.target.conflicts',
		defaultMessage: '{count} conflicting instances',
	},
	defaultLabel: {
		id: 'syncing.target.default',
		defaultMessage: 'Default',
	},
	customLabel: {
		id: 'syncing.target.custom',
		defaultMessage: 'Custom',
	},
	added: {
		id: 'syncing.notification.added',
		defaultMessage: 'Sync target added.',
	},
	updated: {
		id: 'syncing.notification.updated',
		defaultMessage: 'Sync targets updated.',
	},
	removed: {
		id: 'syncing.notification.removed',
		defaultMessage: 'Sync target removed.',
	},
	statusEnabled: {
		id: 'syncing.status.enabled',
		defaultMessage: 'Enabled',
	},
	statusDisabled: {
		id: 'syncing.status.disabled',
		defaultMessage: 'Disabled',
	},
})

breadcrumbs.setRootContext({ name: formatMessage(messages.breadcrumb), link: '/syncing' })

const syncState = ref<SyncState | null>(null)
const loading = ref(false)
const pendingKey = ref<string | null>(null)
const newTargetPath = ref('')
const newTargetType = ref<'folder' | 'file'>('folder')

const defaultFolders = computed(() =>
	(syncState.value?.folders ?? []).filter((target) => target.default_target),
)
const customFolders = computed(() =>
	(syncState.value?.folders ?? []).filter((target) => !target.default_target),
)
const customFiles = computed(() => syncState.value?.files ?? [])

async function loadState() {
	loading.value = true
	try {
		syncState.value = await getSyncState()
	} catch (error) {
		handleError(error as Error)
	} finally {
		loading.value = false
	}
}

async function toggleTarget(target: SyncTarget, enabled: boolean) {
	pendingKey.value = `${target.path}:${target.is_file}`
	try {
		syncState.value = await setSyncTarget(target.path, target.is_file, enabled)
		addNotification({
			title: formatMessage(messages.title),
			text: formatMessage(messages.updated),
			type: 'success',
		})
	} catch (error) {
		handleError(error as Error)
	} finally {
		pendingKey.value = null
	}
}

async function removeTarget(target: SyncTarget) {
	pendingKey.value = `${target.path}:${target.is_file}`
	try {
		syncState.value = await removeSyncTarget(target.path, target.is_file)
		addNotification({
			title: formatMessage(messages.title),
			text: formatMessage(messages.removed),
			type: 'success',
		})
	} catch (error) {
		handleError(error as Error)
	} finally {
		pendingKey.value = null
	}
}

async function addTarget() {
	const path = newTargetPath.value.trim()
	if (!path) return

	pendingKey.value = `new:${newTargetType.value}:${path}`
	try {
		syncState.value = await setSyncTarget(path, newTargetType.value === 'file', true)
		newTargetPath.value = ''
		addNotification({
			title: formatMessage(messages.title),
			text: formatMessage(messages.added),
			type: 'success',
		})
	} catch (error) {
		handleError(error as Error)
	} finally {
		pendingKey.value = null
	}
}

async function applyAll() {
	pendingKey.value = 'apply-all'
	try {
		syncState.value = await applyAllSyncTargets()
		addNotification({
			title: formatMessage(messages.title),
			text: formatMessage(messages.updated),
			type: 'success',
		})
	} catch (error) {
		handleError(error as Error)
	} finally {
		pendingKey.value = null
	}
}

async function openSyncFolder() {
	if (!syncState.value?.sync_folder) return
	await openPath(syncState.value.sync_folder).catch(handleError)
}

function isPending(target: SyncTarget) {
	return pendingKey.value === `${target.path}:${target.is_file}`
}

onMounted(loadState)
</script>

<template>
	<section class="syncing-page">
		<header class="page-header">
			<div>
				<h1>{{ formatMessage(messages.title) }}</h1>
				<p class="description">{{ formatMessage(messages.description) }}</p>
				<p class="description description-subtle">{{ formatMessage(messages.howItWorks) }}</p>
			</div>
			<div class="page-actions">
				<Button @click="openSyncFolder">
					<FolderOpenIcon />
					{{ formatMessage(messages.openFolder) }}
				</Button>
				<Button color="primary" :disabled="pendingKey === 'apply-all'" @click="applyAll">
					<UpdatedIcon />
					{{ formatMessage(messages.applyAll) }}
				</Button>
				<Button :disabled="loading" @click="loadState">
					<UpdatedIcon />
					{{ formatMessage(messages.reload) }}
				</Button>
			</div>
		</header>

		<div class="summary-grid">
			<Card class="summary-card">
				<div class="summary-head">
					<span class="summary-label">{{ formatMessage(messages.sharedFolder) }}</span>
					<FolderIcon class="summary-symbol" />
				</div>
				<span class="summary-value path-value">{{ syncState?.sync_folder ?? '...' }}</span>
				<p class="summary-help">{{ formatMessage(messages.sharedFolderHint) }}</p>
			</Card>
			<Card class="summary-card">
				<div class="summary-head">
					<span class="summary-label">{{ formatMessage(messages.totalInstances) }}</span>
					<LinkIcon class="summary-symbol" />
				</div>
				<span class="summary-value">{{ syncState?.total_count ?? 0 }}</span>
				<p class="summary-help">{{ formatMessage(messages.conflictsHint) }}</p>
			</Card>
		</div>

		<Card class="panel-card">
			<div class="section-heading">
				<h2>{{ formatMessage(messages.addTarget) }}</h2>
			</div>
			<div class="add-row">
				<div class="input-shell">
					<LinkIcon class="input-icon" />
					<input
						v-model="newTargetPath"
						type="text"
						:placeholder="formatMessage(messages.addPlaceholder)"
						@keydown.enter.prevent="addTarget"
					/>
				</div>
				<select v-model="newTargetType" class="type-select">
					<option value="folder">{{ formatMessage(messages.targetTypeFolder) }}</option>
					<option value="file">{{ formatMessage(messages.targetTypeFile) }}</option>
				</select>
				<Button color="primary" :disabled="!newTargetPath.trim()" @click="addTarget">
					<PlusIcon />
					{{ formatMessage(messages.add) }}
				</Button>
			</div>
		</Card>

		<div class="panels-grid">
			<Card class="panel-card">
				<div class="section-heading">
					<h2>{{ formatMessage(messages.defaultFolders) }}</h2>
				</div>
				<div class="targets-list">
					<div v-for="target in defaultFolders" :key="`default-${target.path}`" class="target-row">
						<div class="target-main">
							<div class="target-icon">
								<FolderIcon />
							</div>
							<div class="target-copy">
								<div class="target-topline">
									<span class="target-path">{{ target.path }}</span>
									<span class="badge">{{ formatMessage(messages.defaultLabel) }}</span>
									<span class="status-chip" :class="{ 'status-chip-off': !target.enabled }">
										{{ formatMessage(target.enabled ? messages.statusEnabled : messages.statusDisabled) }}
									</span>
								</div>
								<div class="target-meta">
									<span>{{
										formatMessage(messages.synced, {
											synced: target.sync_count,
											total: syncState?.total_count ?? 0,
										})
									}}</span>
									<span v-if="target.cannot_sync_count > 0">{{ formatMessage(messages.conflicts, { count: target.cannot_sync_count }) }}</span>
								</div>
							</div>
						</div>
						<Checkbox :model-value="target.enabled" :disabled="isPending(target)" @update:model-value="toggleTarget(target, $event)" />
					</div>
				</div>
			</Card>

			<Card class="panel-card">
				<div class="section-heading">
					<h2>{{ formatMessage(messages.customFolders) }}</h2>
				</div>
				<div v-if="customFolders.length" class="targets-list">
					<div v-for="target in customFolders" :key="`folder-${target.path}`" class="target-row">
						<div class="target-main">
							<div class="target-icon">
								<FolderIcon />
							</div>
							<div class="target-copy">
								<div class="target-topline">
									<span class="target-path">{{ target.path }}</span>
									<span class="badge badge-custom">{{ formatMessage(messages.customLabel) }}</span>
									<span class="status-chip" :class="{ 'status-chip-off': !target.enabled }">
										{{ formatMessage(target.enabled ? messages.statusEnabled : messages.statusDisabled) }}
									</span>
								</div>
								<div class="target-meta">
									<span>{{
										formatMessage(messages.synced, {
											synced: target.sync_count,
											total: syncState?.total_count ?? 0,
										})
									}}</span>
									<span v-if="target.cannot_sync_count > 0">{{ formatMessage(messages.conflicts, { count: target.cannot_sync_count }) }}</span>
								</div>
							</div>
						</div>
						<div class="target-actions">
							<Checkbox :model-value="target.enabled" :disabled="isPending(target)" @update:model-value="toggleTarget(target, $event)" />
							<button class="danger-button" :disabled="isPending(target)" @click="removeTarget(target)">
								<TrashIcon />
							</button>
						</div>
					</div>
				</div>
				<p v-else class="empty">{{ formatMessage(messages.emptyCustomFolders) }}</p>
			</Card>

			<Card class="panel-card panels-wide">
				<div class="section-heading">
					<h2>{{ formatMessage(messages.customFiles) }}</h2>
				</div>
				<div v-if="customFiles.length" class="targets-list">
					<div v-for="target in customFiles" :key="`file-${target.path}`" class="target-row">
						<div class="target-main">
							<div class="target-icon target-icon-file">
								<LinkIcon />
							</div>
							<div class="target-copy">
								<div class="target-topline">
									<span class="target-path">{{ target.path }}</span>
									<span class="badge badge-custom">{{ formatMessage(messages.customLabel) }}</span>
									<span class="status-chip">
										{{ formatMessage(messages.statusEnabled) }}
									</span>
								</div>
								<div class="target-meta">
									<span>{{
										formatMessage(messages.synced, {
											synced: target.sync_count,
											total: syncState?.total_count ?? 0,
										})
									}}</span>
									<span v-if="target.cannot_sync_count > 0">{{ formatMessage(messages.conflicts, { count: target.cannot_sync_count }) }}</span>
								</div>
							</div>
						</div>
						<button class="danger-button" :disabled="isPending(target)" @click="removeTarget(target)">
							<TrashIcon />
						</button>
					</div>
				</div>
				<p v-else class="empty">{{ formatMessage(messages.emptyCustomFiles) }}</p>
			</Card>
		</div>
	</section>
</template>

<style scoped lang="scss">
.syncing-page {
	display: flex;
	flex-direction: column;
	gap: 1rem;
	padding: 1rem;
}

.page-header {
	display: flex;
	align-items: flex-start;
	justify-content: space-between;
	gap: 1rem;
	flex-wrap: wrap;
}

h1,
h2 {
	margin: 0;
}

h1 {
	font-size: 1.45rem;
	line-height: 1.1;
	color: var(--color-contrast);
}

.description {
	margin: 0.45rem 0 0;
	max-width: 56rem;
	color: var(--color-secondary-text);
}

.description-subtle {
	font-size: 0.95rem;
}

.page-actions {
	display: flex;
	flex-wrap: wrap;
	align-items: center;
	gap: 0.75rem;
	margin-left: auto;
	justify-content: flex-end;
}

.summary-grid,
.panels-grid {
	display: grid;
	gap: 1rem;
}

.summary-grid {
	grid-template-columns: minmax(0, 2fr) minmax(0, 1fr);
}

.summary-card,
.panel-card {
	display: flex;
	flex-direction: column;
	padding: 1rem 1.1rem;
	border-radius: 1.25rem;
	background: var(--color-bg-raised);
	border: 1px solid var(--color-button-bg);
	box-shadow: 0 8px 24px color-mix(in srgb, black 12%, transparent);
}

.summary-card {
	gap: 0.45rem;
}

.summary-head {
	display: flex;
	align-items: center;
	justify-content: space-between;
	gap: 0.75rem;
}

.summary-symbol {
	width: 1rem;
	height: 1rem;
	color: var(--color-brand);
}

.summary-label {
	font-size: 0.82rem;
	text-transform: uppercase;
	letter-spacing: 0.08em;
	color: var(--color-secondary-text);
}

.summary-value {
	font-size: 1.1rem;
	font-weight: 700;
	color: var(--color-contrast);
}

.summary-help {
	margin: 0;
	font-size: 0.9rem;
	color: var(--color-secondary-text);
}

.path-value {
	word-break: break-all;
	font-size: 0.98rem;
}

.section-heading {
	display: flex;
	align-items: center;
	justify-content: space-between;
	margin-bottom: 0.9rem;
}

.add-row {
	display: flex;
	flex-wrap: wrap;
	gap: 0.75rem;
}

.input-shell {
	display: flex;
	flex: 1 1 26rem;
	align-items: center;
	gap: 0.65rem;
	padding: 0.9rem 1rem;
	border-radius: 1rem;
	background: color-mix(in srgb, var(--color-button-bg) 82%, transparent);
	border: 1px solid color-mix(in srgb, var(--color-button-bg-hover) 65%, transparent);
}

.input-shell input,
.type-select {
	border: 0;
	outline: 0;
	background: transparent;
	color: var(--color-contrast);
}

.input-shell input {
	flex: 1;
	font: inherit;
}

.type-select {
	min-width: 8rem;
	padding: 0.9rem 1rem;
	border-radius: 1rem;
	background: color-mix(in srgb, var(--color-button-bg) 82%, transparent);
	border: 1px solid color-mix(in srgb, var(--color-button-bg-hover) 65%, transparent);
}

.input-icon {
	width: 1rem;
	height: 1rem;
	color: var(--color-secondary-text);
}

.panels-grid {
	grid-template-columns: repeat(2, minmax(0, 1fr));
}

.panels-wide {
	grid-column: 1 / -1;
}

.targets-list {
	display: flex;
	flex-direction: column;
	gap: 0.7rem;
}

.target-row {
	display: flex;
	align-items: center;
	justify-content: space-between;
	gap: 0.85rem;
	padding: 0.85rem 0.95rem;
	border-radius: 1rem;
	background: color-mix(in srgb, var(--color-button-bg) 86%, transparent);
	border: 1px solid color-mix(in srgb, var(--color-button-bg-hover) 45%, transparent);
}

.target-main,
.target-actions,
.target-topline,
.target-meta {
	display: flex;
	align-items: center;
}

.target-main {
	min-width: 0;
	gap: 0.8rem;
}

.target-copy {
	min-width: 0;
}

.target-topline {
	gap: 0.55rem;
	margin-bottom: 0.2rem;
}

.target-meta {
	flex-wrap: wrap;
	gap: 0.7rem;
	color: var(--color-secondary-text);
	font-size: 0.9rem;
}

.target-path {
	font-weight: 700;
	color: var(--color-contrast);
	word-break: break-all;
}

.target-icon {
	display: grid;
	flex: 0 0 auto;
	place-items: center;
	width: 2.35rem;
	height: 2.35rem;
	border-radius: 0.9rem;
	background: color-mix(in srgb, var(--color-button-bg-hover) 65%, transparent);
	color: var(--color-contrast);
}

.target-icon-file {
	color: #7cc4ff;
	background: color-mix(in srgb, #7cc4ff 14%, var(--color-button-bg));
}

.badge {
	padding: 0.22rem 0.5rem;
	border-radius: 999px;
	font-size: 0.74rem;
	font-weight: 700;
	letter-spacing: 0.04em;
	text-transform: uppercase;
	color: var(--color-brand);
	background: color-mix(in srgb, var(--color-brand) 16%, transparent);
}

.badge-custom {
	color: #7cc4ff;
	background: color-mix(in srgb, #7cc4ff 14%, transparent);
}

.status-chip {
	padding: 0.22rem 0.5rem;
	border-radius: 999px;
	font-size: 0.74rem;
	font-weight: 600;
	color: var(--color-green-highlight);
	background: color-mix(in srgb, var(--color-green) 14%, transparent);
}

.status-chip-off {
	color: var(--color-secondary-text);
	background: color-mix(in srgb, var(--color-button-bg-hover) 75%, transparent);
}

.target-actions {
	gap: 0.5rem;
}

.danger-button {
	display: grid;
	place-items: center;
	width: 2.25rem;
	height: 2.25rem;
	border: 0;
	border-radius: 0.85rem;
	background: color-mix(in srgb, var(--color-red) 14%, var(--color-button-bg));
	color: var(--color-red-highlight);
	cursor: pointer;
}

.danger-button:disabled {
	cursor: default;
	opacity: 0.55;
}

.empty {
	margin: 0;
	color: var(--color-secondary-text);
}

@media (max-width: 1100px) {
	.summary-grid,
	.panels-grid {
		grid-template-columns: 1fr;
	}

	.page-header {
		flex-direction: column;
	}

	.page-actions {
		margin-left: 0;
		justify-content: flex-start;
	}

}
</style>
