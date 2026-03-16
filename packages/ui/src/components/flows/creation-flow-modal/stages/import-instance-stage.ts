import { DownloadIcon, LeftArrowIcon } from '@modrinth/assets'
import { defineMessages } from '@vintl/vintl'
import { markRaw } from 'vue'

import type { StageConfigInput } from '../../../base'
import ImportInstanceStage from '../components/ImportInstanceStage.vue'
import type { CreationFlowContextValue } from '../creation-flow-context'

const messages = defineMessages({
	title: {
		id: 'creation-flow.stage.import.title',
		defaultMessage: 'Import instance',
	},
	back: {
		id: 'creation-flow.action.back',
		defaultMessage: 'Back',
	},
	import: {
		id: 'creation-flow.action.import',
		defaultMessage: 'Import',
	},
	importCount: {
		id: 'creation-flow.action.import-count',
		defaultMessage:
			'Import {count, plural, one {{count} instance} other {{count} instances}}',
	},
})

function getSelectedCount(ctx: CreationFlowContextValue): number {
	let count = 0
	for (const set of Object.values(ctx.importSelectedInstances.value)) {
		count += set.size
	}
	return count
}

export const stageConfig: StageConfigInput<CreationFlowContextValue> = {
	id: 'import-instance',
	title: (ctx) => ctx.formatMessage(messages.title),
	stageContent: markRaw(ImportInstanceStage),
	skip: (ctx) => !ctx.isImportMode.value,
	leftButtonConfig: (ctx) => ({
		label: ctx.formatMessage(messages.back),
		icon: LeftArrowIcon,
		onClick: () => {
			ctx.isImportMode.value = false
			ctx.modal.value?.setStage('setup-type')
		},
	}),
	rightButtonConfig: (ctx) => {
		const count = getSelectedCount(ctx)
		return {
			label:
				count > 0
					? ctx.formatMessage(messages.importCount, { count })
					: ctx.formatMessage(messages.import),
			icon: DownloadIcon,
			iconPosition: 'before' as const,
			color: 'brand' as const,
			disabled: count === 0,
			onClick: () => ctx.finish(),
		}
	},
	maxWidth: '520px',
}
