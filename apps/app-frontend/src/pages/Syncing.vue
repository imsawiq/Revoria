<script setup lang="ts">
import {
	FolderIcon,
	FolderOpenIcon,
	LinkIcon,
	PlusIcon,
	TrashIcon,
	UpdatedIcon,
} from '@modrinth/assets'
import {
	ButtonStyled,
	Card,
	Checkbox,
	DropdownSelect,
	injectNotificationManager,
} from '@modrinth/ui'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { computed, onMounted, ref } from 'vue'

import {
	applyAllSyncTargets,
	getSyncState,
	removeSyncTarget,
	setSyncTarget,
} from '@/helpers/syncing.js'
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
const targetTypeOptions: Array<'folder' | 'file'> = ['folder', 'file']

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
	<div class="syncing-page">
		<Card class="section-card">
			<header class="page-header">
				<div>
					<h2 class="page-title">{{ formatMessage(messages.title) }}</h2>
					<p class="page-description">{{ formatMessage(messages.description) }}</p>
					<p class="page-description page-description-subtle">
						{{ formatMessage(messages.howItWorks) }}
					</p>
				</div>
				<div class="page-actions">
					<ButtonStyled color="brand">
						<button :disabled="pendingKey === 'apply-all'" @click="applyAll">
							<UpdatedIcon />
							{{ formatMessage(messages.applyAll) }}
						</button>
					</ButtonStyled>
					<ButtonStyled>
						<button class="open-folder-button" @click="openSyncFolder">
							<FolderOpenIcon />
							{{ formatMessage(messages.openFolder) }}
						</button>
					</ButtonStyled>
					<ButtonStyled>
						<button :disabled="loading" @click="loadState">
							<UpdatedIcon />
							{{ formatMessage(messages.reload) }}
						</button>
					</ButtonStyled>
				</div>
			</header>

			<div class="summary-row">
				<div class="summary-card">
					<div class="summary-head">
						<span class="summary-label">{{ formatMessage(messages.sharedFolder) }}</span>
						<FolderIcon class="summary-symbol" />
					</div>
					<span class="summary-value path-value">{{ syncState?.sync_folder ?? '...' }}</span>
					<p class="summary-help">{{ formatMessage(messages.sharedFolderHint) }}</p>
				</div>
				<div class="summary-card">
					<div class="summary-head">
						<span class="summary-label">{{ formatMessage(messages.totalInstances) }}</span>
						<LinkIcon class="summary-symbol" />
					</div>
					<span class="summary-value">{{ syncState?.total_count ?? 0 }}</span>
					<p class="summary-help">{{ formatMessage(messages.conflictsHint) }}</p>
				</div>
			</div>
		</Card>

		<Card class="section-card">
			<div class="section-heading">
				<h2 class="section-title">{{ formatMessage(messages.addTarget) }}</h2>
			</div>
			<div class="add-row">
				<div class="input-shell iconified-input">
					<LinkIcon class="input-icon" />
					<input
						v-model="newTargetPath"
						type="text"
						:placeholder="formatMessage(messages.addPlaceholder)"
						@keydown.enter.prevent="addTarget"
					/>
				</div>
				<DropdownSelect
					v-model="newTargetType"
					class="type-select"
					:name="formatMessage(messages.addTarget)"
					:options="targetTypeOptions"
					:display-name="
						(option?: 'folder' | 'file') =>
							option === 'file'
								? formatMessage(messages.targetTypeFile)
								: formatMessage(messages.targetTypeFolder)
					"
				/>
				<ButtonStyled color="brand">
					<button :disabled="!newTargetPath.trim()" @click="addTarget">
						<PlusIcon />
						{{ formatMessage(messages.add) }}
					</button>
				</ButtonStyled>
			</div>
		</Card>

		<div class="section-grid">
			<Card class="section-card">
				<div class="section-heading">
					<h2 class="section-title">{{ formatMessage(messages.defaultFolders) }}</h2>
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
										{{
											formatMessage(
												target.enabled ? messages.statusEnabled : messages.statusDisabled,
											)
										}}
									</span>
								</div>
								<div class="target-meta">
									<span>{{
										formatMessage(messages.synced, {
											synced: target.sync_count,
											total: syncState?.total_count ?? 0,
										})
									}}</span>
									<span v-if="target.cannot_sync_count > 0">{{
										formatMessage(messages.conflicts, { count: target.cannot_sync_count })
									}}</span>
								</div>
							</div>
						</div>
						<Checkbox
							:model-value="target.enabled"
							:disabled="isPending(target)"
							@update:model-value="toggleTarget(target, $event)"
						/>
					</div>
				</div>
			</Card>

			<Card class="section-card">
				<div class="section-heading">
					<h2 class="section-title">{{ formatMessage(messages.customFolders) }}</h2>
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
										{{
											formatMessage(
												target.enabled ? messages.statusEnabled : messages.statusDisabled,
											)
										}}
									</span>
								</div>
								<div class="target-meta">
									<span>{{
										formatMessage(messages.synced, {
											synced: target.sync_count,
											total: syncState?.total_count ?? 0,
										})
									}}</span>
									<span v-if="target.cannot_sync_count > 0">{{
										formatMessage(messages.conflicts, { count: target.cannot_sync_count })
									}}</span>
								</div>
							</div>
						</div>
						<div class="target-actions">
							<Checkbox
								:model-value="target.enabled"
								:disabled="isPending(target)"
								@update:model-value="toggleTarget(target, $event)"
							/>
							<button
								class="danger-button"
								:disabled="isPending(target)"
								@click="removeTarget(target)"
							>
								<TrashIcon />
							</button>
						</div>
					</div>
				</div>
				<div v-else class="empty-state">{{ formatMessage(messages.emptyCustomFolders) }}</div>
			</Card>

			<Card class="section-card section-card-wide">
				<div class="section-heading">
					<h2 class="section-title">{{ formatMessage(messages.customFiles) }}</h2>
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
									<span v-if="target.cannot_sync_count > 0">{{
										formatMessage(messages.conflicts, { count: target.cannot_sync_count })
									}}</span>
								</div>
							</div>
						</div>
						<button
							class="danger-button"
							:disabled="isPending(target)"
							@click="removeTarget(target)"
						>
							<TrashIcon />
						</button>
					</div>
				</div>
				<div v-else class="empty-state">{{ formatMessage(messages.emptyCustomFiles) }}</div>
			</Card>
		</div>
	</div>
</template>

<style scoped lang="scss">
.syncing-page {
	display: flex;
	flex-direction: column;
	gap: 1.5rem;
	padding: 1.5rem;
}

.section-card {
	padding: 1.5rem;
	display: flex;
	flex-direction: column;
	gap: 1.5rem;
	background:
		linear-gradient(
			180deg,
			color-mix(in srgb, var(--color-glass-bg-strong) 88%, transparent),
			color-mix(in srgb, var(--color-glass-bg) 92%, transparent)
		);
	border: 1px solid color-mix(in srgb, var(--glass-border) 84%, var(--color-brand-highlight));
	box-shadow: var(--shadow-floating);
}

.page-header {
	display: flex;
	align-items: flex-start;
	justify-content: space-between;
	gap: 1rem;
	flex-wrap: wrap;
}

.summary-card {
	background: color-mix(in srgb, var(--color-glass-bg) 92%, var(--color-button-bg) 8%);
	border: 1px solid var(--glass-border);
	border-radius: var(--radius-lg);
	padding: 1rem;
	display: flex;
	flex-direction: column;
	gap: 0.45rem;
	box-shadow: var(--shadow-card);
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
	color: var(--color-secondary);
}

.summary-value {
	font-size: 1.1rem;
	font-weight: 700;
	color: var(--color-contrast);
}

.summary-help {
	margin: 0;
	font-size: 0.9rem;
	color: var(--color-secondary);
}

.path-value {
	word-break: break-all;
	font-size: 0.98rem;
}

.page-title,
.section-title {
	margin: 0;
	color: var(--color-contrast);
}

.page-title {
	font-size: 1.25rem;
	font-weight: 800;
}

.section-title {
	font-size: 1rem;
	font-weight: 700;
}

.page-description {
	margin: 0.35rem 0 0;
	max-width: 56rem;
	color: var(--color-secondary);
}

.page-description-subtle {
	font-size: 0.95rem;
}

.page-actions {
	display: flex;
	flex-wrap: wrap;
	align-items: center;
	gap: 0.5rem;
	margin-left: auto;
	justify-content: flex-end;
}

.open-folder-button {
	background: color-mix(in srgb, var(--color-button-bg) 88%, var(--color-glass-bg-strong) 12%);
	border: 1px solid var(--glass-border);
	color: var(--color-contrast);
	box-shadow: var(--shadow-card);
}

.open-folder-button:hover {
	background: var(--color-button-bg-hover);
	border-color: var(--color-brand-highlight);
}

.summary-row {
	display: grid;
	grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
	gap: 1rem;
}

.section-heading {
	display: flex;
	align-items: center;
	justify-content: space-between;
}

.add-row {
	display: grid;
	grid-template-columns: minmax(0, 4.5fr) minmax(11rem, 1.35fr) auto;
	gap: 0.75rem;
	align-items: center;
}

.input-shell {
	width: 100%;
	min-width: 0;
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
	height: 3rem;
	background: color-mix(in oklch, var(--color-glass-bg-strong) 70%, transparent);
	border: 1px solid var(--glass-border);
	border-radius: var(--radius-md);
	box-shadow:
		var(--shadow-inset-sm),
		0 0 0 0 transparent;
	transition:
		border-color 150ms ease,
		box-shadow 150ms ease,
		background-color 150ms ease;
}

.input-shell input:focus,
.input-shell input:focus-visible {
	border-color: var(--color-brand);
	box-shadow:
		var(--shadow-inset-sm),
		0 0 0 0.2rem var(--color-brand-shadow);
}

.input-shell input::placeholder {
	color: var(--color-base);
	opacity: 0.6;
}

.type-select {
	width: 100%;
	min-width: 0;
}

.input-icon {
	width: 1rem;
	height: 1rem;
	color: var(--color-secondary);
}

.section-grid {
	grid-template-columns: repeat(2, minmax(0, 1fr));
	display: grid;
	gap: 1rem;
}

.section-card-wide {
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
	border-radius: var(--radius-md);
	background: color-mix(in srgb, var(--color-glass-bg) 88%, var(--color-button-bg) 12%);
	border: 1px solid var(--glass-border);
	box-shadow: var(--shadow-card);
	transition:
		border-color 180ms ease,
		box-shadow 180ms ease,
		background-color 180ms ease;
}

.target-row:hover {
	border-color: var(--color-brand-highlight);
	box-shadow: var(--shadow-floating);
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
	display: flex;
	align-items: center;
	flex-wrap: wrap;
	gap: 0.55rem;
	margin-bottom: 0.2rem;
}

.target-meta {
	flex-wrap: wrap;
	gap: 0.7rem;
	color: var(--color-secondary);
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
	background: var(--color-button-bg);
	color: var(--color-contrast);
}

.target-icon-file {
	color: var(--color-brand);
	background: var(--color-button-bg);
}

.badge {
	display: inline-flex;
	align-items: center;
	justify-content: center;
	padding: 0.18rem 0.56rem;
	height: 1.4rem;
	border-radius: 999px;
	font-size: 0.74rem;
	font-weight: 700;
	line-height: 1.05;
	letter-spacing: 0.04em;
	text-transform: uppercase;
	color: var(--color-secondary);
	background: var(--color-button-bg);
	border: 1px solid var(--glass-border);
	vertical-align: middle;
}

.badge-custom {
	color: var(--color-secondary);
	background: var(--color-button-bg);
	border: 1px solid var(--glass-border);
}

.status-chip {
	display: inline-flex;
	align-items: center;
	justify-content: center;
	padding: 0.18rem 0.56rem;
	height: 1.4rem;
	border-radius: 999px;
	font-size: 0.74rem;
	font-weight: 600;
	line-height: 1.05;
	color: var(--color-contrast);
	background: var(--color-button-bg);
	border: 1px solid var(--glass-border);
	vertical-align: middle;
}

.status-chip-off {
	color: var(--color-secondary);
	background: var(--color-button-bg);
	border: 1px solid var(--glass-border);
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
	background: var(--color-button-bg);
	border: 1px solid var(--color-button-border);
	color: var(--color-red-highlight);
	cursor: pointer;
}

.danger-button:disabled {
	cursor: default;
	opacity: 0.55;
}

.empty-state {
	padding: 1rem;
	border-radius: var(--radius-lg);
	background: var(--color-button-bg);
	color: var(--color-secondary);
	text-align: center;
}

@media (max-width: 1100px) {
	.section-grid {
		grid-template-columns: 1fr;
	}

	.add-row {
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
