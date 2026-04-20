<script setup lang="ts">
import { ShareModal } from '@modrinth/ui'
import { ref } from 'vue'

// import { hide_ads_window, show_ads_window } from '@/helpers/ads.js'
import { useTheming } from '@/store/theme.ts'

const themeStore = useTheming()

const props = defineProps({
	header: {
		type: String,
		default: 'Share',
	},
	shareTitle: {
		type: String,
		default: 'Modrinth',
	},
	shareText: {
		type: String,
		default: null,
	},
	link: {
		type: Boolean,
		default: false,
	},
	openInNewTab: {
		type: Boolean,
		default: true,
	},
	onHide: {
		type: Function,
		default: null,
	},
	fullScreen: {
		type: Boolean,
		default: false,
	},
})

const modal = ref<any>(null)

defineExpose({
	show: (passedContent?: unknown): void => {
		// hide_ads_window()
		modal.value?.show?.(passedContent)
	},
	hide: (): void => {
		onModalHide()
		modal.value?.hide?.()
	},
})

const onModalHide = () => {
	// show_ads_window()
	props.onHide?.()
}
</script>

<template>
	<ShareModal
		ref="modal"
		:header="header"
		:share-title="shareTitle"
		:share-text="shareText"
		:link="link"
		:open-in-new-tab="openInNewTab"
		:on-hide="onModalHide"
		:noblur="!themeStore.advancedRendering"
		:full-screen="fullScreen"
	/>
</template>
