<script setup lang="ts">
import { CompassIcon, LibraryIcon, PlusIcon } from '@modrinth/assets'
import { ButtonStyled, injectNotificationManager } from '@modrinth/ui'
import type { SearchResult } from '@modrinth/utils'
import { defineMessages, useVIntl } from '@vintl/vintl'
import dayjs from 'dayjs'
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import RowDisplay from '@/components/RowDisplay.vue'
import RecentWorldsList from '@/components/ui/world/RecentWorldsList.vue'
import { get_search_results } from '@/helpers/cache.js'
import { profile_listener } from '@/helpers/events'
import { list } from '@/helpers/profile.js'
import type { GameInstance } from '@/helpers/types'
import { useBreadcrumbs } from '@/store/breadcrumbs'

const { handleError } = injectNotificationManager()
const route = useRoute()
const router = useRouter()
const breadcrumbs = useBreadcrumbs()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	breadcrumbHome: {
		id: 'app.home.breadcrumb',
		defaultMessage: 'Home',
	},
	greetingNight: {
		id: 'app.home.greeting.night',
		defaultMessage: 'Good night',
	},
	greetingMorning: {
		id: 'app.home.greeting.morning',
		defaultMessage: 'Good morning',
	},
	greetingAfternoon: {
		id: 'app.home.greeting.afternoon',
		defaultMessage: 'Good afternoon',
	},
	greetingEvening: {
		id: 'app.home.greeting.evening',
		defaultMessage: 'Good evening',
	},
	welcomeBack: {
		id: 'app.home.welcome.back',
		defaultMessage: 'Welcome back!',
	},
	welcomeNew: {
		id: 'app.home.welcome.new',
		defaultMessage: 'Welcome to Revoria',
	},
	instancesCount: {
		id: 'app.home.instances.count',
		defaultMessage: '{count} instances',
	},
	browseContent: {
		id: 'app.home.action.browse-content',
		defaultMessage: 'Browse content',
	},
	myLibrary: {
		id: 'app.home.action.my-library',
		defaultMessage: 'My library',
	},
	discoverModpack: {
		id: 'app.home.row.discover-modpack',
		defaultMessage: 'Discover a modpack',
	},
	discoverMods: {
		id: 'app.home.row.discover-mods',
		defaultMessage: 'Discover mods',
	},
})

breadcrumbs.setRootContext({ name: formatMessage(messages.breadcrumbHome), link: route.path })

const instances = ref<GameInstance[]>([])

const featuredModpacks = ref<SearchResult[]>([])
const featuredMods = ref<SearchResult[]>([])
const installedModpacksFilter = ref('')

const recentInstances = computed(() =>
	instances.value
		.filter((x) => x.last_played)
		.slice()
		.sort((a, b) => dayjs(b.last_played).diff(dayjs(a.last_played))),
)

const hasFeaturedProjects = computed(
	() => (featuredModpacks.value?.length ?? 0) + (featuredMods.value?.length ?? 0) > 0,
)

const greeting = computed(() => {
	const hour = new Date().getHours()
	if (hour < 6) return formatMessage(messages.greetingNight)
	if (hour < 12) return formatMessage(messages.greetingMorning)
	if (hour < 18) return formatMessage(messages.greetingAfternoon)
	return formatMessage(messages.greetingEvening)
})

const instanceCount = computed(() => instances.value?.length ?? 0)

async function fetchInstances() {
	instances.value = (await list().catch(handleError)) ?? []

	const filters = []
	for (const instance of instances.value) {
		if (instance.linked_data && instance.linked_data.project_id) {
			filters.push(`NOT"project_id"="${instance.linked_data.project_id}"`)
		}
	}
	installedModpacksFilter.value = filters.join(' AND ')
}

async function fetchFeaturedModpacks() {
	const response = await get_search_results(
		`?facets=[["project_type:modpack"]]&limit=10&index=follows&filters=${installedModpacksFilter.value}`,
	)

	if (response) {
		featuredModpacks.value = response.result.hits
	} else {
		featuredModpacks.value = []
	}
}

async function fetchFeaturedMods() {
	const response = await get_search_results('?facets=[["project_type:mod"]]&limit=10&index=follows')

	if (response) {
		featuredMods.value = response.result.hits
	} else {
		featuredModpacks.value = []
	}
}

async function refreshFeaturedProjects() {
	await Promise.all([fetchFeaturedModpacks(), fetchFeaturedMods()])
}

let unlistenProfile: (() => void) | null = null

onMounted(async () => {
	await fetchInstances()
	await refreshFeaturedProjects()

	unlistenProfile = await profile_listener(
		async (e: { event: string; profile_path_id: string }) => {
			await fetchInstances()

			if (e.event === 'added' || e.event === 'created' || e.event === 'removed') {
				await refreshFeaturedProjects()
			}
		},
	).catch(() => null)
})

onBeforeUnmount(() => {
	unlistenProfile?.()
})
</script>

<template>
	<div class="home-page p-6 flex flex-col gap-5">
		<div class="hero-section relative overflow-hidden rounded-2xl bg-gradient-to-br from-[rgba(27,217,106,0.12)] via-[rgba(27,217,106,0.04)] to-transparent border border-[--glass-border] p-6">
			<div class="relative z-10 flex flex-col gap-3">
				<div class="flex items-end justify-between gap-4">
					<div>
						<p class="m-0 text-sm font-medium text-secondary uppercase tracking-wider">{{ greeting }}</p>
						<h1 class="m-0 mt-1 text-3xl font-extrabold text-contrast leading-tight">
							<template v-if="recentInstances?.length > 0">{{ formatMessage(messages.welcomeBack) }}</template>
							<template v-else>{{ formatMessage(messages.welcomeNew) }}</template>
						</h1>
					</div>
					<div v-if="instanceCount > 0" class="flex items-center gap-4 text-sm text-secondary shrink-0 pb-1">
						<div class="flex items-center gap-1.5">
							<LibraryIcon class="w-4 h-4" />
							<span>{{ formatMessage(messages.instancesCount, { count: instanceCount }) }}</span>
						</div>
					</div>
				</div>
				<div class="flex gap-2 mt-1">
					<ButtonStyled color="brand">
						<button @click="router.push('/browse/modpack')">
							<CompassIcon class="w-4 h-4" />
							{{ formatMessage(messages.browseContent) }}
						</button>
					</ButtonStyled>
					<ButtonStyled type="outlined">
						<button @click="router.push('/library')">
							<LibraryIcon class="w-4 h-4" />
							{{ formatMessage(messages.myLibrary) }}
						</button>
					</ButtonStyled>
				</div>
			</div>
			<div class="hero-glow"></div>
		</div>

		<RecentWorldsList :recent-instances="recentInstances" />

		<RowDisplay
			v-if="hasFeaturedProjects"
			:instances="[
				{
					label: formatMessage(messages.discoverModpack),
					route: '/browse/modpack',
					instances: featuredModpacks,
					downloaded: false,
				},
				{
					label: formatMessage(messages.discoverMods),
					route: '/browse/mod',
					instances: featuredMods,
					downloaded: false,
				},
			]"
			:can-paginate="true"
		/>
	</div>
</template>

<style scoped>
.hero-glow {
	position: absolute;
	top: -40%;
	right: -10%;
	width: 300px;
	height: 300px;
	background: radial-gradient(circle, rgba(27, 217, 106, 0.15) 0%, transparent 70%);
	pointer-events: none;
	z-index: 0;
}
</style>
