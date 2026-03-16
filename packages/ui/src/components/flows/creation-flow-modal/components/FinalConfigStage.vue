<template>
	<div class="space-y-6">
		<div
			v-if="ctx.flowType !== 'server-onboarding' && ctx.flowType !== 'reset-server'"
			class="flex flex-col gap-2"
		>
			<span class="font-semibold text-contrast">{{ formatMessage(messages.worldNameLabel) }}</span>
			<StyledInput
				v-model="worldName"
				:placeholder="formatMessage(messages.worldNamePlaceholder)"
			/>
		</div>

		<div v-if="ctx.setupType.value === 'vanilla'" class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">{{ formatMessage(messages.gameVersionLabel) }}</span>
			<Combobox
				v-model="selectedGameVersion"
				:options="gameVersionOptions"
				searchable
				sync-with-selection
				:placeholder="formatMessage(messages.gameVersionPlaceholder)"
			>
				<template v-if="ctx.showSnapshotToggle" #dropdown-footer>
					<button
						class="flex w-full cursor-pointer items-center justify-center gap-1.5 border-0 border-t border-solid border-surface-5 bg-transparent py-3 text-center text-sm font-semibold text-secondary transition-colors hover:text-contrast"
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

		<div class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">{{ formatMessage(messages.gamemodeLabel) }}</span>
			<Chips v-model="gamemode" :items="gamemodeItems" :format-label="formatGamemode" />
		</div>

		<div v-if="gamemode !== 'hardcore'" class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">
				{{ formatMessage(messages.difficultyLabel) }}
			</span>
			<Chips v-model="difficulty" :items="difficultyItems" :format-label="formatDifficulty" />
		</div>

		<div class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">{{ formatMessage(messages.worldTypeLabel) }}</span>
			<Combobox
				v-model="worldTypeOption"
				:options="worldTypeOptions"
				:placeholder="formatMessage(messages.worldTypePlaceholder)"
			/>
		</div>

		<div class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">
				{{ formatMessage(messages.worldSeedLabel) }}
				<span class="text-secondary font-normal">
					({{ formatMessage(messages.optional) }})
				</span>
			</span>
			<StyledInput v-model="worldSeed" :placeholder="formatMessage(messages.worldSeedPlaceholder)" />
			<span class="text-sm text-secondary">{{ formatMessage(messages.worldSeedHint) }}</span>
		</div>

		<div class="h-px w-full bg-surface-5" />

		<Accordion overflow-visible button-class="w-full bg-transparent m-0 p-0 border-none">
			<template #title>
				<SettingsIcon class="size-4 shrink-0 text-primary" />
				<span class="font-semibold text-contrast text-lg">
					{{ formatMessage(messages.additionalSettings) }}
				</span>
			</template>
			<div class="flex flex-col gap-4 pt-4">
				<div class="flex w-full flex-row items-center justify-between gap-4">
					<div class="flex flex-col gap-1">
						<span class="font-semibold text-contrast">
							{{ formatMessage(messages.generateStructuresTitle) }}
						</span>
						<span class="text-sm text-secondary">
							{{ formatMessage(messages.generateStructuresDesc) }}
						</span>
					</div>
					<Toggle v-model="generateStructures" small class="shrink-0" />
				</div>

				<div class="flex flex-col gap-2">
					<span class="font-semibold text-contrast">
						{{ formatMessage(messages.generatorSettingsLabel) }}
					</span>
					<Combobox
						v-model="generatorSettingsMode"
						:options="generatorSettingsOptions"
						:placeholder="formatMessage(messages.generatorSettingsPlaceholder)"
					/>
					<StyledInput
						v-if="generatorSettingsMode === 'custom'"
						v-model="generatorSettingsCustom"
						multiline
						:rows="4"
						:placeholder="formatMessage(messages.generatorSettingsInputPlaceholder)"
						input-class="font-mono"
					/>
					<span class="text-sm text-secondary">
						{{ formatMessage(messages.generatorSettingsHint) }}
					</span>
				</div>
			</div>
		</Accordion>
	</div>
</template>

<script setup lang="ts">
import { EyeIcon, EyeOffIcon, SettingsIcon } from '@modrinth/assets'
import { defineMessages } from '@vintl/vintl'
import { computed, watch } from 'vue'

import { useDebugLogger } from '#ui/composables/debug-logger'

import { injectTags } from '../../../../providers'
import Accordion from '../../../base/Accordion.vue'
import Chips from '../../../base/Chips.vue'
import Combobox, { type ComboboxOption } from '../../../base/Combobox.vue'
import StyledInput from '../../../base/StyledInput.vue'
import Toggle from '../../../base/Toggle.vue'
import type { Difficulty, Gamemode, GeneratorSettingsMode } from '../creation-flow-context'
import { injectCreationFlowContext } from '../creation-flow-context'
import { capitalize } from '../shared'

const debug = useDebugLogger('FinalConfigStage')
const ctx = injectCreationFlowContext()
const { formatMessage } = ctx
const {
	worldName,
	gamemode,
	difficulty,
	worldTypeOption,
	worldSeed,
	generateStructures,
	generatorSettingsMode,
	generatorSettingsCustom,
	selectedGameVersion,
} = ctx

const messages = defineMessages({
	worldNameLabel: {
		id: 'creation-flow.final.world-name.label',
		defaultMessage: 'World name',
	},
	worldNamePlaceholder: {
		id: 'creation-flow.final.world-name.placeholder',
		defaultMessage: 'Enter world name',
	},
	gameVersionLabel: {
		id: 'creation-flow.final.game-version.label',
		defaultMessage: 'Game version',
	},
	gameVersionPlaceholder: {
		id: 'creation-flow.final.game-version.placeholder',
		defaultMessage: 'Select game version',
	},
	hideSnapshots: {
		id: 'creation-flow.custom.snapshots.hide',
		defaultMessage: 'Hide snapshots',
	},
	showAllVersions: {
		id: 'creation-flow.custom.snapshots.show',
		defaultMessage: 'Show all versions',
	},
	gamemodeLabel: {
		id: 'creation-flow.final.gamemode.label',
		defaultMessage: 'Gamemode',
	},
	difficultyLabel: {
		id: 'creation-flow.final.difficulty.label',
		defaultMessage: 'Difficulty',
	},
	worldTypeLabel: {
		id: 'creation-flow.final.world-type.label',
		defaultMessage: 'World type',
	},
	worldTypePlaceholder: {
		id: 'creation-flow.final.world-type.placeholder',
		defaultMessage: 'Select world type',
	},
	worldSeedLabel: {
		id: 'creation-flow.final.world-seed.label',
		defaultMessage: 'World seed',
	},
	optional: {
		id: 'creation-flow.common.optional',
		defaultMessage: 'Optional',
	},
	worldSeedPlaceholder: {
		id: 'creation-flow.final.world-seed.placeholder',
		defaultMessage: 'Enter world seed',
	},
	worldSeedHint: {
		id: 'creation-flow.final.world-seed.hint',
		defaultMessage: 'Leave blank for a random seed.',
	},
	additionalSettings: {
		id: 'creation-flow.final.additional-settings',
		defaultMessage: 'Additional settings',
	},
	generateStructuresTitle: {
		id: 'creation-flow.final.generate-structures.title',
		defaultMessage: 'Generate structures',
	},
	generateStructuresDesc: {
		id: 'creation-flow.final.generate-structures.desc',
		defaultMessage:
			'Controls whether villages, strongholds, and other structures generate in new chunks.',
	},
	generatorSettingsLabel: {
		id: 'creation-flow.final.generator-settings.label',
		defaultMessage: 'Generator settings',
	},
	generatorSettingsPlaceholder: {
		id: 'creation-flow.final.generator-settings.placeholder',
		defaultMessage: 'Select generator settings',
	},
	generatorSettingsInputPlaceholder: {
		id: 'creation-flow.final.generator-settings.input',
		defaultMessage: 'Enter generator settings JSON',
	},
	generatorSettingsHint: {
		id: 'creation-flow.final.generator-settings.hint',
		defaultMessage:
			'Used for advanced world customization such as custom Superflat layers.',
	},
	gamemodeSurvival: {
		id: 'creation-flow.final.gamemode.survival',
		defaultMessage: 'Survival',
	},
	gamemodeCreative: {
		id: 'creation-flow.final.gamemode.creative',
		defaultMessage: 'Creative',
	},
	gamemodeHardcore: {
		id: 'creation-flow.final.gamemode.hardcore',
		defaultMessage: 'Hardcore',
	},
	difficultyPeaceful: {
		id: 'creation-flow.final.difficulty.peaceful',
		defaultMessage: 'Peaceful',
	},
	difficultyEasy: {
		id: 'creation-flow.final.difficulty.easy',
		defaultMessage: 'Easy',
	},
	difficultyNormal: {
		id: 'creation-flow.final.difficulty.normal',
		defaultMessage: 'Normal',
	},
	difficultyHard: {
		id: 'creation-flow.final.difficulty.hard',
		defaultMessage: 'Hard',
	},
	worldTypeDefault: {
		id: 'creation-flow.final.world-type.default',
		defaultMessage: 'Default',
	},
	worldTypeSuperflat: {
		id: 'creation-flow.final.world-type.superflat',
		defaultMessage: 'Superflat',
	},
	worldTypeLargeBiomes: {
		id: 'creation-flow.final.world-type.large-biomes',
		defaultMessage: 'Large Biomes',
	},
	worldTypeAmplified: {
		id: 'creation-flow.final.world-type.amplified',
		defaultMessage: 'Amplified',
	},
	worldTypeSingleBiome: {
		id: 'creation-flow.final.world-type.single-biome',
		defaultMessage: 'Single Biome',
	},
	generatorSettingsDefault: {
		id: 'creation-flow.final.generator-settings.default',
		defaultMessage: 'Default',
	},
	generatorSettingsFlat: {
		id: 'creation-flow.final.generator-settings.flat',
		defaultMessage: 'Flat',
	},
	generatorSettingsCustom: {
		id: 'creation-flow.final.generator-settings.custom',
		defaultMessage: 'Custom',
	},
})

debug(
	'mounted, setupType:',
	ctx.setupType.value,
	'loader:',
	ctx.selectedLoader.value,
	'gameVersion:',
	ctx.selectedGameVersion.value,
	'loaderVersion:',
	ctx.selectedLoaderVersion.value,
)

// Game version options for vanilla flow
const tags = injectTags()
const gameVersionOptions = computed<ComboboxOption<string>[]>(() => {
	const versions = ctx.showSnapshots.value
		? tags.gameVersions.value
		: tags.gameVersions.value.filter((v) => v.version_type === 'release')
	return versions.map((v) => ({ value: v.version, label: v.version }))
})

// Auto-select latest game version for vanilla
watch(
	gameVersionOptions,
	(options) => {
		if (!selectedGameVersion.value && options.length > 0) {
			selectedGameVersion.value = options[0].value
		}
	},
	{ immediate: true },
)

// Hardcore locks difficulty to hard
let previousDifficulty: Difficulty = difficulty.value
watch(gamemode, (mode) => {
	if (mode === 'hardcore') {
		previousDifficulty = difficulty.value
		difficulty.value = 'hard'
	} else {
		difficulty.value = previousDifficulty
	}
})

const gamemodeItems: Gamemode[] = ['survival', 'creative', 'hardcore']
const difficultyItems: Difficulty[] = ['peaceful', 'easy', 'normal', 'hard']

function formatGamemode(value: Gamemode) {
	switch (value) {
		case 'survival':
			return formatMessage(messages.gamemodeSurvival)
		case 'creative':
			return formatMessage(messages.gamemodeCreative)
		case 'hardcore':
			return formatMessage(messages.gamemodeHardcore)
		default:
			return capitalize(value)
	}
}

function formatDifficulty(value: Difficulty) {
	switch (value) {
		case 'peaceful':
			return formatMessage(messages.difficultyPeaceful)
		case 'easy':
			return formatMessage(messages.difficultyEasy)
		case 'normal':
			return formatMessage(messages.difficultyNormal)
		case 'hard':
			return formatMessage(messages.difficultyHard)
		default:
			return capitalize(value)
	}
}

const worldTypeOptions: ComboboxOption<string>[] = [
	{ value: 'minecraft:normal', label: formatMessage(messages.worldTypeDefault) },
	{ value: 'minecraft:flat', label: formatMessage(messages.worldTypeSuperflat) },
	{ value: 'minecraft:large_biomes', label: formatMessage(messages.worldTypeLargeBiomes) },
	{ value: 'minecraft:amplified', label: formatMessage(messages.worldTypeAmplified) },
	{ value: 'minecraft:single_biome_surface', label: formatMessage(messages.worldTypeSingleBiome) },
]

const generatorSettingsOptions: ComboboxOption<GeneratorSettingsMode>[] = [
	{ value: 'default', label: formatMessage(messages.generatorSettingsDefault) },
	{ value: 'flat', label: formatMessage(messages.generatorSettingsFlat) },
	{ value: 'custom', label: formatMessage(messages.generatorSettingsCustom) },
]
</script>
