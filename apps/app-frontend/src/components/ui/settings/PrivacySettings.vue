<script setup lang="ts">
import { Toggle } from '@modrinth/ui'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { ref, watch } from 'vue'

import { optInAnalytics, optOutAnalytics } from '@/helpers/analytics'
import { get, set } from '@/helpers/settings.ts'

const settings = ref(await get())
const { formatMessage } = useVIntl()

const messages = defineMessages({
	personalizedAdsTitle: {
		id: 'settings.privacy.personalized-ads.title',
		defaultMessage: 'Personalized ads',
	},
	personalizedAdsDescription: {
		id: 'settings.privacy.personalized-ads.description',
		defaultMessage:
			"Modrinth's ad provider, Aditude, shows ads based on your preferences. By disabling this option, you opt out and ads will no longer be shown based on your interests.",
	},
	telemetryTitle: { id: 'settings.privacy.telemetry.title', defaultMessage: 'Telemetry' },
	telemetryDescription: {
		id: 'settings.privacy.telemetry.description',
		defaultMessage:
			'Modrinth collects anonymized analytics and usage data to improve our user experience and customize your experience. By disabling this option, you opt out and your data will no longer be collected.',
	},
	discordRpcTitle: { id: 'settings.privacy.discord-rpc.title', defaultMessage: 'Discord RPC' },
	discordRpcDescription: {
		id: 'settings.privacy.discord-rpc.description',
		defaultMessage:
			"Manages the Discord Rich Presence integration. Disabling this will cause 'Modrinth' to no longer show up as a game or app you are using on your Discord profile.",
	},
	discordRpcNote: {
		id: 'settings.privacy.discord-rpc.note',
		defaultMessage:
			'Note: This will not prevent any instance-specific Discord Rich Presence integrations, such as those added by mods. (app restart required to take effect)',
	},
})

watch(
	settings,
	async () => {
		if (settings.value.telemetry) {
			optInAnalytics()
		} else {
			optOutAnalytics()
		}

		await set(settings.value)
	},
	{ deep: true },
)
</script>

<template>
	<div class="settings-row flex items-center justify-between gap-4">
		<div>
			<h2 class="m-0 text-lg font-extrabold text-contrast">
				{{ formatMessage(messages.personalizedAdsTitle) }}
			</h2>
			<p class="m-0 text-sm">
				{{ formatMessage(messages.personalizedAdsDescription) }}
			</p>
		</div>
		<!-- [AR] Patch. Disabled element by default -->
		<Toggle
			id="personalized-ads"
			v-model="settings.personalized_ads"
			:disabled="!settings.personalized_ads"
		/>
	</div>

	<div class="settings-row mt-4 flex items-center justify-between gap-4">
		<div>
			<h2 class="m-0 text-lg font-extrabold text-contrast">
				{{ formatMessage(messages.telemetryTitle) }}
			</h2>
			<p class="m-0 text-sm">
				{{ formatMessage(messages.telemetryDescription) }}
			</p>
		</div>
		<!-- [AR] Patch. Disabled element by default -->
		<Toggle id="opt-out-analytics" v-model="settings.telemetry" :disabled="!settings.telemetry" />
	</div>

	<div class="settings-row mt-4 flex items-center justify-between gap-4">
		<div>
			<h2 class="m-0 text-lg font-extrabold text-contrast">
				{{ formatMessage(messages.discordRpcTitle) }}
			</h2>
			<p class="m-0 text-sm">
				{{ formatMessage(messages.discordRpcDescription) }}
			</p>
			<p class="m-0 mt-2 text-sm">
				{{ formatMessage(messages.discordRpcNote) }}
			</p>
		</div>
		<Toggle id="disable-discord-rpc" v-model="settings.discord_rpc" />
	</div>
</template>

<style lang="scss" scoped>
.settings-row {
	padding: 1rem 1.125rem;
	border: 1px solid var(--glass-border);
	border-radius: var(--radius-xl);
	background: linear-gradient(
		180deg,
		color-mix(in oklch, var(--color-glass-bg-strong) 88%, transparent),
		color-mix(in oklch, var(--color-glass-bg) 94%, transparent)
	);
	box-shadow: var(--shadow-card);
}
</style>
