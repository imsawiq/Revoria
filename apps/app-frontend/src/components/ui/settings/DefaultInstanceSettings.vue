<script setup lang="ts">
import { injectNotificationManager, Slider, Toggle } from '@modrinth/ui'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { ref, watch } from 'vue'

import useMemorySlider from '@/composables/useMemorySlider'
import { get, set } from '@/helpers/settings.ts'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	windowSizeTitle: {
		id: 'settings.default-instance.window-size.title',
		defaultMessage: 'Window size',
	},
	fullscreenTitle: {
		id: 'settings.default-instance.fullscreen.title',
		defaultMessage: 'Fullscreen',
	},
	fullscreenDescription: {
		id: 'settings.default-instance.fullscreen.description',
		defaultMessage: 'Overwrites the options.txt file to start in full screen when launched.',
	},
	widthTitle: { id: 'settings.default-instance.width.title', defaultMessage: 'Width' },
	widthDescription: {
		id: 'settings.default-instance.width.description',
		defaultMessage: 'The width of the game window when launched.',
	},
	widthPlaceholder: {
		id: 'settings.default-instance.width.placeholder',
		defaultMessage: 'Enter width...',
	},
	heightTitle: { id: 'settings.default-instance.height.title', defaultMessage: 'Height' },
	heightDescription: {
		id: 'settings.default-instance.height.description',
		defaultMessage: 'The height of the game window when launched.',
	},
	heightPlaceholder: {
		id: 'settings.default-instance.height.placeholder',
		defaultMessage: 'Enter height...',
	},
	memoryAllocatedTitle: {
		id: 'settings.default-instance.memory-allocated.title',
		defaultMessage: 'Memory allocated',
	},
	memoryAllocatedDescription: {
		id: 'settings.default-instance.memory-allocated.description',
		defaultMessage: 'The memory allocated to each instance when it is ran.',
	},
	javaArgumentsTitle: {
		id: 'settings.default-instance.java-arguments.title',
		defaultMessage: 'Java arguments',
	},
	javaArgumentsPlaceholder: {
		id: 'settings.default-instance.java-arguments.placeholder',
		defaultMessage: 'Enter java arguments...',
	},
	environmentalVariablesTitle: {
		id: 'settings.default-instance.environmental-variables.title',
		defaultMessage: 'Environmental variables',
	},
	environmentalVariablesPlaceholder: {
		id: 'settings.default-instance.environmental-variables.placeholder',
		defaultMessage: 'Enter environmental variables...',
	},
	hooksTitle: { id: 'settings.default-instance.hooks.title', defaultMessage: 'Hooks' },
	preLaunchTitle: {
		id: 'settings.default-instance.pre-launch.title',
		defaultMessage: 'Pre launch',
	},
	preLaunchDescription: {
		id: 'settings.default-instance.pre-launch.description',
		defaultMessage: 'Ran before the instance is launched.',
	},
	preLaunchPlaceholder: {
		id: 'settings.default-instance.pre-launch.placeholder',
		defaultMessage: 'Enter pre-launch command...',
	},
	wrapperTitle: { id: 'settings.default-instance.wrapper.title', defaultMessage: 'Wrapper' },
	wrapperDescription: {
		id: 'settings.default-instance.wrapper.description',
		defaultMessage: 'Wrapper command for launching Minecraft.',
	},
	wrapperPlaceholder: {
		id: 'settings.default-instance.wrapper.placeholder',
		defaultMessage: 'Enter wrapper command...',
	},
	postExitTitle: { id: 'settings.default-instance.post-exit.title', defaultMessage: 'Post exit' },
	postExitDescription: {
		id: 'settings.default-instance.post-exit.description',
		defaultMessage: 'Ran after the game closes.',
	},
	postExitPlaceholder: {
		id: 'settings.default-instance.post-exit.placeholder',
		defaultMessage: 'Enter post-exit command...',
	},
})

const fetchSettings = await get()
fetchSettings.launchArgs = fetchSettings.extra_launch_args.join(' ')
fetchSettings.envVars = fetchSettings.custom_env_vars.map((x) => x.join('=')).join(' ')

const settings = ref(fetchSettings)

const { maxMemory, snapPoints } = (await useMemorySlider().catch(handleError)) as unknown as {
	maxMemory: number
	snapPoints: number[]
}

watch(
	settings,
	async () => {
		const setSettings = JSON.parse(JSON.stringify(settings.value))

		setSettings.extra_launch_args = setSettings.launchArgs.trim().split(/\s+/).filter(Boolean)
		setSettings.custom_env_vars = setSettings.envVars
			.trim()
			.split(/\s+/)
			.filter(Boolean)
			.map((x) => x.split('=').filter(Boolean))

		if (!setSettings.hooks.pre_launch) {
			setSettings.hooks.pre_launch = null
		}
		if (!setSettings.hooks.wrapper) {
			setSettings.hooks.wrapper = null
		}
		if (!setSettings.hooks.post_exit) {
			setSettings.hooks.post_exit = null
		}

		if (!setSettings.custom_dir) {
			setSettings.custom_dir = null
		}

		await set(setSettings)
	},
	{ deep: true },
)
</script>

<template>
	<div>
		<h2 class="m-0 text-lg font-extrabold text-contrast">
			{{ formatMessage(messages.windowSizeTitle) }}
		</h2>

		<div class="flex items-center justify-between gap-4">
			<div>
				<h3 class="mt-2 m-0 text-base font-extrabold text-primary">
					{{ formatMessage(messages.fullscreenTitle) }}
				</h3>
				<p class="m-0 mt-1 mb-2 leading-tight text-secondary">
					{{ formatMessage(messages.fullscreenDescription) }}
				</p>
			</div>

			<Toggle id="fullscreen" v-model="settings.force_fullscreen" />
		</div>

		<div class="flex items-center justify-between gap-4">
			<div>
				<h3 class="mt-2 m-0 text-base font-extrabold text-primary">
					{{ formatMessage(messages.widthTitle) }}
				</h3>
				<p class="m-0 mt-1 mb-2 leading-tight text-secondary">
					{{ formatMessage(messages.widthDescription) }}
				</p>
			</div>

			<input
				id="width"
				v-model="settings.game_resolution[0]"
				:disabled="settings.force_fullscreen"
				autocomplete="off"
				type="number"
				:placeholder="formatMessage(messages.widthPlaceholder)"
			/>
		</div>

		<div class="flex items-center justify-between gap-4">
			<div>
				<h3 class="mt-2 m-0 text-base font-extrabold text-primary">
					{{ formatMessage(messages.heightTitle) }}
				</h3>
				<p class="m-0 mt-1 mb-2 leading-tight text-secondary">
					{{ formatMessage(messages.heightDescription) }}
				</p>
			</div>

			<input
				id="height"
				v-model="settings.game_resolution[1]"
				:disabled="settings.force_fullscreen"
				autocomplete="off"
				type="number"
				class="input"
				:placeholder="formatMessage(messages.heightPlaceholder)"
			/>
		</div>

		<hr class="mt-4 bg-button-border border-none h-[1px]" />

		<h2 class="mt-4 m-0 text-lg font-extrabold text-contrast">
			{{ formatMessage(messages.memoryAllocatedTitle) }}
		</h2>
		<p class="m-0 mt-1 leading-tight">{{ formatMessage(messages.memoryAllocatedDescription) }}</p>
		<Slider
			id="max-memory"
			v-model="settings.memory.maximum"
			:min="512"
			:max="maxMemory"
			:step="64"
			:snap-points="snapPoints"
			:snap-range="512"
			unit="MB"
		/>

		<h2 class="mt-4 mb-2 text-lg font-extrabold text-contrast">
			{{ formatMessage(messages.javaArgumentsTitle) }}
		</h2>
		<input
			id="java-args"
			v-model="settings.launchArgs"
			autocomplete="off"
			type="text"
			:placeholder="formatMessage(messages.javaArgumentsPlaceholder)"
			class="w-full"
		/>

		<h2 class="mt-4 mb-2 text-lg font-extrabold text-contrast">
			{{ formatMessage(messages.environmentalVariablesTitle) }}
		</h2>
		<input
			id="env-vars"
			v-model="settings.envVars"
			autocomplete="off"
			type="text"
			:placeholder="formatMessage(messages.environmentalVariablesPlaceholder)"
			class="w-full"
		/>

		<hr class="mt-4 bg-button-border border-none h-[1px]" />

		<h2 class="mt-4 m-0 text-lg font-extrabold text-contrast">
			{{ formatMessage(messages.hooksTitle) }}
		</h2>

		<h3 class="mt-2 m-0 text-base font-extrabold text-primary">
			{{ formatMessage(messages.preLaunchTitle) }}
		</h3>
		<p class="m-0 mt-1 mb-2 leading-tight text-secondary">
			{{ formatMessage(messages.preLaunchDescription) }}
		</p>
		<input
			id="pre-launch"
			v-model="settings.hooks.pre_launch"
			autocomplete="off"
			type="text"
			:placeholder="formatMessage(messages.preLaunchPlaceholder)"
			class="w-full"
		/>

		<h3 class="mt-2 m-0 text-base font-extrabold text-primary">
			{{ formatMessage(messages.wrapperTitle) }}
		</h3>
		<p class="m-0 mt-1 mb-2 leading-tight text-secondary">
			{{ formatMessage(messages.wrapperDescription) }}
		</p>
		<input
			id="wrapper"
			v-model="settings.hooks.wrapper"
			autocomplete="off"
			type="text"
			:placeholder="formatMessage(messages.wrapperPlaceholder)"
			class="w-full"
		/>

		<h3 class="mt-2 m-0 text-base font-extrabold text-primary">
			{{ formatMessage(messages.postExitTitle) }}
		</h3>
		<p class="m-0 mt-1 mb-2 leading-tight text-secondary">
			{{ formatMessage(messages.postExitDescription) }}
		</p>
		<input
			id="post-exit"
			v-model="settings.hooks.post_exit"
			autocomplete="off"
			type="text"
			:placeholder="formatMessage(messages.postExitPlaceholder)"
			class="w-full"
		/>
	</div>
</template>
