<script setup lang="ts">
import { FolderOpenIcon, TrashIcon, UpdatedIcon } from '@modrinth/assets'
import { Button, ButtonStyled, Card, Checkbox, injectNotificationManager } from '@modrinth/ui'
import { formatBytes } from '@modrinth/utils'
import { invoke } from '@tauri-apps/api/core'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { useStorage } from '@vueuse/core'
import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'
import { computed, onMounted, ref } from 'vue'

import ConfirmModalWrapper from '@/components/ui/modal/ConfirmModalWrapper.vue'
import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import { list, remove, get_full_path } from '@/helpers/profile.js'
import { openPath } from '@/helpers/utils.js'
import type { GameInstance } from '@/helpers/types'
import { useBreadcrumbs } from '@/store/breadcrumbs'

dayjs.extend(relativeTime)

const breadcrumbs = useBreadcrumbs()
const { handleError, addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	breadcrumb: {
		id: 'maintenance.breadcrumb',
		defaultMessage: 'Maintenance',
	},
	ruleLogs: {
		id: 'maintenance.rule.logs',
		defaultMessage: 'Logs',
	},
	ruleCrashReports: {
		id: 'maintenance.rule.crash-reports',
		defaultMessage: 'Crash reports',
	},
	ruleScreenshots: {
		id: 'maintenance.rule.screenshots',
		defaultMessage: 'Screenshots',
	},
	ruleNew: {
		id: 'maintenance.rule.new',
		defaultMessage: 'New rule',
	},
	never: {
		id: 'maintenance.last-played.never',
		defaultMessage: 'Never',
	},
	failedOpenFolder: {
		id: 'maintenance.error.open-folder',
		defaultMessage: 'Failed to open folder.',
	},
	cleanupCompleteTitle: {
		id: 'maintenance.cleanup.complete.title',
		defaultMessage: 'Cleanup complete',
	},
	cleanupCompleteText: {
		id: 'maintenance.cleanup.complete.text',
		defaultMessage: 'Freed {size}',
	},
	sectionTitle: {
		id: 'maintenance.section.memory-manager.title',
		defaultMessage: 'Memory Manager',
	},
	sectionDescription: {
		id: 'maintenance.section.memory-manager.description',
		defaultMessage: 'Scans instances and helps clear safe junk fast.',
	},
	scanning: {
		id: 'maintenance.action.scanning',
		defaultMessage: 'Scanning...',
	},
	rescan: {
		id: 'maintenance.action.rescan',
		defaultMessage: 'Rescan',
	},
	totalInstances: {
		id: 'maintenance.summary.total-instances',
		defaultMessage: 'Total instances',
	},
	totalSize: {
		id: 'maintenance.summary.total-size',
		defaultMessage: 'Total size',
	},
	noInstances: {
		id: 'maintenance.empty.no-instances',
		defaultMessage: 'No instances to analyze.',
	},
	clean: {
		id: 'maintenance.action.clean',
		defaultMessage: 'Clean',
	},
	folder: {
		id: 'maintenance.action.folder',
		defaultMessage: 'Folder',
	},
	delete: {
		id: 'maintenance.action.delete',
		defaultMessage: 'Delete',
	},
	rulesTitle: {
		id: 'maintenance.rules.title',
		defaultMessage: 'Cleanup rules',
	},
	rulesDescription: {
		id: 'maintenance.rules.description',
		defaultMessage:
			'Paths are relative to the instance folder. Separate multiple paths with commas.',
	},
	addRule: {
		id: 'maintenance.action.add-rule',
		defaultMessage: 'Add rule',
	},
	ruleNamePlaceholder: {
		id: 'maintenance.rule-name.placeholder',
		defaultMessage: 'Rule name',
	},
	rulePathsPlaceholder: {
		id: 'maintenance.rule-paths.placeholder',
		defaultMessage: 'logs, crash-reports',
	},
	deleteInstanceTitle: {
		id: 'maintenance.delete-instance.title',
		defaultMessage: 'Delete instance {name}?',
	},
	deleteInstanceDescription: {
		id: 'maintenance.delete-instance.description',
		defaultMessage: 'All data will be removed and cannot be recovered.',
	},
	cleanupModalHeader: {
		id: 'maintenance.cleanup.modal.header',
		defaultMessage: 'Instance cleanup',
	},
	safeToRemove: {
		id: 'maintenance.cleanup.safe-to-remove',
		defaultMessage: 'Safe to remove',
	},
	selectAll: {
		id: 'maintenance.action.select-all',
		defaultMessage: 'Select all',
	},
	clearSelection: {
		id: 'maintenance.action.clear-selection',
		defaultMessage: 'Clear',
	},
	selectedCount: {
		id: 'maintenance.cleanup.selected-count',
		defaultMessage: '{count} selected',
	},
	loadingInstances: {
		id: 'maintenance.loading.instances',
		defaultMessage: 'Loading instances...',
	},
	noCleanupTargets: {
		id: 'maintenance.cleanup.no-targets',
		defaultMessage: 'No cleanup files found for the selected rules.',
	},
})

breadcrumbs.setRootContext({ name: formatMessage(messages.breadcrumb), link: '/maintenance' })

const instances = ref<GameInstance[]>([])
const loadingInstances = ref(true)
const scanningSizes = ref(false)
const sizeByPath = ref<Record<string, number>>({})

const deleteConfirmModal = ref<InstanceType<typeof ConfirmModalWrapper>>()
const selectedForDelete = ref<GameInstance | null>(null)

const cleanupModal = ref<InstanceType<typeof ModalWrapper>>()
const cleanupTargets = ref<CleanupTarget[]>([])
const cleanupSelections = ref<Record<string, boolean>>({})
const cleanupLoading = ref(false)

const selectedInstancePath = ref<string | null>(null)

const defaultCleanupRules: CleanupRule[] = [
	{
		id: 'logs',
		label: formatMessage(messages.ruleLogs),
		paths: 'logs',
		enabled: true,
	},
	{
		id: 'crash-reports',
		label: formatMessage(messages.ruleCrashReports),
		paths: 'crash-reports',
		enabled: true,
	},
	{
		id: 'screenshots',
		label: formatMessage(messages.ruleScreenshots),
		paths: 'screenshots',
		enabled: false,
	},
]

const cleanupRules = useStorage<CleanupRule[]>('maintenance-cleanup-rules', defaultCleanupRules)
const hasInstances = computed(() => instances.value.length > 0)
const totalManagedSize = computed(() =>
	Object.values(sizeByPath.value).reduce((total, size) => total + size, 0),
)
const selectedCleanupSize = computed(() =>
	cleanupTargets.value.reduce(
		(total, item) => total + (cleanupSelections.value[item.fullPath] ? item.size : 0),
		0,
	),
)
const selectedCleanupCount = computed(
	() => cleanupTargets.value.filter((item) => cleanupSelections.value[item.fullPath]).length,
)
const sortedInstances = computed(() =>
	[...instances.value].sort(
		(a, b) => (sizeByPath.value[b.path] ?? 0) - (sizeByPath.value[a.path] ?? 0),
	),
)

function formatLastPlayed(instance: GameInstance): string {
	if (!instance.last_played) return formatMessage(messages.never)
	return dayjs(instance.last_played).fromNow()
}

function joinPath(base: string, child: string): string {
	const sep = base.includes('\\') ? '\\' : '/'
	return base.replace(/[\\/]+$/, '') + sep + child.replace(/^[\\/]+/, '')
}

function normalizeRulePath(input: string): string | null {
	const normalized = input
		.trim()
		.replace(/\\/g, '/')
		.replace(/^\.\/+/, '')
		.replace(/\/{2,}/g, '/')
		.replace(/\/+$/, '')

	if (!normalized || normalized === '.') return null
	if (normalized.startsWith('/')) return null
	if (/^[a-zA-Z]:\//.test(normalized)) return null
	if (normalized.split('/').some((part) => part === '..')) return null

	return normalized
}

function normalizeRules() {
	cleanupRules.value = cleanupRules.value.map((rule) => ({
		...rule,
		id: rule.id || `rule-${Math.random().toString(36).slice(2, 9)}`,
	}))
}

async function loadInstances() {
	loadingInstances.value = true
	try {
		instances.value = (await list().catch(handleError)) ?? []
	} finally {
		loadingInstances.value = false
	}
}

function selectAllCleanup() {
	for (const item of cleanupTargets.value) {
		cleanupSelections.value[item.fullPath] = true
	}
}

function clearCleanupSelection() {
	for (const item of cleanupTargets.value) {
		cleanupSelections.value[item.fullPath] = false
	}
}

async function scanInstanceSizes() {
	if (!instances.value.length) return
	scanningSizes.value = true
	try {
		const results: Record<string, number> = {}
		await Promise.all(
			instances.value.map(async (instance) => {
				const fullPath = await get_full_path(instance.path).catch(handleError)
				if (!fullPath) return
				const size = await invoke<number>('plugin:utils|get_dir_size', { path: fullPath })
				results[instance.path] = size
			}),
		)
		sizeByPath.value = results
	} finally {
		scanningSizes.value = false
	}
}

function addRule() {
	cleanupRules.value.push({
		id: `rule-${Date.now()}`,
		label: formatMessage(messages.ruleNew),
		paths: '',
		enabled: true,
	})
}

function removeRule(ruleId: string) {
	cleanupRules.value = cleanupRules.value.filter((rule) => rule.id !== ruleId)
}

async function openInstanceFolder(instance: GameInstance) {
	const fullPath = await get_full_path(instance.path).catch(handleError)
	if (!fullPath) return
	await openPath(fullPath).catch(() =>
		handleError(new Error(formatMessage(messages.failedOpenFolder))),
	)
}

function confirmDelete(instance: GameInstance) {
	selectedForDelete.value = instance
	deleteConfirmModal.value?.show()
}

async function deleteInstance() {
	if (!selectedForDelete.value) return
	await remove(selectedForDelete.value.path).catch(handleError)
	selectedForDelete.value = null
	await loadInstances()
	await scanInstanceSizes()
}

async function openCleanup(instancePath: string | null) {
	if (!instancePath) return
	selectedInstancePath.value = instancePath
	cleanupModal.value?.show()
	await analyzeCleanup(instancePath)
}

async function analyzeCleanup(instancePath: string) {
	cleanupLoading.value = true
	cleanupTargets.value = []
	cleanupSelections.value = {}
	try {
		const instance = instances.value.find((item) => item.path === instancePath)
		if (!instance) return
		const fullPath = await get_full_path(instance.path).catch(handleError)
		if (!fullPath) return

		const targetMap = new Map<string, CleanupTarget>()
		const nextSelections: Record<string, boolean> = {}
		for (const rule of cleanupRules.value) {
			if (!rule.paths) continue
			const parts = Array.from(
				new Set(
					rule.paths
						.split(',')
						.map((part) => normalizeRulePath(part))
						.filter((part): part is string => Boolean(part)),
				),
			)
			for (const part of parts) {
				const fullTarget = joinPath(fullPath, part)
				const size = await invoke<number>('plugin:utils|get_dir_size', { path: fullTarget }).catch(
					() => 0,
				)
				if (!Number.isFinite(size) || size <= 0) continue

				const existing = targetMap.get(fullTarget)
				if (existing) {
					nextSelections[fullTarget] = nextSelections[fullTarget] || rule.enabled
					continue
				}

				targetMap.set(fullTarget, {
					ruleId: rule.id,
					label: rule.label,
					relativePath: part,
					fullPath: fullTarget,
					size,
				})
				nextSelections[fullTarget] = rule.enabled
			}
		}
		cleanupTargets.value = Array.from(targetMap.values()).sort((a, b) => b.size - a.size)
		cleanupSelections.value = nextSelections
	} finally {
		cleanupLoading.value = false
	}
}

async function applyCleanup() {
	const selectedPaths = cleanupTargets.value
		.filter((item) => cleanupSelections.value[item.fullPath])
		.map((item) => item.fullPath)
	if (!selectedPaths.length) return
	const removed = await invoke<number>('plugin:utils|delete_paths', { paths: selectedPaths })
	addNotification({
		title: formatMessage(messages.cleanupCompleteTitle),
		text: formatMessage(messages.cleanupCompleteText, { size: formatBytes(removed) }),
		type: 'success',
	})
	if (selectedInstancePath.value) {
		await scanInstanceSizes()
		await analyzeCleanup(selectedInstancePath.value)
	}
}

onMounted(async () => {
	normalizeRules()
	await loadInstances()
	await scanInstanceSizes()
})

interface CleanupRule {
	id: string
	label: string
	paths: string
	enabled: boolean
}

interface CleanupTarget {
	ruleId: string
	label: string
	relativePath: string
	fullPath: string
	size: number
}
</script>

<template>
	<div class="p-6 flex flex-col gap-6">
		<Card class="section-card">
			<div class="flex items-center justify-between gap-4">
				<div>
					<h2 class="text-xl font-extrabold m-0">{{ formatMessage(messages.sectionTitle) }}</h2>
					<p class="m-0 text-secondary">{{ formatMessage(messages.sectionDescription) }}</p>
				</div>
				<ButtonStyled color="brand">
					<button :disabled="scanningSizes" @click="scanInstanceSizes">
						<UpdatedIcon />
						{{ scanningSizes ? formatMessage(messages.scanning) : formatMessage(messages.rescan) }}
					</button>
				</ButtonStyled>
			</div>
			<div class="summary-row">
				<div class="summary-card">
					<span class="label">{{ formatMessage(messages.totalInstances) }}</span>
					<span class="value">{{ instances.length }}</span>
				</div>
				<div class="summary-card">
					<span class="label">{{ formatMessage(messages.totalSize) }}</span>
					<span class="value">{{ formatBytes(totalManagedSize) }}</span>
				</div>
			</div>
			<div v-if="loadingInstances" class="empty-state">
				{{ formatMessage(messages.loadingInstances) }}
			</div>
			<div v-else-if="!hasInstances" class="empty-state">
				{{ formatMessage(messages.noInstances) }}
			</div>
			<div v-else class="instance-grid">
				<div v-for="instance in sortedInstances" :key="instance.path" class="instance-card">
					<div class="instance-header">
						<h3 class="m-0 text-lg text-contrast">{{ instance.name }}</h3>
						<span class="text-xs text-secondary">{{ formatLastPlayed(instance) }}</span>
					</div>
					<div class="instance-meta">
						<span class="chip">{{ instance.loader }} {{ instance.game_version }}</span>
						<span class="chip">{{ formatBytes(sizeByPath[instance.path] ?? 0) }}</span>
					</div>
					<div class="instance-actions">
						<ButtonStyled>
							<button @click="openCleanup(instance.path)">
								{{ formatMessage(messages.clean) }}
							</button>
						</ButtonStyled>
						<ButtonStyled type="transparent">
							<button @click="openInstanceFolder(instance)">
								<FolderOpenIcon /> {{ formatMessage(messages.folder) }}
							</button>
						</ButtonStyled>
						<ButtonStyled color="red" type="transparent">
							<button @click="confirmDelete(instance)">
								<TrashIcon /> {{ formatMessage(messages.delete) }}
							</button>
						</ButtonStyled>
					</div>
				</div>
			</div>
		</Card>

		<Card class="section-card">
			<div class="flex items-start justify-between gap-4">
				<div>
					<h2 class="text-xl font-extrabold m-0">{{ formatMessage(messages.rulesTitle) }}</h2>
					<p class="m-0 text-secondary">
						{{ formatMessage(messages.rulesDescription) }}
					</p>
				</div>
				<ButtonStyled color="brand"
					><button @click="addRule">{{ formatMessage(messages.addRule) }}</button></ButtonStyled
				>
			</div>
			<div class="rules-grid">
				<div v-for="rule in cleanupRules" :key="rule.id" class="rule-card">
					<div class="rule-header">
						<Checkbox v-model="rule.enabled" />
						<input
							v-model="rule.label"
							class="rule-input"
							:placeholder="formatMessage(messages.ruleNamePlaceholder)"
						/>
						<Button icon-only @click="removeRule(rule.id)">
							<TrashIcon />
						</Button>
					</div>
					<input
						v-model="rule.paths"
						class="rule-input full"
						:placeholder="formatMessage(messages.rulePathsPlaceholder)"
					/>
				</div>
			</div>
		</Card>
	</div>
	<ConfirmModalWrapper
		ref="deleteConfirmModal"
		:title="formatMessage(messages.deleteInstanceTitle, { name: selectedForDelete?.name ?? '' })"
		:description="formatMessage(messages.deleteInstanceDescription)"
		@proceed="deleteInstance"
	/>

	<ModalWrapper ref="cleanupModal" :header="formatMessage(messages.cleanupModalHeader)">
		<div class="modal-content">
			<div class="modal-header">
				<div class="modal-header-title">
					<h3 class="m-0 text-lg">{{ formatMessage(messages.safeToRemove) }}</h3>
					<span class="text-secondary text-sm">
						{{ formatMessage(messages.selectedCount, { count: selectedCleanupCount }) }}
					</span>
				</div>
				<span class="text-secondary">{{ formatBytes(selectedCleanupSize) }}</span>
			</div>
			<div class="modal-tools">
				<ButtonStyled type="transparent">
					<button @click="selectAllCleanup">{{ formatMessage(messages.selectAll) }}</button>
				</ButtonStyled>
				<ButtonStyled type="transparent">
					<button @click="clearCleanupSelection">
						{{ formatMessage(messages.clearSelection) }}
					</button>
				</ButtonStyled>
			</div>
			<div v-if="cleanupLoading" class="empty-state">{{ formatMessage(messages.scanning) }}</div>
			<div v-else-if="cleanupTargets.length === 0" class="empty-state">
				{{ formatMessage(messages.noCleanupTargets) }}
			</div>
			<div v-else class="cleanup-list">
				<div v-for="item in cleanupTargets" :key="item.fullPath" class="cleanup-row">
					<Checkbox v-model="cleanupSelections[item.fullPath]" />
					<div>
						<p class="m-0 text-contrast">{{ item.label }}</p>
						<p class="m-0 text-xs text-secondary">{{ item.relativePath }}</p>
					</div>
					<span class="text-sm">{{ formatBytes(item.size) }}</span>
				</div>
			</div>
			<div class="modal-actions">
				<ButtonStyled color="brand"
					><button @click="applyCleanup">{{ formatMessage(messages.clean) }}</button></ButtonStyled
				>
			</div>
		</div>
	</ModalWrapper>
</template>

<style scoped lang="scss">
.section-card {
	padding: 1.5rem;
	display: flex;
	flex-direction: column;
	gap: 1.5rem;
	background: var(--color-glass-bg-strong);
	border: 1px solid var(--glass-border);
	box-shadow: var(--glass-shadow);
}

.summary-row {
	display: grid;
	grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
	gap: 1rem;
}

.summary-card {
	background: var(--color-glass-bg);
	border: 1px solid var(--glass-border);
	border-radius: var(--radius-lg);
	padding: 1rem;
	display: flex;
	flex-direction: column;
	gap: 0.35rem;
	box-shadow: var(--shadow-card);
}

.summary-card .label {
	font-size: 0.75rem;
	text-transform: uppercase;
	letter-spacing: 0.06em;
	color: var(--color-secondary);
}

.summary-card .value {
	font-size: 1.4rem;
	font-weight: 700;
	color: var(--color-contrast);
}

.instance-grid {
	display: grid;
	grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
	gap: 1rem;
}

.instance-card {
	padding: 1rem;
	border-radius: var(--radius-lg);
	background: var(--color-glass-bg);
	border: 1px solid var(--glass-border);
	display: flex;
	flex-direction: column;
	gap: 0.75rem;
	box-shadow: var(--shadow-card);
	transition:
		transform 180ms ease,
		box-shadow 180ms ease,
		border-color 180ms ease;
}

.instance-card:hover {
	transform: translateY(-2px);
	box-shadow: var(--shadow-floating);
	border-color: var(--color-brand-highlight);
}

.instance-header {
	display: flex;
	align-items: center;
	justify-content: space-between;
}

.instance-meta {
	display: flex;
	gap: 0.5rem;
	flex-wrap: wrap;
}

.chip {
	padding: 0.25rem 0.5rem;
	border-radius: 999px;
	background: var(--color-button-bg);
	font-size: 0.75rem;
	color: var(--color-secondary);
}

.instance-actions {
	display: flex;
	gap: 0.5rem;
	flex-wrap: wrap;
}

.rules-grid {
	display: grid;
	grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
	gap: 1rem;
}

.rule-card {
	background: var(--color-glass-bg);
	border: 1px solid var(--glass-border);
	border-radius: var(--radius-lg);
	padding: 1rem;
	display: flex;
	flex-direction: column;
	gap: 0.75rem;
	box-shadow: var(--shadow-card);
}

.rule-header {
	display: flex;
	align-items: center;
	gap: 0.5rem;
}

.rule-input {
	width: 100%;
	background: var(--color-super-raised-bg);
	border: 1px solid var(--glass-border);
	border-radius: 0.75rem;
	padding: 0.5rem 0.75rem;
	color: var(--color-contrast);
	transition:
		border-color 150ms ease,
		box-shadow 150ms ease;
}

.rule-input:focus {
	outline: none;
	border-color: var(--color-brand);
	box-shadow: 0 0 0 2px var(--color-brand-highlight);
}

.rule-input.full {
	width: 100%;
}

.empty-state {
	padding: 1rem;
	border-radius: var(--radius-lg);
	background: var(--color-button-bg);
	color: var(--color-secondary);
	text-align: center;
}

.modal-content {
	display: flex;
	flex-direction: column;
	gap: 1rem;
}

.modal-header {
	display: flex;
	justify-content: space-between;
	align-items: center;
}

.modal-header-title {
	display: flex;
	flex-direction: column;
	gap: 0.15rem;
}

.modal-tools {
	display: flex;
	gap: 0.5rem;
}

.cleanup-list {
	display: flex;
	flex-direction: column;
	gap: 0.75rem;
	max-height: 320px;
	overflow: auto;
}

.cleanup-row {
	display: grid;
	grid-template-columns: auto 1fr auto;
	gap: 0.75rem;
	align-items: center;
	padding: 0.6rem 0.75rem;
	border-radius: var(--radius-md);
	background: var(--color-button-bg);
	border: 1px solid var(--color-button-border);
}

.modal-actions {
	display: flex;
	justify-content: flex-end;
}
</style>
