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
		<ButtonStyled v-if="showOpenFolder" type="transparent">
			<button @click="emit('open-folder')">
				<FolderOpenIcon />
				{{ formatMessage(messages.openFolder) }}
			</button>
		</ButtonStyled>
		<ButtonStyled v-if="hasLogs" type="transparent">
			<button @click="emit('copy')">
				<ClipboardCopyIcon />
				{{ formatMessage(messages.copy) }}
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
	ClipboardCopyIcon,
	ContractIcon,
	ExpandIcon,
	FolderOpenIcon,
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
	showOpenFolder?: boolean
	deleteDisabled?: boolean
	deleteDisabledTooltip?: string
}>()

const emit = defineEmits<{
	clear: []
	copy: []
	share: []
	'toggle-fullscreen': []
	delete: []
	'open-folder': []
}>()

const { formatMessage } = useVIntl()
const messages = defineMessages({
	clear: { id: 'instance.logs.clear', defaultMessage: 'Clear' },
	delete: { id: 'instance.logs.delete', defaultMessage: 'Delete' },
	openFolder: { id: 'instance.logs.open-folder', defaultMessage: 'Open logs folder' },
	copy: { id: 'instance.logs.copy', defaultMessage: 'Copy' },
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
