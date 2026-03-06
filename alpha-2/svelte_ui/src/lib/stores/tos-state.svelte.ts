/**
 * TOS State Store — Reactive state driven by Brain IPC.
 * Uses Svelte 5 runes for fine-grained reactivity.
 */

// --- Type Definitions ---

export interface LogEntry {
    text: string;
    priority: number;
    timestamp: string;
}

export interface TerminalLine {
    text: string;
    priority: number;
}

export interface ProcessInfo {
    pid: number;
    name: string;
    cpu_usage: number;
    mem_usage: number;
    status: string;
    snapshot?: string | null;
    icon?: string | null;
}

export interface Participant {
    id: string;
    alias: string;
    current_level: number;
}

export interface Hub {
    id?: string;
    mode: string;
    prompt: string;
    current_directory: string;
    terminal_output: TerminalLine[];
    staged_command?: string | null;
    ai_explanation?: string | null;
    json_context?: Record<string, any> | null;
    shell_listing?: { path: string; entries: { name: string; is_dir: boolean; size: number }[] } | null;
    activity_listing?: { processes: ProcessInfo[] } | null;
    split_layout?: SplitNode | null;
    focused_pane_id?: string | null;
}

export interface Sector {
    id?: string;
    name: string;
    type?: string;
    status?: string;
    snapshot?: string | null;
    hubs: Hub[];
    active_hub_index: number;
    participants?: Participant[];
    frozen?: boolean;
}

export interface ModuleInfo {
    id: string;
    name: string;
    layout: string;
}

export interface AiModuleInfo {
    id: string;
    name: string;
    provider?: string;
}

export interface AiBehavior {
    id: string;
    name: string;
    enabled: boolean;
    backend_override?: string | null;
    context_fields: string[];
    config: Record<string, string>;
}

export type PaneContent = 'Terminal' | { Application: string };

export interface SplitPane {
    id: string;
    weight: number;
    cwd: string;
    content: PaneContent;
}

export type SplitNode =
    | { Leaf: SplitPane }
    | { Container: { orientation: 'Vertical' | 'Horizontal'; children: SplitNode[] } };

export interface ThemeInfo {
    id: string;
    name: string;
}

export interface PortalInfo {
    active: boolean;
    url?: string;
    token?: string;
}

export interface TosSettings {
    global: Record<string, string>;
    sectors: Record<string, Record<string, string>>;
    applications: Record<string, Record<string, string>>;
}

export interface TosState {
    sys_prefix: string;
    sys_title: string;
    sys_status: string;
    sys_ready: string;
    brain_time: string;
    active_sector_index: number;
    sectors: Sector[];
    system_log: LogEntry[];
    terminal_output: TerminalLine[];
    collab_presence: Participant[];
    settings: TosSettings;
    pending_confirmation?: boolean;
    available_modules: ModuleInfo[];
    active_terminal_module: string;
    available_themes: ThemeInfo[];
    active_theme: string;
    portal?: PortalInfo;
    // AI
    available_ai_modules: AiModuleInfo[];
    active_ai_module: string;
    ai_behaviors: AiBehavior[];
    bezel_expanded: boolean;
    ai_default_backend: string;
}

// --- Default state (used when Brain is not connected) ---

export function getDefaultState(): TosState {
    return {
        sys_prefix: 'ALPHA-2.2 // INTEL-DRIVEN',
        sys_title: 'AWAITING BRAIN LINK...',
        sys_status: 'BRAIN: DISCONNECTED',
        sys_ready: 'LINK FAILURE.',
        brain_time: '--:--:--',
        active_sector_index: 0,
        sectors: [
            {
                name: 'Primary',
                type: 'Standard',
                status: 'Active',
                hubs: [{
                    mode: 'Command',
                    prompt: '',
                    current_directory: '~',
                    terminal_output: [],
                    staged_command: null,
                    ai_explanation: null
                }],
                active_hub_index: 0
            }
        ],
        system_log: [
            {
                text: 'No connection to Brain.',
                priority: 1,
                timestamp: new Date().toISOString()
            }
        ],
        terminal_output: [],
        collab_presence: [],
        settings: { global: {}, sectors: {}, applications: {} },
        available_modules: [],
        active_terminal_module: '',
        available_themes: [],
        active_theme: '',
        available_ai_modules: [],
        active_ai_module: '',
        ai_behaviors: [],
        bezel_expanded: false,
        ai_default_backend: 'tos-ai-standard'
    };
}
