<script setup>
import {
	DownloadIcon,
	GameIcon,
	PlayIcon,
	SpinnerIcon,
	StopCircleIcon,
	TimerIcon,
} from '@modrinth/assets'
import { Avatar, ButtonStyled, injectNotificationManager, useRelativeTime } from '@modrinth/ui'
import { convertFileSrc } from '@tauri-apps/api/core'
import dayjs from 'dayjs'
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useRouter } from 'vue-router'

import { trackEvent } from '@/helpers/analytics'
import { process_listener } from '@/helpers/events'
import { get_by_profile_path } from '@/helpers/process'
import { finish_install, kill, run } from '@/helpers/profile'
import { showProfileInFolder } from '@/helpers/utils.js'
import { handleSevereError } from '@/store/error.js'

const { handleError } = injectNotificationManager()
const formatRelativeTime = useRelativeTime()

const props = defineProps({
	instance: {
		type: Object,
		default() {
			return {}
		},
	},
	compact: {
		type: Boolean,
		default: false,
	},
	first: {
		type: Boolean,
		default: false,
	},
})

const playing = ref(false)
const loading = ref(false)
const modLoading = computed(
	() =>
		loading.value ||
		currentEvent.value === 'installing' ||
		(currentEvent.value === 'launched' && !playing.value),
)
const installing = computed(() => props.instance.install_stage.includes('installing'))
const installed = computed(() => props.instance.install_stage === 'installed')

const router = useRouter()

const seeInstance = async () => {
	await router.push({
		name: 'Mods',
		params: { id: props.instance.path },
	})
}

const checkProcess = async () => {
	const runningProcesses = await get_by_profile_path(props.instance.path).catch(handleError)

	playing.value = runningProcesses.length > 0
}

const play = async (e, context) => {
	e?.stopPropagation()
	loading.value = true
	await run(props.instance.path)
		.catch((err) => handleSevereError(err, { profilePath: props.instance.path }))
		.finally(() => {
			trackEvent('InstancePlay', {
				loader: props.instance.loader,
				game_version: props.instance.game_version,
				source: context,
			})
		})
	loading.value = false
}

const stop = async (e, context) => {
	e?.stopPropagation()
	playing.value = false

	await kill(props.instance.path).catch(handleError)

	trackEvent('InstanceStop', {
		loader: props.instance.loader,
		game_version: props.instance.game_version,
		source: context,
	})
}

const repair = async (e) => {
	e?.stopPropagation()

	await finish_install(props.instance).catch(handleError)
}

const openFolder = async () => {
	await showProfileInFolder(props.instance.path)
}

const addContent = async () => {
	await router.push({
		path: `/browse/${props.instance.loader === 'vanilla' ? 'datapack' : 'mod'}`,
		query: { i: props.instance.path },
	})
}

defineExpose({
	play,
	stop,
	seeInstance,
	openFolder,
	addContent,
	instance: props.instance,
})

const currentEvent = ref(null)

const unlisten = await process_listener((e) => {
	if (e.profile_path_id === props.instance.path) {
		currentEvent.value = e.event
		if (e.event === 'finished') {
			playing.value = false
		}
	}
})

onMounted(() => checkProcess())
onUnmounted(() => unlisten())
</script>

<template>
	<template v-if="compact">
		<div
			class="instance-card group grid grid-cols-[auto_1fr_auto] bg-[--color-glass-bg-strong] border border-[--glass-border] shadow-[--glass-shadow] rounded-xl p-3 pl-4 gap-2 cursor-pointer transition-colors"
			@click="seeInstance"
		>
			<Avatar
				size="48px"
				:src="instance.icon_path ? convertFileSrc(instance.icon_path) : null"
				:tint-by="instance.path"
				alt="Mod card"
			/>
			<div class="h-full flex items-center font-bold text-contrast leading-normal">
				<span class="line-clamp-2">{{ instance.name }}</span>
			</div>
			<div class="flex items-center">
				<ButtonStyled v-if="playing" color="red" circular @mousehover="checkProcess">
					<button v-tooltip="'Stop'" @click="(e) => stop(e, 'InstanceCard')">
						<StopCircleIcon />
					</button>
				</ButtonStyled>
				<ButtonStyled v-else-if="modLoading" color="standard" circular>
					<button v-tooltip="'Instance is loading...'" disabled>
						<SpinnerIcon class="animate-spin" />
					</button>
				</ButtonStyled>
				<ButtonStyled v-else :color="first ? 'brand' : 'standard'" circular>
					<button
						v-tooltip="'Play'"
						@click="(e) => play(e, 'InstanceCard')"
						@mousehover="checkProcess"
					>
						<!-- Translate for optical centering -->
						<PlayIcon class="translate-x-[1px]" />
					</button>
				</ButtonStyled>
			</div>
			<div class="flex items-center col-span-3 gap-1 text-secondary font-semibold">
				<TimerIcon />
				<span class="text-sm">
					<template v-if="instance.last_played">
						Played {{ formatRelativeTime(dayjs(instance.last_played).toISOString()) }}
					</template>
					<template v-else> Never played </template>
				</span>
			</div>
		</div>
	</template>
	<div v-else>
		<div
			class="instance-card bg-[--color-glass-bg-strong] border border-[--glass-border] shadow-[--glass-shadow] p-4 rounded-xl grid grid-cols-[auto_1fr_auto] gap-x-3 gap-y-1 items-center group cursor-pointer"
			:class="{ 'is-busy': modLoading || installing }"
			@click="seeInstance"
		>
			<div class="row-span-2 flex items-center justify-center">
				<Avatar
					size="48px"
					:src="instance.icon_path ? convertFileSrc(instance.icon_path) : null"
					:tint-by="instance.path"
					alt="Mod card"
					:class="`transition-opacity duration-200 ease-out ${modLoading || installing ? `opacity-55` : `group-hover:opacity-90`}`"
				/>
			</div>
			<div class="min-w-0 flex flex-col gap-1">
				<p class="m-0 text-md font-bold text-contrast leading-tight line-clamp-1">
					{{ instance.name }}
				</p>
				<div class="flex items-center col-span-3 gap-1 text-secondary font-semibold mt-auto">
					<GameIcon class="shrink-0" />
					<span class="text-sm capitalize">
						{{ instance.loader }} {{ instance.game_version }}
					</span>
				</div>
			</div>
			<div class="row-span-2 flex items-center justify-end">
				<ButtonStyled v-if="playing" color="red" circular @mousehover="checkProcess">
					<button v-tooltip="'Stop'" @click="(e) => stop(e, 'InstanceCard')">
						<StopCircleIcon />
					</button>
				</ButtonStyled>
				<ButtonStyled v-else-if="modLoading || installing" color="standard" circular>
					<button v-tooltip="modLoading ? 'Instance is loading...' : 'Installing...'" disabled>
						<SpinnerIcon class="animate-spin" />
					</button>
				</ButtonStyled>
				<ButtonStyled v-else-if="!installed" color="brand" circular>
					<button v-tooltip="'Repair'" @click="(e) => repair(e)">
						<DownloadIcon />
					</button>
				</ButtonStyled>
				<ButtonStyled v-else :color="first ? 'brand' : 'standard'" circular>
					<button
						v-tooltip="'Play'"
						@click="(e) => play(e, 'InstanceCard')"
						@mousehover="checkProcess"
					>
						<PlayIcon class="translate-x-[1px]" />
					</button>
				</ButtonStyled>
			</div>
		</div>
	</div>
</template>

<style scoped>
.instance-card {
	position: relative;
	border-radius: 0.9rem;
	overflow: hidden;
	transform: translateY(0) scale(1) translateZ(0);
	will-change: transform, box-shadow;
	transition:
		transform 520ms cubic-bezier(0.16, 1, 0.3, 1),
		box-shadow 520ms cubic-bezier(0.16, 1, 0.3, 1),
		background-color 360ms cubic-bezier(0.16, 1, 0.3, 1),
		border-color 520ms cubic-bezier(0.16, 1, 0.3, 1);
}

.instance-card::before {
	content: '';
	position: absolute;
	inset: 0;
	background:
		radial-gradient(700px 180px at 15% -10%, rgba(255, 255, 255, 0.08), transparent 60%),
		radial-gradient(520px 200px at 90% -20%, rgba(27, 217, 106, 0.1), transparent 62%);
	opacity: 0;
	transition: opacity 520ms cubic-bezier(0.16, 1, 0.3, 1);
	pointer-events: none;
}

.instance-card::after {
	content: none;
}

.instance-card:hover {
	transform: translateY(-2px) scale(1.008) translateZ(0);
	box-shadow: var(--shadow-floating);
	border-color: rgba(27, 217, 106, 0.22);
}

.instance-card:hover::before {
	opacity: 0.32;
}

.instance-card:hover::after {
}

.instance-card:focus-within {
	border-color: rgba(27, 217, 106, 0.35);
	box-shadow:
		0 0 0 2px rgba(27, 217, 106, 0.25),
		var(--shadow-floating);
}

.instance-card:active {
	transform: translateY(-1px) scale(0.995);
}

.instance-action-layer {
	display: grid;
	place-items: center;
	opacity: 0;
	transform: translateY(6px) scale(0.98);
	pointer-events: none;
	transition:
		opacity 360ms cubic-bezier(0.16, 1, 0.3, 1),
		transform 360ms cubic-bezier(0.16, 1, 0.3, 1);
}

.group:hover .instance-action-layer,
.group:focus-within .instance-action-layer {
	opacity: 1;
	transform: translateY(0px) scale(1);
	pointer-events: auto;
}

.instance-action-wrapper {
	grid-area: 1 / 1;
	opacity: 0;
	transform: translateY(10px) scale(0.92);
	visibility: hidden;
	filter: drop-shadow(0 12px 26px rgba(16, 29, 24, 0.2));
	transition:
		opacity 360ms cubic-bezier(0.16, 1, 0.3, 1),
		transform 360ms cubic-bezier(0.16, 1, 0.3, 1);
	pointer-events: none;
}

.instance-action-wrapper.is-visible {
	opacity: 1;
	transform: translateY(0px) scale(1);
	visibility: visible;
	pointer-events: auto;
}

.instance-spinner {
	grid-area: 1 / 1;
	opacity: 0;
	transform: translateY(10px) scale(0.92);
	visibility: hidden;
	transition:
		opacity 360ms cubic-bezier(0.16, 1, 0.3, 1),
		transform 360ms cubic-bezier(0.16, 1, 0.3, 1);
	pointer-events: none;
}

.instance-spinner.is-visible {
	opacity: 1;
	transform: translateY(0px) scale(1);
	visibility: visible;
}

@media (prefers-reduced-motion: reduce) {
	.instance-card {
		transition:
			transform 180ms ease,
			box-shadow 180ms ease,
			background-color 180ms ease,
			border-color 180ms ease;
	}
	.instance-card::before {
		transition: opacity 180ms ease;
	}
	.instance-card::after {
		transition: none;
	}
	.instance-action-layer,
	.instance-action-wrapper {
		transition:
			opacity 180ms ease,
			transform 180ms ease;
	}
}
</style>
