<template>
	<div class="space-y-6">
		<!-- Instance-specific: Icon upload -->
		<div v-if="ctx.flowType === 'instance'" class="flex items-center gap-4">
			<Avatar :src="ctx.instanceIconUrl.value ?? undefined" size="5rem" />
			<div class="flex flex-col gap-2">
				<ButtonStyled type="outlined">
					<button class="!border-[--glass-border]" @click="triggerIconInput">
						<UploadIcon />
						{{ formatMessage(messages.selectIcon) }}
					</button>
				</ButtonStyled>
				<ButtonStyled type="outlined">
					<button class="!border-[--glass-border]" :disabled="!ctx.instanceIcon.value" @click="removeIcon">
						<XIcon />
						{{ formatMessage(messages.removeIcon) }}
					</button>
				</ButtonStyled>
			</div>
		</div>

		<!-- Instance-specific: Name field -->
		<div v-if="ctx.flowType === 'instance'" class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">{{ formatMessage(messages.nameLabel) }}</span>
			<StyledInput
				v-model="ctx.instanceName.value"
				:placeholder="ctx.autoInstanceName.value || formatMessage(messages.namePlaceholder)"
			/>
		</div>

		<!-- Loader chips -->
		<div v-if="!hideLoaderChips" class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">{{
				ctx.flowType === 'instance'
					? formatMessage(messages.loaderLabelInstance)
					: formatMessage(messages.loaderLabelContent)
			}}</span>
			<Chips
				v-model="selectedLoader"
				:items="effectiveLoaders"
				:format-label="formatLoaderLabel"
				:never-empty="false"
			/>
		</div>

		<!-- Game version -->
		<div class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">{{ formatMessage(messages.gameVersionLabel) }}</span>
			<Combobox
				v-model="selectedGameVersion"
				:options="gameVersionOptions"
				searchable
				sync-with-selection
				:placeholder="formatMessage(messages.gameVersionPlaceholder)"
				:search-placeholder="formatMessage(messages.gameVersionSearch)"
			>
				<template v-if="ctx.showSnapshotToggle" #dropdown-footer>
					<button
						class="flex w-full cursor-pointer items-center justify-center gap-1.5 border-0 border-t border-solid border-[--glass-border] bg-transparent py-3 text-center text-sm font-semibold text-secondary transition-colors hover:text-contrast"
						@mousedown.prevent
						@click="ctx.showSnapshots.value = !ctx.showSnapshots.value"
					>
						<EyeOffIcon v-if="ctx.showSnapshots.value" class="size-4" />
						<EyeIcon v-else class="size-4" />
						{{
							ctx.showSnapshots.value
								? formatMessage(messages.hideSnapshots)
								: formatMessage(messages.showAllVersions)
						}}
					</button>
				</template>
			</Combobox>
		</div>

		<!-- Loader version -->
		<template v-if="!hideLoaderVersion">
			<Collapsible :collapsed="!selectedLoader || !selectedGameVersion" overflow-visible>
				<div class="flex flex-col gap-2">
					<span class="font-semibold text-contrast">{{
						isPaperLike
							? formatMessage(messages.buildNumberLabel)
							: formatMessage(messages.loaderVersionLabel)
					}}</span>
					<Chips
						v-if="!isPaperLike"
						v-model="loaderVersionType"
						:items="loaderVersionTypeItems"
						:format-label="formatLoaderVersionType"
					/>
					<div v-if="isPaperLike || loaderVersionType === 'other'">
						<Combobox
							v-model="selectedLoaderVersion"
							:options="loaderVersionOptions"
							:no-options-message="
								loaderVersionsLoading
									? formatMessage(messages.loaderVersionsLoading)
									: formatMessage(messages.loaderVersionsEmpty)
							"
							searchable
							sync-with-selection
							:placeholder="
								isPaperLike
									? formatMessage(messages.selectBuildNumber)
									: formatMessage(messages.selectLoaderVersion)
							"
							:search-placeholder="
								isPaperLike
									? formatMessage(messages.searchBuildNumber)
									: formatMessage(messages.searchLoaderVersion)
							"
						/>
					</div>
				</div>
			</Collapsible>
		</template>
	</div>
</template>

<script setup lang="ts">
import { EyeIcon, EyeOffIcon, UploadIcon, XIcon } from '@modrinth/assets'
import { defineMessages } from '@vintl/vintl'
import { computed, onMounted, ref, watch } from 'vue'

import { useDebugLogger } from '#ui/composables/debug-logger'

import { injectFilePicker, injectTags } from '../../../../providers'
import Avatar from '../../../base/Avatar.vue'
import ButtonStyled from '../../../base/ButtonStyled.vue'
import Chips from '../../../base/Chips.vue'
import Collapsible from '../../../base/Collapsible.vue'
import Combobox, { type ComboboxOption } from '../../../base/Combobox.vue'
import StyledInput from '../../../base/StyledInput.vue'
import type { LoaderVersionType } from '../creation-flow-context'
import { injectCreationFlowContext } from '../creation-flow-context'
import { capitalize, formatLoaderLabel } from '../shared'

const debug = useDebugLogger('CustomSetupStage')
const ctx = injectCreationFlowContext()
const { formatMessage } = ctx
const {
	selectedLoader,
	selectedGameVersion,
	loaderVersionType,
	selectedLoaderVersion,
	hideLoaderChips,
	hideLoaderVersion,
} = ctx

// For instance flow, prepend 'vanilla' to available loaders.
// For server flows, vanilla is a separate option in the setup type stage, so exclude it here.
const effectiveLoaders = computed(() => {
	if (ctx.flowType === 'instance') {
		return ['vanilla', ...ctx.availableLoaders.filter((l) => l !== 'vanilla')]
	}
	if (ctx.flowType === 'server-onboarding' || ctx.flowType === 'reset-server') {
		return ctx.availableLoaders.filter((l) => l !== 'vanilla')
	}
	return ctx.availableLoaders
})

// Pre-select loader and game version from initial values
onMounted(() => {
	debug('mounted, initialLoader:', ctx.initialLoader, 'initialGameVersion:', ctx.initialGameVersion)
	if (!selectedLoader.value) {
		if (ctx.initialLoader) {
			selectedLoader.value = ctx.initialLoader
		} else {
			selectedLoader.value = 'fabric'
		}
	}
	if (ctx.initialGameVersion && !selectedGameVersion.value) {
		selectedGameVersion.value = ctx.initialGameVersion
	}
	debug('after init:', { loader: selectedLoader.value, gameVersion: selectedGameVersion.value })
})

const tags = injectTags()

const messages = defineMessages({
	selectIcon: {
		id: 'creation-flow.custom.icon.select',
		defaultMessage: 'Select icon',
	},
	removeIcon: {
		id: 'creation-flow.custom.icon.remove',
		defaultMessage: 'Remove icon',
	},
	nameLabel: {
		id: 'creation-flow.custom.name.label',
		defaultMessage: 'Name',
	},
	namePlaceholder: {
		id: 'creation-flow.custom.name.placeholder',
		defaultMessage: 'Enter instance name',
	},
	loaderLabelInstance: {
		id: 'creation-flow.custom.loader.label.instance',
		defaultMessage: 'Loader',
	},
	loaderLabelContent: {
		id: 'creation-flow.custom.loader.label.content',
		defaultMessage: 'Content loader',
	},
	gameVersionLabel: {
		id: 'creation-flow.custom.game-version.label',
		defaultMessage: 'Game version',
	},
	gameVersionPlaceholder: {
		id: 'creation-flow.custom.game-version.placeholder',
		defaultMessage: 'Select game version',
	},
	gameVersionSearch: {
		id: 'creation-flow.custom.game-version.search',
		defaultMessage: 'Search game version...',
	},
	hideSnapshots: {
		id: 'creation-flow.custom.snapshots.hide',
		defaultMessage: 'Hide snapshots',
	},
	showAllVersions: {
		id: 'creation-flow.custom.snapshots.show',
		defaultMessage: 'Show all versions',
	},
	buildNumberLabel: {
		id: 'creation-flow.custom.build-number.label',
		defaultMessage: 'Build number',
	},
	loaderVersionLabel: {
		id: 'creation-flow.custom.loader-version.label',
		defaultMessage: 'Loader version',
	},
	loaderVersionStable: {
		id: 'creation-flow.custom.loader-version.stable',
		defaultMessage: 'Stable',
	},
	loaderVersionLatest: {
		id: 'creation-flow.custom.loader-version.latest',
		defaultMessage: 'Latest',
	},
	loaderVersionOther: {
		id: 'creation-flow.custom.loader-version.other',
		defaultMessage: 'Other',
	},
	loaderVersionsLoading: {
		id: 'creation-flow.common.loading',
		defaultMessage: 'Loading...',
	},
	loaderVersionsEmpty: {
		id: 'creation-flow.custom.loader-version.none',
		defaultMessage: 'No versions available',
	},
	selectBuildNumber: {
		id: 'creation-flow.custom.build-number.placeholder',
		defaultMessage: 'Select build number',
	},
	selectLoaderVersion: {
		id: 'creation-flow.custom.loader-version.placeholder',
		defaultMessage: 'Select loader version',
	},
	searchBuildNumber: {
		id: 'creation-flow.custom.build-number.search',
		defaultMessage: 'Search build number...',
	},
	searchLoaderVersion: {
		id: 'creation-flow.custom.loader-version.search',
		defaultMessage: 'Search loader version...',
	},
	buildLabel: {
		id: 'creation-flow.custom.build-number.label-value',
		defaultMessage: 'Build {build}',
	},
	stableSuffix: {
		id: 'creation-flow.custom.loader-version.stable-suffix',
		defaultMessage: '{version} (stable)',
	},
})

const loaderVersionTypeItems: LoaderVersionType[] = ['stable', 'latest', 'other']

function formatLoaderVersionType(value: LoaderVersionType) {
	switch (value) {
		case 'stable':
			return formatMessage(messages.loaderVersionStable)
		case 'latest':
			return formatMessage(messages.loaderVersionLatest)
		case 'other':
			return formatMessage(messages.loaderVersionOther)
		default:
			return capitalize(value)
	}
}

const isPaperLike = computed(
	() => selectedLoader.value === 'paper' || selectedLoader.value === 'purpur',
)

// Icon upload handling
const filePicker = injectFilePicker()

async function triggerIconInput() {
	const picked = await filePicker.pickImage()
	if (picked) {
		ctx.instanceIcon.value = picked.file
		ctx.instanceIconUrl.value = picked.previewUrl
		ctx.instanceIconPath.value = picked.path ?? null
	}
}

function removeIcon() {
	ctx.instanceIcon.value = null
	ctx.instanceIconUrl.value = null
	ctx.instanceIconPath.value = null
}

// Loader versions fetched from launcher-meta
interface LoaderVersionEntry {
	id: string
	stable: boolean
}

const loaderVersionsLoading = ref(false)
const loaderVersionsData = ref<LoaderVersionEntry[]>([])
const loaderVersionsCache = ref<Record<string, { id: string; loaders: LoaderVersionEntry[] }[]>>({})

// Paper/Purpur build caches
const paperVersions = ref<Record<string, number[]>>({})
const purpurVersions = ref<Record<string, string[]>>({})

// Paper/Purpur supported game version sets (for filtering the game version combobox)
const paperSupportedVersions = ref<Set<string> | null>(null)
const purpurSupportedVersions = ref<Set<string> | null>(null)

// Game versions from tags provider, filtered by loader support
const gameVersionOptions = computed<ComboboxOption<string>[]>(() => {
	const versions = ctx.showSnapshots.value
		? tags.gameVersions.value
		: tags.gameVersions.value.filter((v) => v.version_type === 'release')

	// For loaders with per-version data, only show game versions that have builds
	if (selectedLoader.value && selectedLoader.value !== 'vanilla') {
		if (selectedLoader.value === 'paper' && paperSupportedVersions.value) {
			return versions
				.filter((v) => paperSupportedVersions.value!.has(v.version))
				.map((v) => ({ value: v.version, label: v.version }))
		}

		if (selectedLoader.value === 'purpur' && purpurSupportedVersions.value) {
			return versions
				.filter((v) => purpurSupportedVersions.value!.has(v.version))
				.map((v) => ({ value: v.version, label: v.version }))
		}

		let apiLoader = selectedLoader.value
		if (apiLoader === 'neoforge') apiLoader = 'neo'

		const manifest = loaderVersionsCache.value[apiLoader]
		if (manifest) {
			const hasPlaceholder = manifest.some((x) => x.id === '${modrinth.gameVersion}')
			if (!hasPlaceholder) {
				const supportedVersions = new Set(
					manifest.filter((x) => x.loaders.length > 0).map((x) => x.id),
				)
				return versions
					.filter((v) => supportedVersions.has(v.version))
					.map((v) => ({ value: v.version, label: v.version }))
			}
		}
	}

	return versions.map((v) => ({ value: v.version, label: v.version }))
})

// Auto-select latest game version when options change and current selection is missing or invalid
watch(
	gameVersionOptions,
	(options) => {
		if (options.length === 0) return
		if (!selectedGameVersion.value || !options.some((o) => o.value === selectedGameVersion.value)) {
			selectedGameVersion.value = options[0].value
		}
	},
	{ immediate: true },
)

async function fetchLoaderManifest(loader: string) {
	let apiLoader = loader
	if (apiLoader === 'neoforge') apiLoader = 'neo'

	if (loaderVersionsCache.value[apiLoader]) return

	try {
		const res = await fetch(`https://launcher-meta.modrinth.com/${apiLoader}/v0/manifest.json`)
		const data = (await res.json()) as {
			gameVersions: { id: string; loaders: LoaderVersionEntry[] }[]
		}
		loaderVersionsCache.value[apiLoader] = data.gameVersions
	} catch {
		loaderVersionsCache.value[apiLoader] = []
	}
}

async function fetchPaperSupportedVersions() {
	if (paperSupportedVersions.value) return
	try {
		const res = await fetch('https://api.papermc.io/v2/projects/paper')
		const data = (await res.json()) as { versions: string[] }
		paperSupportedVersions.value = new Set(data.versions)
	} catch {
		paperSupportedVersions.value = new Set()
	}
}

async function fetchPurpurSupportedVersions() {
	if (purpurSupportedVersions.value) return
	try {
		const res = await fetch('https://api.purpurmc.org/v2/purpur')
		const data = (await res.json()) as { versions: string[] }
		purpurSupportedVersions.value = new Set(data.versions)
	} catch {
		purpurSupportedVersions.value = new Set()
	}
}

async function fetchPaperVersions(mcVersion: string) {
	if (paperVersions.value[mcVersion]) return
	try {
		const res = await fetch(`https://fill.papermc.io/v3/projects/paper/versions/${mcVersion}`)
		const data = (await res.json()) as { builds: number[] }
		paperVersions.value[mcVersion] = data.builds.sort((a, b) => b - a)
	} catch {
		paperVersions.value[mcVersion] = []
	}
}

async function fetchPurpurVersions(mcVersion: string) {
	if (purpurVersions.value[mcVersion]) return
	try {
		const res = await fetch(`https://api.purpurmc.org/v2/purpur/${mcVersion}`)
		const data = (await res.json()) as { builds: { all: string[] } }
		purpurVersions.value[mcVersion] = data.builds.all.sort((a, b) => parseInt(b) - parseInt(a))
	} catch {
		purpurVersions.value[mcVersion] = []
	}
}

function getLoaderVersionsForGameVersion(
	loader: string,
	gameVersion: string,
): LoaderVersionEntry[] {
	let apiLoader = loader
	if (apiLoader === 'neoforge') apiLoader = 'neo'

	const manifest = loaderVersionsCache.value[apiLoader]
	if (!manifest) return []

	// Some loaders (e.g. Fabric) list all versions under a placeholder entry
	const placeholder = manifest.find((x) => x.id === '${modrinth.gameVersion}')
	if (placeholder) return placeholder.loaders

	const entry = manifest.find((x) => x.id === gameVersion)
	return entry?.loaders ?? []
}

// Fetch version data when loader changes so game versions can be filtered
watch(
	() => selectedLoader.value,
	async (loader) => {
		if (!loader || loader === 'vanilla') return
		if (loader === 'paper') {
			await fetchPaperSupportedVersions()
			return
		}
		if (loader === 'purpur') {
			await fetchPurpurSupportedVersions()
			return
		}
		await fetchLoaderManifest(loader)
	},
	{ immediate: true },
)

// Watch loader + game version to resolve loader versions
watch(
	[() => selectedLoader.value, () => selectedGameVersion.value],
	async ([loader, gameVersion]) => {
		loaderVersionsData.value = []
		selectedLoaderVersion.value = null

		if (!loader || !gameVersion || loader === 'vanilla') return

		loaderVersionsLoading.value = true

		if (loader === 'paper') {
			await fetchPaperVersions(gameVersion)
			loaderVersionsLoading.value = false
			// Auto-select latest build
			const builds = paperVersions.value[gameVersion]
			if (builds?.length) {
				selectedLoaderVersion.value = `${builds[0]}`
			}
			return
		}

		if (loader === 'purpur') {
			await fetchPurpurVersions(gameVersion)
			loaderVersionsLoading.value = false
			// Auto-select latest build
			const builds = purpurVersions.value[gameVersion]
			if (builds?.length) {
				selectedLoaderVersion.value = builds[0]
			}
			return
		}

		await fetchLoaderManifest(loader)
		loaderVersionsData.value = getLoaderVersionsForGameVersion(loader, gameVersion)
		loaderVersionsLoading.value = false

		// Auto-select based on loaderVersionType
		autoSelectLoaderVersion()
	},
)

watch(
	() => loaderVersionType.value,
	() => autoSelectLoaderVersion(),
)

function autoSelectLoaderVersion() {
	if (loaderVersionType.value === 'stable') {
		const stable = loaderVersionsData.value.find((v) => v.stable)
		selectedLoaderVersion.value = stable?.id ?? loaderVersionsData.value[0]?.id ?? null
	} else if (loaderVersionType.value === 'latest') {
		selectedLoaderVersion.value = loaderVersionsData.value[0]?.id ?? null
	} else if (loaderVersionType.value === 'other' && !selectedLoaderVersion.value) {
		selectedLoaderVersion.value = loaderVersionsData.value[0]?.id ?? null
	}
	debug('autoSelectLoaderVersion:', selectedLoaderVersion.value, 'type:', loaderVersionType.value)
}

const loaderVersionOptions = computed<ComboboxOption<string>[]>(() => {
	if (selectedLoader.value === 'paper' && selectedGameVersion.value) {
		const builds = paperVersions.value[selectedGameVersion.value] ?? []
		return builds.map((b) => ({
			value: `${b}`,
			label: formatMessage(messages.buildLabel, { build: b }),
		}))
	}

	if (selectedLoader.value === 'purpur' && selectedGameVersion.value) {
		const builds = purpurVersions.value[selectedGameVersion.value] ?? []
		return builds.map((b) => ({
			value: b,
			label: formatMessage(messages.buildLabel, { build: b }),
		}))
	}

	return loaderVersionsData.value.map((v) => ({
		value: v.id,
		label: v.stable
			? formatMessage(messages.stableSuffix, { version: v.id })
			: v.id,
	}))
})
</script>
