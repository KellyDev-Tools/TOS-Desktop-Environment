<script lang="ts">
    import { getTosState, sendIpc } from "$lib/stores/ipc.svelte";
    import { onMount } from "svelte";

    const tosState = $derived(getTosState());
    let activeSector = $derived(tosState.sectors[tosState.active_sector_index]);
    let board = $derived(activeSector?.kanban_board);

    onMount(() => {
        if (!board) {
            sendIpc("kanban_get", "");
        }
    });

    async function initBoard() {
        await sendIpc("kanban_init", "");
    }

    async function addTask(laneId: string) {
        const title = prompt("Task Title:");
        if (!title) return;
        const description = prompt("Description:");
        await sendIpc("kanban_task_add", JSON.stringify({
            lane_id: laneId,
            title,
            description: description || ""
        }));
    }

    async function moveTask(taskId: string, fromLaneId: string, toLaneId: string) {
        await sendIpc("kanban_task_move", JSON.stringify({
            task_id: taskId,
            from_lane: fromLaneId,
            to_lane: toLaneId
        }));
    }

    async function deleteTask(taskId: string, laneId: string) {
        if (confirm("Delete this task?")) {
            await sendIpc("kanban_task_delete", JSON.stringify({
                task_id: taskId,
                lane_id: laneId
            }));
        }
    }
</script>

<div class="workflow-container">
    {#if !board}
        <div class="empty-state">
            <h2 class="lcars-text">NO PROJECT WORKFLOW DETECTED</h2>
            <p class="lcars-text gray small">Sectors require initialization to support Kanban features (§7.1)</p>
            <button class="lcars-pill-button amber" onclick={initBoard}>
                INITIALIZE WORKFLOW SERVICE
            </button>
        </div>
    {:else}
        <div class="board-header">
            <div class="path-segment amber">{board.title}</div>
            <div class="path-segment gray">// workflow // active</div>
            <div class="header-actions">
                <button class="lcars-pill-button indigo" onclick={() => sendIpc("ai_roadmap_plan", "")}>
                    SYNC ROADMAP
                </button>
                <button class="lcars-pill-button rose" onclick={() => sendIpc("ai_dream_consolidate", "")}>
                    CONSOLIDATE MEMORY
                </button>
            </div>
        </div>
        <div class="lanes-container">
            {#each board.lanes as lane}
                <div class="lane">
                    <div class="lane-header">
                        <span class="lane-title">{lane.title}</span>
                        <div class="lane-stats">{lane.tasks.length}</div>
                        <button class="icon-btn amber" title="Add Task" aria-label="Add Task" onclick={() => addTask(lane.id)}>+</button>
                    </div>
                    <div class="tasks-list">
                        {#each lane.tasks as task}
                            <div class="task-card" draggable="true">
                                <div class="task-title">{task.title}</div>
                                <div class="task-desc">{task.description}</div>
                                <div class="task-footer">
                                    <div class="status-indicator {task.status.toLowerCase()}"></div>
                                    <div class="task-id">#{task.id.slice(0, 4)}</div>
                                    <div class="actions">
                                        <button class="small-btn red" onclick={() => deleteTask(task.id, lane.id)}>DEL</button>
                                    </div>
                                </div>
                            </div>
                        {/each}
                    </div>
                </div>
            {/each}
        </div>
    {/if}
</div>

<style>
    .workflow-container {
        display: flex;
        flex-direction: column;
        height: 100%;
        padding: 1rem;
        background: rgba(0, 0, 0, 0.4);
        font-family: 'Outfit', sans-serif;
        color: #ddd;
        overflow: hidden;
    }

    .empty-state {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        height: 100%;
        text-align: center;
        gap: 1.5rem;
    }

    .lcars-text {
        letter-spacing: 0.1em;
        text-transform: uppercase;
    }

    .lcars-text.gray { color: #888; }
    .lcars-text.small { font-size: 0.8rem; }

    .lcars-pill-button {
        padding: 0.6rem 2rem;
        border-radius: 20px;
        border: none;
        font-weight: 800;
        cursor: pointer;
        text-transform: uppercase;
        font-family: inherit;
        transition: filter 0.2s;
    }

    .lcars-pill-button.amber { background: var(--lcars-amber, #ff9900); color: #000; }
    .lcars-pill-button:hover { filter: brightness(1.2); }

    .board-header {
        display: flex;
        gap: 0.5rem;
        margin-bottom: 1.5rem;
        font-size: 0.9rem;
        text-transform: uppercase;
        font-weight: bold;
    }

    .path-segment.amber { color: var(--lcars-amber, #ff9900); }
    .path-segment.gray { color: #666; }
    
    .header-actions {
        margin-left: auto;
        display: flex;
        gap: 0.5rem;
    }
    
    .lcars-pill-button.indigo { background: #5544ff; color: #fff; font-size: 0.7rem; }
    .lcars-pill-button.rose { background: #ff4488; color: #fff; font-size: 0.7rem; }

    .lanes-container {
        display: flex;
        gap: 1.5rem;
        flex: 1;
        overflow-x: auto;
        padding-bottom: 1rem;
        align-items: flex-start;
    }

    .lane {
        flex: 1;
        min-width: 300px;
        max-width: 400px;
        background: rgba(30,30,40,0.6);
        border-radius: 8px 8px 4px 4px;
        display: flex;
        flex-direction: column;
        border-top: 10px solid var(--lcars-amber, #ff9900);
        box-shadow: 0 4px 15px rgba(0,0,0,0.3);
        height: 100%;
    }

    .lane-header {
        padding: 1rem;
        background: rgba(255, 153, 0, 0.05);
        display: flex;
        align-items: center;
        gap: 0.75rem;
        border-bottom: 1px solid rgba(255, 153, 0, 0.2);
    }

    .lane-title {
        font-weight: 900;
        color: var(--lcars-amber, #ff9900);
        flex: 1;
        font-size: 1.1rem;
    }

    .lane-stats {
        background: rgba(255, 153, 0, 0.2);
        color: var(--lcars-amber, #ff9900);
        padding: 2px 8px;
        border-radius: 4px;
        font-size: 0.8rem;
        font-weight: bold;
    }

    .icon-btn {
        background: none;
        border: none;
        font-size: 1.5rem;
        cursor: pointer;
        padding: 0 5px;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .icon-btn.amber { color: var(--lcars-amber, #ff9900); }

    .tasks-list {
        padding: 1rem;
        display: flex;
        flex-direction: column;
        gap: 1rem;
        overflow-y: auto;
        flex: 1;
    }

    .task-card {
        background: rgba(255, 255, 255, 0.05);
        padding: 1rem;
        border-radius: 6px;
        border-left: 5px solid #444;
        transition: all 0.2s;
        cursor: grab;
    }

    .task-card:hover {
        background: rgba(255, 255, 255, 0.08);
        border-left-color: var(--lcars-amber, #ff9900);
        transform: translateY(-2px);
    }

    .task-title {
        font-weight: 700;
        margin-bottom: 0.5rem;
        color: #fff;
    }

    .task-desc {
        font-size: 0.85rem;
        color: #999;
        margin-bottom: 1rem;
        line-height: 1.4;
    }

    .task-footer {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        font-size: 0.7rem;
        color: #666;
        text-transform: uppercase;
        font-weight: bold;
    }

    .status-indicator {
        width: 8px;
        height: 8px;
        border-radius: 50%;
        background: #444;
    }

    .status-indicator.todo { background: #888; }
    .status-indicator.inprogress { background: var(--lcars-blue, #3366ff); box-shadow: 0 0 5px var(--lcars-blue); }
    .status-indicator.done { background: #00ff00; box-shadow: 0 0 5px #00ff00; }

    .task-id { flex: 1; }

    .small-btn {
        background: none;
        border: 1px solid rgba(255,255,255,0.1);
        color: #666;
        font-size: 0.6rem;
        padding: 2px 6px;
        border-radius: 3px;
        cursor: pointer;
        text-transform: uppercase;
        font-weight: bold;
    }

    .small-btn:hover {
        background: rgba(255,0,0,0.2);
        color: #ff3333;
        border-color: #ff3333;
    }

    /* Scrollbar Styling */
    .lanes-container::-webkit-scrollbar,
    .tasks-list::-webkit-scrollbar {
        width: 6px;
        height: 6px;
    }

    .lanes-container::-webkit-scrollbar-track,
    .tasks-list::-webkit-scrollbar-track {
        background: rgba(0,0,0,0.1);
    }

    .lanes-container::-webkit-scrollbar-thumb,
    .tasks-list::-webkit-scrollbar-thumb {
        background: rgba(255, 153, 0, 0.2);
        border-radius: 3px;
    }
</style>
