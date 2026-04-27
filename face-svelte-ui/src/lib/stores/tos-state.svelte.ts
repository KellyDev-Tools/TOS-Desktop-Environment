/**
 * TOS State Store — Reactive state driven by Brain IPC.
 * Uses Svelte 5 runes for fine-grained reactivity.
 */

// --- Type Definitions ---

export type HierarchyLevel = 1 | 2 | 3 | 4 | 5 | 6;

export interface TerminalLine {
    text: string;
    priority: number;
    timestamp: string;
}

export interface ProcessEntry {
    pid: number;
    name: string;
    cpu_usage: number;
    mem_usage: number;
    snapshot?: string | null;
}

export interface Participant {
    id: string;
    alias: string;
    status: 'active' | 'idle' | 'offline';
    role: 'observer' | 'operator' | 'admin';
    current_level: number;
    viewport_title?: string;
    cursor_x?: number;
    cursor_y?: number;
}

export interface Hub {
    id: string;
    mode: 'Command' | 'Directory' | 'Activity' | 'Search' | 'Ai';
    prompt: string;
    current_directory: string;
    terminal_output: TerminalLine[];
    staged_command?: string | null;
    ai_explanation?: string | null;
    json_context?: Record<string, any> | null;
    shell_listing?: { path: string; entries: { name: string; is_dir: boolean; size: number }[] } | null;
    activity_listing?: { processes: ProcessEntry[] } | null;
    split_layout?: SplitNode | null;
    focused_pane_id?: string | null;
    is_running: boolean;
    last_exit_status?: number | null;
}

export interface Sector {
    id: string;
    name: string;
    hubs: Hub[];
    active_hub_index: number;
    frozen: boolean;
    is_remote: boolean;
    disconnected: boolean;
    priority: number;
    active_apps: any[];
    participants: Participant[];
    version: number;
}

export interface TerminalOutputModule {
    id: string;
    name: string;
    version: string;
    layout: 'Rectangular' | 'Cinematic';
    supports_high_contrast: boolean;
    supports_reduced_motion: boolean;
}

export interface ShellModuleMeta {
    id: string;
    name: string;
    version: string;
    author: string;
    executable: string;
    integration: any;
}

export interface ThemeModule {
    id: string;
    name: string;
    version: string;
    author: string;
    description: string;
}

export interface AiModuleMetadata {
    id: string;
    name: string;
    capabilities: string[];
}

export interface AiBehavior {
    id: string;
    name: string;
    enabled: boolean;
    backend_override?: string | null;
    context_fields: string[];
    config: Record<string, string>;
}

export interface ConfirmationRequest {
    id: string;
    original_request: string;
    message: string;
    progress: number;
}

export type SplitOrientation = 'Vertical' | 'Horizontal';

export interface SplitPane {
    id: string;
    weight: number;
    cwd: string;
    content: any;
}

export type SplitNode =
    | { Leaf: SplitPane }
    | { Container: { orientation: SplitOrientation; children: SplitNode[] } };

export interface TosSettings {
    global: Record<string, string>;
    sectors: Record<string, Record<string, string>>;
    applications: Record<string, Record<string, string>>;
    ai_patterns: Record<string, string>;
}

export type FaceProfile = 'desktop' | 'handheld' | 'spatial' | 'headless';

export interface TosState {
    current_level: HierarchyLevel;
    sectors: Sector[];
    active_sector_index: number;
    settings: TosSettings;
    pending_confirmation?: ConfirmationRequest | null;
    system_log: TerminalLine[];
    sys_prefix: string;
    sys_title: string;
    sys_status: string;
    brain_time: string;
    active_terminal_module: string;
    available_modules: TerminalOutputModule[];
    active_shell_module: string;
    available_shell_modules: ShellModuleMeta[];
    active_ai_module: string;
    available_ai_modules: AiModuleMetadata[];
    ai_behaviors: AiBehavior[];
    bezel_expanded: boolean;
    ai_default_backend: string;
    active_theme: string;
    available_themes: ThemeModule[];
    device_profile: FaceProfile;
    ai_offline_queue: any[];
    active_agents: any[];
    active_agent_stack: string[];
    active_curators: string[];
    version: number;
}

// --- Default state (used when Brain is not connected) ---

export function getDefaultState(): TosState {
    return {
        current_level: 1,
        sys_prefix: 'TOS // SYSTEM-BRAIN',
        sys_title: 'BETA-0 // INTEL-DRIVEN',
        sys_status: 'BRAIN: DISCONNECTED',
        brain_time: '--:--:--',
        active_sector_index: 0,
        sectors: [
            {
                id: 'mock-sector-1',
                name: 'TESTING',
                hubs: [{
                    id: 'mock-hub-1',
                    mode: 'Command',
                    prompt: 'tos> ',
                    current_directory: '~',
                    terminal_output: [],
                    is_running: false,
                    activity_listing: {
                        processes: [{ pid: 1, name: 'systemd', cpu_usage: 0.1, mem_usage: 1024 }]
                    }
                }],
                active_hub_index: 0,
                frozen: false,
                is_remote: false,
                disconnected: true,
                priority: 1,
                active_apps: [],
                participants: [],
                version: 0
            }
        ],
        pending_confirmation: null,
        system_log: [
            {
                text: 'No connection to Brain.',
                priority: 1,
                timestamp: new Date().toISOString()
            }
        ],
        settings: {
            global: {
                'tos.onboarding.first_run_complete': 'true',
                'tos.onboarding.wizard_complete': 'true'
            },
            sectors: {},
            applications: {},
            ai_patterns: {}
        },
        available_modules: [],
        active_terminal_module: 'tos-standard-rect',
        available_shell_modules: [],
        active_shell_module: 'tos-shell-fish',
        available_themes: [],
        active_theme: 'tos-classic-lcars',
        available_ai_modules: [],
        active_ai_module: '',
        ai_behaviors: [],
        bezel_expanded: false,
        ai_default_backend: 'tos-ai-standard',
        device_profile: 'desktop',
        ai_offline_queue: [],
        active_agents: [],
        active_agent_stack: [],
        active_curators: [],
        version: 0
    };
}
