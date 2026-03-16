import { LeftArrowIcon, PlusIcon, RightArrowIcon } from '@modrinth/assets'
import { defineMessages } from '@vintl/vintl'
import { markRaw } from 'vue'

import type { StageConfigInput } from '../../../base'
import FinalConfigStage from '../components/FinalConfigStage.vue'
import { type CreationFlowContextValue, getFlowTypeHeading } from '../creation-flow-context'

const messages = defineMessages({
	back: {
		id: 'creation-flow.action.back',
		defaultMessage: 'Back',
	},
	createWorld: {
		id: 'creation-flow.action.create-world',
		defaultMessage: 'Create world',
	},
	resetServer: {
		id: 'creation-flow.action.reset-server',
		defaultMessage: 'Reset server',
	},
	setupServer: {
		id: 'creation-flow.action.setup-server',
		defaultMessage: 'Setup server',
	},
	continue: {
		id: 'creation-flow.action.continue',
		defaultMessage: 'Continue',
	},
})

function isForwardBlocked(ctx: CreationFlowContextValue): boolean {
	if (ctx.flowType === 'world' && !ctx.worldName.value.trim()) return true
	if (ctx.setupType.value === 'vanilla' && !ctx.selectedGameVersion.value) return true
	return false
}

export const stageConfig: StageConfigInput<CreationFlowContextValue> = {
	id: 'final-config',
	title: (ctx) => getFlowTypeHeading(ctx, ctx.formatMessage),
	stageContent: markRaw(FinalConfigStage),
	skip: (ctx) => ctx.flowType === 'instance' || ctx.isImportMode.value,
	cannotNavigateForward: isForwardBlocked,
	leftButtonConfig: (ctx) => ({
		label: ctx.formatMessage(messages.back),
		icon: LeftArrowIcon,
		onClick: () => {
			if (ctx.onBack) {
				ctx.onBack()
			} else {
				ctx.modal.value?.prevStage()
			}
		},
	}),
	rightButtonConfig: (ctx) => {
		const isWorld = ctx.flowType === 'world'
		const isOnboarding = ctx.flowType === 'server-onboarding'
		const isReset = ctx.flowType === 'reset-server'
		const isFinish = isWorld || isOnboarding || isReset
		return {
			label: isWorld
				? ctx.formatMessage(messages.createWorld)
				: isReset
					? ctx.formatMessage(messages.resetServer)
					: isOnboarding
						? ctx.formatMessage(messages.setupServer)
						: ctx.formatMessage(messages.continue),
			icon: isFinish ? PlusIcon : RightArrowIcon,
			iconPosition: isFinish ? ('before' as const) : ('after' as const),
			color: isReset ? ('red' as const) : isFinish ? ('brand' as const) : undefined,
			disabled: isForwardBlocked(ctx),
			loading: isFinish && ctx.loading.value,
			onClick: () => {
				if (isFinish) {
					ctx.finish()
				} else {
					ctx.modal.value?.nextStage()
				}
			},
		}
	},
	maxWidth: '520px',
}
