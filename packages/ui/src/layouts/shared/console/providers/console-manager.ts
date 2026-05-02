import type { ComputedRef, Ref } from 'vue'

import { createContext } from '#ui/providers/create-context'

import type { CrashAnalysisResult, LogLine, LogSource } from '../types'

export interface ConsoleManagerContext {
	logLines: Ref<LogLine[]>

	logSources?: ComputedRef<LogSource[]>
	activeLogSourceIndex?: Ref<number>

	sendCommand?: (cmd: string) => void
	showCommandInput?: boolean | Ref<boolean> | ComputedRef<boolean>
	disableCommandInput?: boolean | Ref<boolean> | ComputedRef<boolean>

	loading?: Ref<boolean> | ComputedRef<boolean>

	onClear?: () => void
	onDelete?: () => Promise<void>
	onOpenFolder?: () => Promise<void>
	deleteDisabled?: Ref<boolean> | ComputedRef<boolean>
	deleteDisabledTooltip?: string

	shareDisabled?: Ref<boolean> | ComputedRef<boolean>

	emptyStateType?: 'server' | 'instance'

	crashAnalysis?: Ref<CrashAnalysisResult | null>
	onDismissCrash?: () => void
}

export const [injectConsoleManager, provideConsoleManager] = createContext<ConsoleManagerContext>(
	'ConsolePageLayout',
	'consoleManagerContext',
)
