import { LeftArrowIcon, PlusIcon, RightArrowIcon } from '@modrinth/assets'
import { defineMessages } from '@vintl/vintl'
import { markRaw } from 'vue'

import type { StageConfigInput } from '../../../base'
import CustomSetupStage from '../components/CustomSetupStage.vue'
import { type CreationFlowContextValue, getFlowTypeHeading } from '../creation-flow-context'

const messages = defineMessages({
	back: {
		id: 'creation-flow.action.back',
		defaultMessage: 'Back',
	},
	createInstance: {
		id: 'creation-flow.action.create-instance',
		defaultMessage: 'Create instance',
	},
	continue: {
		id: 'creation-flow.action.continue',
		defaultMessage: 'Continue',
	},
	finish: {
		id: 'creation-flow.action.finish',
		defaultMessage: 'Finish',
	},
})

function isForwardBlocked(ctx: CreationFlowContextValue): boolean {
	if (!ctx.selectedGameVersion.value) return true
	if (!ctx.hideLoaderChips.value && !ctx.selectedLoader.value) return true
	if (
		!ctx.hideLoaderVersion.value &&
		ctx.loaderVersionType.value === 'other' &&
		!ctx.selectedLoaderVersion.value
	)
		return true
	return false
}

export const stageConfig: StageConfigInput<CreationFlowContextValue> = {
	id: 'custom-setup',
	title: (ctx) => getFlowTypeHeading(ctx, ctx.formatMessage),
	stageContent: markRaw(CustomSetupStage),
	skip: (ctx) =>
		ctx.setupType.value === 'modpack' ||
		ctx.setupType.value === 'vanilla' ||
		ctx.isImportMode.value,
	cannotNavigateForward: isForwardBlocked,
	leftButtonConfig: (ctx) => ({
		label: ctx.formatMessage(messages.back),
		icon: LeftArrowIcon,
		onClick: () => ctx.modal.value?.setStage('setup-type'),
	}),
	rightButtonConfig: (ctx) => {
		const isInstance = ctx.flowType === 'instance'
		const goesToNextStage =
			ctx.flowType === 'world' ||
			ctx.flowType === 'server-onboarding' ||
			ctx.flowType === 'reset-server'
		const disabled = isForwardBlocked(ctx)

		if (isInstance) {
			return {
				label: ctx.formatMessage(messages.createInstance),
				icon: PlusIcon,
				iconPosition: 'before' as const,
				color: 'brand' as const,
				disabled,
				loading: ctx.loading.value,
				onClick: () => ctx.finish(),
			}
		}

		return {
			label: goesToNextStage
				? ctx.formatMessage(messages.continue)
				: ctx.formatMessage(messages.finish),
			icon: goesToNextStage ? RightArrowIcon : null,
			iconPosition: 'after' as const,
			color: goesToNextStage ? undefined : ('brand' as const),
			disabled,
			onClick: () => {
				if (goesToNextStage) {
					ctx.modal.value?.nextStage()
				} else {
					ctx.finish()
				}
			},
		}
	},
	maxWidth: '520px',
}
