<template>
	<div class="console-action-buttons flex items-center gap-1">
		<ButtonStyled v-if="showClear && hasLogs" type="transparent">
			<button @click="emit('clear')">
				<XIcon />
				{{ formatMessage(messages.clear) }}
			</button>
		</ButtonStyled>
		<ButtonStyled v-if="showDelete" type="transparent" hover-color-fill="background" color="red">
			<button
				v-tooltip="deleteDisabled ? deleteDisabledTooltip : undefined"
				:disabled="deleteDisabled"
				@click="emit('delete')"
			>
				<TrashIcon />
				{{ formatMessage(messages.delete) }}
			</button>
		</ButtonStyled>
		<ButtonStyled v-if="hasLogs" type="transparent">
			<button
				v-tooltip="shareDisabled ? shareDisabledTooltip : undefined"
				:disabled="shareDisabled || sharing"
				@click="emit('share')"
			>
				<SpinnerIcon v-if="sharing" class="animate-spin" />
				<ShareIcon v-else />
				{{ formatMessage(messages.share) }}
			</button>
		</ButtonStyled>
		<ButtonStyled type="transparent">
			<button @click="emit('toggle-fullscreen')">
				<ContractIcon v-if="fullscreen" />
				<ExpandIcon v-else />
				{{ fullscreen ? formatMessage(messages.collapse) : formatMessage(messages.expand) }}
			</button>
		</ButtonStyled>
	</div>
</template>

<script setup lang="ts">
import {
	ContractIcon,
	ExpandIcon,
	ShareIcon,
	SpinnerIcon,
	TrashIcon,
	XIcon,
} from '@modrinth/assets'
import { defineMessages, useVIntl } from '@vintl/vintl'

import ButtonStyled from '#ui/components/base/ButtonStyled.vue'

defineProps<{
	showClear?: boolean
	hasLogs?: boolean
	shareDisabled?: boolean
	shareDisabledTooltip?: string
	sharing?: boolean
	fullscreen?: boolean
	showDelete?: boolean
	deleteDisabled?: boolean
	deleteDisabledTooltip?: string
}>()

const emit = defineEmits<{
	clear: []
	share: []
	'toggle-fullscreen': []
	delete: []
}>()

const { formatMessage } = useVIntl()
const messages = defineMessages({
	clear: { id: 'instance.logs.clear', defaultMessage: 'Clear' },
	delete: { id: 'instance.logs.delete', defaultMessage: 'Delete' },
	share: { id: 'instance.logs.share', defaultMessage: 'Share' },
	expand: { id: 'console.actions.expand', defaultMessage: 'Expand' },
	collapse: { id: 'console.actions.collapse', defaultMessage: 'Collapse' },
})
</script>

<style scoped>
.console-action-buttons :deep(button) {
	white-space: nowrap;
}
</style>
