<script setup lang="ts">
import {
	ChevronRightIcon,
	CodeIcon,
	CoffeeIcon,
	InfoIcon,
	MonitorIcon,
	WrenchIcon,
} from '@modrinth/assets'
import {
	Avatar,
	commonMessages,
	defineMessage,
	NewModal,
	TabbedModal,
	type TabbedModalTab,
	useVIntl,
} from '@modrinth/ui'
import { convertFileSrc } from '@tauri-apps/api/core'
import { nextTick, ref, watch } from 'vue'

import GeneralSettings from '@/components/ui/instance_settings/GeneralSettings.vue'
import HooksSettings from '@/components/ui/instance_settings/HooksSettings.vue'
import InstallationSettings from '@/components/ui/instance_settings/InstallationSettings.vue'
import JavaSettings from '@/components/ui/instance_settings/JavaSettings.vue'
import WindowSettings from '@/components/ui/instance_settings/WindowSettings.vue'
import type { InstanceSettingsTabProps } from '../../../helpers/types'

const { formatMessage } = useVIntl()

const props = defineProps<InstanceSettingsTabProps>()

const tabs: TabbedModalTab<InstanceSettingsTabProps>[] = [
	{
		name: defineMessage({
			id: 'instance.settings.tabs.general',
			defaultMessage: 'General',
		}),
		icon: InfoIcon,
		content: GeneralSettings,
	},
	{
		name: defineMessage({
			id: 'instance.settings.tabs.installation',
			defaultMessage: 'Installation',
		}),
		icon: WrenchIcon,
		content: InstallationSettings,
	},
	{
		name: defineMessage({
			id: 'instance.settings.tabs.window',
			defaultMessage: 'Window',
		}),
		icon: MonitorIcon,
		content: WindowSettings,
	},
	{
		name: defineMessage({
			id: 'instance.settings.tabs.java',
			defaultMessage: 'Java and memory',
		}),
		icon: CoffeeIcon,
		content: JavaSettings,
	},
	{
		name: defineMessage({
			id: 'instance.settings.tabs.hooks',
			defaultMessage: 'Launch hooks',
		}),
		icon: CodeIcon,
		content: HooksSettings,
	},
]

const modal = ref()
const tabbedModal = ref<InstanceType<typeof TabbedModal> | null>(null)
const pendingTab = ref<InstanceSettingsTabId | null>(null)

type InstanceSettingsTabId = 'general' | 'installation' | 'window' | 'java' | 'hooks'
const tabIdToIndex: Record<InstanceSettingsTabId, number> = {
	general: 0,
	installation: 1,
	window: 2,
	java: 3,
	hooks: 4,
}

function show(tab?: InstanceSettingsTabId) {
	if (tab !== undefined) {
		pendingTab.value = tab
	}
	modal.value.show()
	nextTick(() => {
		requestAnimationFrame(() => applyPendingTab())
	})
}

function applyPendingTab() {
	if (pendingTab.value && tabbedModal.value) {
		tabbedModal.value.setTab(tabIdToIndex[pendingTab.value])
		pendingTab.value = null
	}
}

function handleModalShow() {
	// Ensure the tab is applied after the modal is fully shown
	requestAnimationFrame(() => applyPendingTab())
}

watch(
	tabbedModal,
	(value) => {
		if (value) applyPendingTab()
	},
	{ flush: 'post' },
)

defineExpose({ show })

</script>
<template>
	<NewModal
		ref="modal"
		:max-width="'min(928px, calc(95vw - 10rem))'"
		:width="'min(928px, calc(95vw - 10rem))'"
		:on-show="handleModalShow"
	>
		<template #title>
			<span class="flex items-center gap-2 text-lg font-semibold text-primary">
				<Avatar
					:src="instance.icon_path ? convertFileSrc(instance.icon_path) : undefined"
					size="24px"
					:tint-by="props.instance.path"
				/>
				{{ instance.name }} <ChevronRightIcon />
				<span class="font-extrabold text-contrast">
					{{ formatMessage(commonMessages.settingsLabel) }}
				</span>
			</span>
		</template>

		<TabbedModal ref="tabbedModal" :tabs="tabs.map((tab) => ({ ...tab, props }))" />
	</NewModal>
</template>

<style lang="scss" scoped>
</style>
