import {
	BadgeCheckIcon,
	BlocksIcon,
	CloudIcon,
	CompassIcon,
	CrownIcon,
	GlobeIcon,
	GridIcon,
	HeartIcon,
	LockIcon,
	PickaxeIcon,
	RefreshCwIcon,
	ShieldIcon,
	SkullIcon,
	StarIcon,
	TerminalSquareIcon,
	UsersIcon,
} from '@modrinth/assets'
import { formatCategory } from '@modrinth/utils'
import { computed, type Ref, ref } from 'vue'
import { useRoute } from 'vue-router'

import { defineMessages, useVIntl } from '#ui/composables/i18n'

import type { FilterType, FilterValue, SortType, Tags } from './search'

const SERVER_REGIONS = [
	{ code: 'us_east', name: 'US East' },
	{ code: 'us_west', name: 'US West' },
	{ code: 'europe', name: 'Europe' },
	{ code: 'asia', name: 'Asia' },
	{ code: 'australia', name: 'Australia' },
	{ code: 'south_america', name: 'South America' },
	{ code: 'middle_east', name: 'Middle East' },
	{ code: 'russia', name: 'Russia' },
]

const SERVER_LANGUAGES = [
	{ code: 'en', name: 'English' },
	{ code: 'de', name: 'German' },
	{ code: 'fr', name: 'French' },
	{ code: 'es', name: 'Spanish' },
	{ code: 'pt', name: 'Portuguese' },
	{ code: 'ru', name: 'Russian' },
	{ code: 'zh', name: 'Chinese' },
	{ code: 'ja', name: 'Japanese' },
	{ code: 'ko', name: 'Korean' },
	{ code: 'nl', name: 'Dutch' },
	{ code: 'pl', name: 'Polish' },
	{ code: 'it', name: 'Italian' },
	{ code: 'tr', name: 'Turkish' },
	{ code: 'sv', name: 'Swedish' },
	{ code: 'fi', name: 'Finnish' },
]

const FILTER_FIELD_MAP: Record<string, string> = {
	server_content_type: 'minecraft_java_server.content.kind',
	server_game_version: 'minecraft_java_server.content.supported_game_versions',
	server_status: 'minecraft_java_server.ping.data',
	server_region: 'minecraft_server.region',
	server_language: 'minecraft_server.languages',
}

const messages = defineMessages({
	sortRelevance: { id: 'server.search.sort.relevance', defaultMessage: 'Relevance' },
	sortVerifiedPlays: {
		id: 'server.search.sort.verified-plays',
		defaultMessage: 'Verified Plays',
	},
	sortPlayers: { id: 'server.search.sort.players', defaultMessage: 'Players' },
	sortFollowers: { id: 'server.search.sort.followers', defaultMessage: 'Followers' },
	sortDatePublished: {
		id: 'server.search.sort.date-published',
		defaultMessage: 'Date Published',
	},
	sortDateUpdated: {
		id: 'server.search.sort.date-updated',
		defaultMessage: 'Date Updated',
	},
	headerCommunity: {
		id: 'server.search.header.community',
		defaultMessage: 'Community',
	},
	headerFeatures: { id: 'server.search.header.features', defaultMessage: 'Features' },
	headerGameplay: { id: 'server.search.header.gameplay', defaultMessage: 'Gameplay' },
	headerMeta: { id: 'server.search.header.meta', defaultMessage: 'Meta' },
	status: { id: 'server.search.filter.status', defaultMessage: 'Status' },
	statusOnline: { id: 'server.search.filter.status-online', defaultMessage: 'Online' },
	statusOffline: { id: 'server.search.filter.status-offline', defaultMessage: 'Offline' },
	contentType: { id: 'server.search.filter.content-type', defaultMessage: 'Type' },
	contentVanilla: { id: 'server.search.filter.content-vanilla', defaultMessage: 'Vanilla' },
	contentModded: { id: 'server.search.filter.content-modded', defaultMessage: 'Modded' },
	gameVersion: { id: 'server.search.filter.game-version', defaultMessage: 'Game Version' },
	region: { id: 'server.search.filter.region', defaultMessage: 'Region' },
	language: { id: 'server.search.filter.language', defaultMessage: 'Language' },
	categorySkyblock: { id: 'server.search.category.skyblock', defaultMessage: 'Skyblock' },
	categoryPrison: { id: 'server.search.category.prison', defaultMessage: 'Prison' },
	categorySmp: { id: 'server.search.category.smp', defaultMessage: 'SMP' },
	categoryPokemon: { id: 'server.search.category.pokemon', defaultMessage: 'Pokemon' },
	categoryAdventure: { id: 'server.search.category.adventure', defaultMessage: 'Adventure' },
	categoryAnarchy: { id: 'server.search.category.anarchy', defaultMessage: 'Anarchy' },
	categoryPvp: { id: 'server.search.category.pvp', defaultMessage: 'PvP' },
	categoryPve: { id: 'server.search.category.pve', defaultMessage: 'PvE' },
	categoryBosses: { id: 'server.search.category.bosses', defaultMessage: 'Bosses' },
	categoryNetwork: { id: 'server.search.category.network', defaultMessage: 'Network' },
	categoryCustomContent: {
		id: 'server.search.category.custom-content',
		defaultMessage: 'Custom content',
	},
	categoryMinigames: { id: 'server.search.category.minigames', defaultMessage: 'Minigames' },
	categoryParkour: { id: 'server.search.category.parkour', defaultMessage: 'Parkour' },
	categoryQuesting: { id: 'server.search.category.questing', defaultMessage: 'Questing' },
	categoryLifesteal: { id: 'server.search.category.lifesteal', defaultMessage: 'Lifesteal' },
	categoryWhitelisted: {
		id: 'server.search.category.whitelisted',
		defaultMessage: 'Whitelisted',
	},
	categoryGens: { id: 'server.search.category.gens', defaultMessage: 'Gens' },
	categoryTechnical: { id: 'server.search.category.technical', defaultMessage: 'Technical' },
	categoryWorldResets: {
		id: 'server.search.category.world-resets',
		defaultMessage: 'World resets',
	},
})

const SERVER_CATEGORY_ICON_COMPONENTS: Record<string, unknown> = {
	skyblock: CloudIcon,
	prison: LockIcon,
	smp: UsersIcon,
	pokemon: GlobeIcon,
	adventure: CompassIcon,
	anarchy: SkullIcon,
	pvp: ShieldIcon,
	pve: ShieldIcon,
	bosses: CrownIcon,
	network: GlobeIcon,
	custom_content: BlocksIcon,
	custom_content_: BlocksIcon,
	minigames: GridIcon,
	parkour: StarIcon,
	questing: StarIcon,
	lifesteal: HeartIcon,
	whitelisted: BadgeCheckIcon,
	gens: PickaxeIcon,
	technical: TerminalSquareIcon,
	world_resets: RefreshCwIcon,
}

function formatServerHeader(formatMessage: ReturnType<typeof useVIntl>['formatMessage'], header: string): string {
	switch (header) {
		case 'minecraft_server_community':
			return formatMessage(messages.headerCommunity)
		case 'minecraft_server_features':
			return formatMessage(messages.headerFeatures)
		case 'minecraft_server_gameplay':
			return formatMessage(messages.headerGameplay)
		case 'minecraft_server_meta':
			return formatMessage(messages.headerMeta)
	}
	return header
		.replaceAll('_', ' ')
		.replace(/\b\w/g, (char) => char.toUpperCase())
}

function resolveFilterIcon(icon: unknown) {
	if (typeof icon === 'string') {
		const trimmed = icon.trim()
		if (
			trimmed.startsWith('<') ||
			trimmed.startsWith('http') ||
			trimmed.startsWith('data:image') ||
			trimmed.startsWith('/')
		) {
			return icon
		}
	}
	return ''
}

function resolveServerCategoryIconByName(name: string, apiIcon: unknown) {
	const normalized = name.toLowerCase().replace(/[-\s]/g, '_')
	return (
		SERVER_CATEGORY_ICON_COMPONENTS[normalized] ??
		SERVER_CATEGORY_ICON_COMPONENTS[name.toLowerCase()] ??
		resolveFilterIcon(apiIcon)
	)
}

function formatServerCategoryName(formatMessage: ReturnType<typeof useVIntl>['formatMessage'], name: string) {
	const normalized = name.toLowerCase().replace(/[-\s]/g, '_')
	switch (normalized) {
		case 'skyblock':
			return formatMessage(messages.categorySkyblock)
		case 'prison':
			return formatMessage(messages.categoryPrison)
		case 'smp':
			return formatMessage(messages.categorySmp)
		case 'pokemon':
			return formatMessage(messages.categoryPokemon)
		case 'adventure':
			return formatMessage(messages.categoryAdventure)
		case 'anarchy':
			return formatMessage(messages.categoryAnarchy)
		case 'pvp':
			return formatMessage(messages.categoryPvp)
		case 'pve':
			return formatMessage(messages.categoryPve)
		case 'bosses':
			return formatMessage(messages.categoryBosses)
		case 'network':
			return formatMessage(messages.categoryNetwork)
		case 'custom_content':
		case 'custom_content_':
			return formatMessage(messages.categoryCustomContent)
		case 'minigames':
			return formatMessage(messages.categoryMinigames)
		case 'parkour':
			return formatMessage(messages.categoryParkour)
		case 'questing':
			return formatMessage(messages.categoryQuesting)
		case 'lifesteal':
			return formatMessage(messages.categoryLifesteal)
		case 'whitelisted':
			return formatMessage(messages.categoryWhitelisted)
		case 'gens':
			return formatMessage(messages.categoryGens)
		case 'technical':
			return formatMessage(messages.categoryTechnical)
		case 'world_resets':
			return formatMessage(messages.categoryWorldResets)
		default:
			return formatCategory(name)
	}
}

function getFilterField(filterId: string): string | undefined {
	if (filterId.startsWith('server_category_')) return 'categories'
	return FILTER_FIELD_MAP[filterId]
}

export function useServerSearch(opts: {
	tags: Ref<Tags>
	query: Ref<string>
	maxResults: Ref<number>
	currentPage: Ref<number>
}) {
	const { formatMessage } = useVIntl()
	const toPositiveInt = (value: unknown, fallback: number) => {
		const parsed = Number(value)
		if (!Number.isFinite(parsed)) return fallback
		const rounded = Math.floor(parsed)
		return rounded > 0 ? rounded : fallback
	}

	const { tags, query, maxResults, currentPage } = opts
	const route = useRoute()

	const serverSortTypes = computed<SortType[]>(() => [
		{ display: formatMessage(messages.sortRelevance), name: 'relevance' },
		{
			display: formatMessage(messages.sortVerifiedPlays),
			name: 'minecraft_java_server.verified_plays_2w',
		},
		{
			display: formatMessage(messages.sortPlayers),
			name: 'minecraft_java_server.ping.data.players_online',
		},
		{ display: formatMessage(messages.sortFollowers), name: 'follows' },
		{ display: formatMessage(messages.sortDatePublished), name: 'date_created' },
		{ display: formatMessage(messages.sortDateUpdated), name: 'date_modified' },
	])
	const serverCurrentSortType = ref<SortType>(serverSortTypes.value[0])
	const serverCurrentFilters = ref<FilterValue[]>([{ type: 'server_status', option: 'online' }])
	const serverToggledGroups = ref<string[]>([])

	const serverFilterTypes = computed<FilterType[]>(() => {
		const categoryFilters: Record<string, FilterType> = {}
		for (const c of (tags.value?.categories ?? []).filter(
			(c) => c.project_type === 'minecraft_java_server',
		)) {
			const filterTypeId = `server_category_${c.header}`
			if (!categoryFilters[filterTypeId]) {
				categoryFilters[filterTypeId] = {
					id: filterTypeId,
					formatted_name: formatServerHeader(formatMessage, c.header),
					supported_project_types: ['server'],
					display: 'all',
					query_param: 'sc',
					supports_negative_filter: true,
					searchable: false,
					options: [],
				}
			}
				categoryFilters[filterTypeId].options.push({
				id: c.name,
				formatted_name: formatServerCategoryName(formatMessage, c.name),
				icon: resolveServerCategoryIconByName(c.name, c.icon),
				method: 'or',
				value: c.name,
			})
		}

		const preferredSections = [
			'minecraft_server_features',
			'minecraft_server_gameplay',
			'minecraft_server_meta',
			'minecraft_server_community',
		]
		const sectionFilters = preferredSections
			.map((h) => `server_category_${h}`)
			.map((id) => categoryFilters[id])
			.filter(Boolean)

		return [
			{
				id: 'server_content_type',
				formatted_name: formatMessage(messages.contentType),
				supported_project_types: ['server'],
				display: 'all',
				query_param: 'sct',
				supports_negative_filter: false,
				searchable: false,
				options: [
					{
						id: 'vanilla',
						formatted_name: formatMessage(messages.contentVanilla),
						method: 'or',
						value: 'vanilla',
					},
					{
						id: 'modpack',
						formatted_name: formatMessage(messages.contentModded),
						method: 'or',
						value: 'modpack',
					},
				],
			},
			...sectionFilters,
			{
				id: 'server_game_version',
				formatted_name: formatMessage(messages.gameVersion),
				supported_project_types: ['server'],
				display: 'scrollable',
				query_param: 'sgv',
				supports_negative_filter: false,
				searchable: true,
				options: (tags.value?.gameVersions ?? []).map((gv) => ({
					id: gv.version,
					toggle_group: gv.version_type !== 'release' ? 'all_versions' : undefined,
					method: 'or',
					value: gv.version,
					query_value: gv.version,
				})),
			},
			{
				id: 'server_region',
				formatted_name: formatMessage(messages.region),
				supported_project_types: ['server'],
				display: 'all',
				query_param: 'sr',
				supports_negative_filter: true,
				searchable: false,
				options: SERVER_REGIONS.map((r) => ({
					id: r.code,
					formatted_name: r.name,
					method: 'or',
					value: r.code,
				})),
			},
			{
				id: 'server_language',
				formatted_name: formatMessage(messages.language),
				supported_project_types: ['server'],
				display: 'scrollable',
				query_param: 'sl',
				supports_negative_filter: false,
				searchable: true,
				options: SERVER_LANGUAGES.map((l) => ({
					id: l.code,
					formatted_name: l.name,
					method: 'or',
					value: l.code,
				})),
			},
			{
				id: 'server_status',
				formatted_name: formatMessage(messages.status),
				supported_project_types: ['server'],
				display: 'all',
				query_param: 'sst',
				supports_negative_filter: false,
				searchable: false,
				options: [
					{
						id: 'online',
						formatted_name: formatMessage(messages.statusOnline),
						method: 'or',
						value: 'online',
					},
					{
						id: 'offline',
						formatted_name: formatMessage(messages.statusOffline),
						method: 'or',
						value: 'offline',
					},
				],
			},
		]
	})

	const newFilters = computed(() => {
		const parts = ['project_types = minecraft_java_server']

		for (const filterType of serverFilterTypes.value) {
			const field = getFilterField(filterType.id)
			if (!field) continue
			const matched = serverCurrentFilters.value.filter((f) => f.type === filterType.id)
			if (matched.length === 0) continue

			if (filterType.id === 'server_status') {
				const selected = matched[0]?.option
				if (selected === 'online') {
					parts.push(`${field} EXISTS`)
				} else if (selected === 'offline') {
					parts.push(`${field} NOT EXISTS`)
				}
				continue
			}

			const included = matched.filter((f) => !f.negative)
			const excluded = matched.filter((f) => f.negative)
			if (included.length > 0) {
				const values = included.map((f) => `"${f.option}"`).join(', ')
				parts.push(`${field} IN [${values}]`)
			}
			if (excluded.length > 0) {
				const values = excluded.map((f) => `"${f.option}"`).join(', ')
				parts.push(`${field} NOT IN [${values}]`)
			}
		}

		return parts.join(' AND ')
	})

	const serverRequestParams = computed(() => {
		const params = [`limit=${maxResults.value}`, `index=${serverCurrentSortType.value.name}`]
		if (query.value) params.push(`query=${encodeURIComponent(query.value)}`)
		const offset = (currentPage.value - 1) * maxResults.value
		if (offset > 0) params.push(`offset=${offset}`)
		params.push(`new_filters=${encodeURIComponent(newFilters.value)}`)
		return `?${params.join('&')}`
	})

	function readServerQueryParams() {
		const q = route.query
		if (q.q) {
			query.value = String(q.q)
		}
		if (q.m) {
			maxResults.value = toPositiveInt(q.m, 20)
		}
		if (q.page) {
			currentPage.value = toPositiveInt(q.page, 1)
		}
		if (q.ss) {
			serverCurrentSortType.value =
				serverSortTypes.value.find((s) => s.name === String(q.ss)) ?? serverSortTypes.value[0]
		}
		for (const filterType of serverFilterTypes.value) {
			const paramValue = q[filterType.query_param]
			if (!paramValue) continue
			const values =
				typeof paramValue === 'string'
					? [paramValue]
					: paramValue.filter((v): v is string => v !== null)
			for (const value of values) {
				const isNegative = value.startsWith('!')
				const cleanValue = isNegative ? value.slice(1) : value
				const option = filterType.options.find((o) => o.id === cleanValue)
				if (option) {
					serverCurrentFilters.value.push({
						type: filterType.id,
						option: option.id,
						negative: isNegative,
					})
				}
			}
		}
	}

	function createServerPageParams(): Record<string, string | string[]> {
		const items: Record<string, string[]> = {}
		if (query.value) items.q = [query.value]
		for (const filterValue of serverCurrentFilters.value) {
			const type = serverFilterTypes.value.find((t) => t.id === filterValue.type)
			if (type) {
				const value = filterValue.negative ? `!${filterValue.option}` : filterValue.option
				if (items[type.query_param]) items[type.query_param].push(value)
				else items[type.query_param] = [value]
			}
		}
		if (serverCurrentSortType.value.name !== 'relevance') {
			items.ss = [serverCurrentSortType.value.name]
		}
		if (maxResults.value !== 20) {
			items.m = [String(maxResults.value)]
		}
		if (currentPage.value > 1) {
			items.page = [String(currentPage.value)]
		}
		return items
	}

	readServerQueryParams()

	return {
		serverCurrentSortType,
		serverCurrentFilters,
		serverToggledGroups,
		serverSortTypes,
		serverFilterTypes,
		serverRequestParams,
		createServerPageParams,
	}
}
