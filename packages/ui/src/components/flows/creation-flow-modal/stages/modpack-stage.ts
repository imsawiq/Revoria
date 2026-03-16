import { LeftArrowIcon } from '@modrinth/assets'
import { defineMessages } from '@vintl/vintl'
import { markRaw } from 'vue'

import type { StageConfigInput } from '../../../base'
import ModpackStage from '../components/ModpackStage.vue'
import type { CreationFlowContextValue } from '../creation-flow-context'

const messages = defineMessages({
	title: {
		id: 'creation-flow.stage.modpack.title',
		defaultMessage: 'Choose modpack',
	},
	back: {
		id: 'creation-flow.action.back',
		defaultMessage: 'Back',
	},
})

export const stageConfig: StageConfigInput<CreationFlowContextValue> = {
	id: 'modpack',
	title: (ctx) => ctx.formatMessage(messages.title),
	stageContent: markRaw(ModpackStage),
	skip: (ctx) => ctx.setupType.value !== 'modpack' || ctx.isImportMode.value,
	leftButtonConfig: (ctx) => ({
		label: ctx.formatMessage(messages.back),
		icon: LeftArrowIcon,
		onClick: () => ctx.modal.value?.setStage('setup-type'),
	}),
	rightButtonConfig: null,
	maxWidth: '520px',
}
