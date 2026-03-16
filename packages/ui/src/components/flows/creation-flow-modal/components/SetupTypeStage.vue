<template>
	<div class="flex flex-col gap-4">
		<span class="font-semibold text-contrast">
			{{
				ctx.flowType === 'instance'
					? formatMessage(messages.titleInstance)
					: ctx.flowType === 'server-onboarding' || ctx.flowType === 'reset-server'
						? formatMessage(messages.titleServer)
						: formatMessage(messages.titleWorld)
			}}
		</span>

		<!-- Instance flow options -->
		<template v-if="ctx.flowType === 'instance'">
			<div class="flex flex-col gap-3">
				<BigOptionButton
					:icon="BoxesIcon"
					:title="formatMessage(messages.customTitle)"
					:description="formatMessage(messages.customDesc)"
					@click="setSetupType('custom')"
				/>
				<BigOptionButton
					:icon="PackageIcon"
					:title="formatMessage(messages.modpackTitle)"
					:description="formatMessage(messages.modpackDesc)"
					@click="setSetupType('modpack')"
				/>
				<BigOptionButton
					:icon="BoxImportIcon"
					:title="formatMessage(messages.importTitle)"
					:description="formatMessage(messages.importDesc)"
					@click="ctx.setImportMode()"
				/>
			</div>
			<span class="text-sm text-secondary">
				{{ formatMessage(messages.instanceNote) }}
			</span>
		</template>

		<!-- World / Server onboarding flow options -->
		<template v-else>
			<div class="flex flex-col gap-3">
				<BigOptionButton
					:icon="PackageIcon"
					:title="formatMessage(messages.modpackTitle)"
					:description="formatMessage(messages.modpackDesc)"
					@click="setSetupType('modpack')"
				/>
				<BigOptionButton
					:icon="BoxesIcon"
					:title="formatMessage(messages.customTitle)"
					:description="formatMessage(messages.customDesc)"
					@click="setSetupType('custom')"
				/>
				<BigOptionButton
					:icon="BoxIcon"
					:title="formatMessage(messages.vanillaTitle)"
					:description="formatMessage(messages.vanillaDesc)"
					@click="setSetupType('vanilla')"
				/>
			</div>
			<InlineBackupCreator
				v-if="ctx.flowType === 'reset-server'"
				:backup-name="formatMessage(messages.backupName)"
			/>
		</template>
	</div>
</template>

<script setup lang="ts">
import { BoxesIcon, BoxIcon, BoxImportIcon, PackageIcon } from '@modrinth/assets'
import { defineMessages } from '@vintl/vintl'

import { useDebugLogger } from '#ui/composables/debug-logger'

import InlineBackupCreator from '../../../../layouts/shared/content-tab/components/modals/InlineBackupCreator.vue'
import BigOptionButton from '../../../base/BigOptionButton.vue'
import { injectCreationFlowContext } from '../creation-flow-context'

const debug = useDebugLogger('SetupTypeStage')
const ctx = injectCreationFlowContext()
const { setSetupType: _setSetupType } = ctx
const { formatMessage } = ctx

const messages = defineMessages({
	titleInstance: {
		id: 'creation-flow.setup.title.instance',
		defaultMessage: 'Choose instance type',
	},
	titleServer: {
		id: 'creation-flow.setup.title.server',
		defaultMessage: 'Select installation type',
	},
	titleWorld: {
		id: 'creation-flow.setup.title.world',
		defaultMessage: 'Select world type',
	},
	customTitle: {
		id: 'creation-flow.setup.option.custom.title',
		defaultMessage: 'Custom setup',
	},
	customDesc: {
		id: 'creation-flow.setup.option.custom.desc',
		defaultMessage: 'Start from scratch by picking a loader and game version.',
	},
	modpackTitle: {
		id: 'creation-flow.setup.option.modpack.title',
		defaultMessage: 'Modpack base',
	},
	modpackDesc: {
		id: 'creation-flow.setup.option.modpack.desc',
		defaultMessage: 'Use a popular modpack as your starting point.',
	},
	importTitle: {
		id: 'creation-flow.setup.option.import.title',
		defaultMessage: 'Import instance',
	},
	importDesc: {
		id: 'creation-flow.setup.option.import.desc',
		defaultMessage: 'Import an instance from Prism, CurseForge, or similar.',
	},
	vanillaTitle: {
		id: 'creation-flow.setup.option.vanilla.title',
		defaultMessage: 'Vanilla Minecraft',
	},
	vanillaDesc: {
		id: 'creation-flow.setup.option.vanilla.desc',
		defaultMessage: 'Classic Minecraft with no mods or plugins.',
	},
	instanceNote: {
		id: 'creation-flow.setup.instance.note',
		defaultMessage: 'An instance is a Minecraft setup with a specific loader, version, and mods.',
	},
	backupName: {
		id: 'creation-flow.setup.backup.name',
		defaultMessage: 'Before reinstall',
	},
})

function setSetupType(type: 'modpack' | 'custom' | 'vanilla') {
	debug('selected:', type)
	_setSetupType(type)
}
</script>
