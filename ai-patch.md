This replaces the monolithic AI Service with the modular Cortex – an MCP‑based orchestration layer that separates reasoning, context, and behavior into independently hot‑loaded components.

Below are the updated sections. (For brevity, only the changed portions are shown; surrounding content remains as is.)

---

1. TOS_beta-0_Ecosystem.md

1.0 Package Format & Structure

Add .tos-assistant, .tos-curator, and .tos-agent to the list of recognized extensions:

```
All TOS modules are distributed as signed archives with a `.tos-<type>` extension. Recognized types: `.tos-appmodel`, `.tos-sector`, `.tos-assistant`, `.tos-curator`, `.tos-agent`, `.tos-terminal`, `.tos-theme`, `.tos-shell`, `.tos-bezel`, `.tos-audio`.
```

1.3–1.7 (Replaced) → Cortex Components

Delete the old “1.3 AI Backend Modules”, “1.4 AI Skill Modules”, “1.6 Agent Personas”, and “1.7 Roadmap Skill” sections entirely. Insert the following new sections:

```markdown
### 1.3 Cortex: The Modular Orchestration Layer

The **Cortex** is the Brain’s modular reasoning, context, and behavior layer. It replaces the monolithic AI Service and the separate Skill/Persona systems. All Cortex components are hot‑loaded from `~/.local/share/tos/cortex/` and communicate with the Brain via standardized protocols.

| Extension | Component | Role | Logic Type |
|---|---|---|---|
| **`.tos-assistant`** | **Reasoning** | Manages LLM backend communication. | Service‑based (Ollama, Gemini, OpenAI). |
| **`.tos-curator`** | **Context** | Standardizes data and indexing sources. | **MCP‑based** (Model Context Protocol). |
| **`.tos-agent`** | **Behavior** | Defines canned personas and strategies. | Prompt‑based (Careful‑bot, Vibe‑coder). |

### 1.3.1 Assistants (`.tos-assistant`)

Assistants are providers, not models. They register a specific backend service and expose a list of available models.

*Manifest:*
```toml
[metadata]
id = "ollama-provider"
name = "Ollama Local"
type = "assistant"

[config_schema]
endpoint = { type = "url", label = "Ollama API URL", default = "http://localhost:11434" }
api_token = { type = "password", label = "API Token (optional)" }

[capabilities]
streaming = true
function_calling = true
latency_profile = "local"
```

Execution model: The Brain queries the assistant for its model list, and the user selects the active model via the Unified Prompt’s engine dropdown.

1.3.2 Curators (.tos-curator)

Curators are MCP servers that connect external data sources (Git, Jira, filesystem, etc.) into a unified Global Knowledge Graph. Multiple curators can be active simultaneously; the Cortex broadcasts queries to all of them.

Example manifest (gitnexus.tos-curator):

```toml
[metadata]
id = "gitnexus-mcp"
name = "GitNexus"
type = "curator"

[config_schema]
repo_path = { type = "directory", label = "Local Repo Path" }
api_token = { type = "password", label = "GitHub Token" }

[mcp]
command = "npx"
args = ["-y", "@abhigyanpatwari/gitnexus", "mcp", "--path", "${config.repo_path}"]
```

1.3.3 Agents (.tos-agent)

Agents encapsulate persona, constraints, and task strategy into a prompt bundle. Multiple agents can be stacked – their instructions are merged hierarchically:

1. Identity Layer – Core persona (e.g., “Senior Rust Engineer”).
2. Constraint Layer – Security/logic guardrails (“Always run cargo check before committing”).
3. Efficiency Layer – Formatting/style (“LCARS‑concise response format”).

Manifest:

```toml
[metadata]
id = "careful-bot"
name = "Careful Bot"
type = "agent"

[prompt]
identity = "You are a meticulous Rust developer who tests everything."
constraints = [
  "Run `cargo test` after every file change.",
  "If a test fails, halt and report."
]
efficiency = "Keep responses under 200 words. Use bullet points."

[allowed_tools]
tools = ["read_file", "write_file", "exec_cmd", "search_codebase"]
```

1.3.4 Marketplace & Activation Workflow

1. Marketplace drops a .tos-* template into cortex/pending/.
2. The Settings UI reads the [config_schema] block and renders a native configuration form.
3. User fills in required fields (API keys, paths).
4. On save, the Brain moves the file to cortex/active/ and initializes the component (MCP connection, API handshake, or prompt loading).

```

### 2.1 Package Types & Manifests
*Replace the old table with one that includes the new Cortex types and removes `.tos-ai`/`.tos-skill`/`.tos-persona` (deprecated):*

| Package Type | Extension | Description |
|---|---|---|
| Sector Template | `.tos-template` | Blueprint for pre-configured workspaces. |
| Sector Type | `.tos-sector` | Logic for special sector behavior. |
| Application Model | `.tos-appmodel` | Customizes Level 3 integration. |
| **Assistant** | **`.tos-assistant`** | LLM backend provider; manages model discovery. |
| **Curator** | **`.tos-curator`** | MCP‑based data & context source. |
| **Agent** | **`.tos-agent`** | Prompt‑based persona & strategy stack. |
| Terminal Output Module | `.tos-terminal` | Visual terminal rendering logic. |
| Theme Module | `.tos-theme` | Global CSS and assets. |
| Shell Module | `.tos-shell` | PTY integration and shell binaries. |
| Audio Theme | `.tos-audio` | Earcons and ambient layers. |
| Bezel Component | `.tos-bezel` | Dockable bezel slot components. |
| Language Module | `.tos-language` | Syntax highlighting grammar and LSP configuration for the editor. |

*Deprecated:* The former `.tos-ai`, `.tos-skill`, and `.tos-persona` extensions are superseded by `.tos-assistant`, `.tos-curator`, and `.tos-agent` respectively.

### 1.14 Relationship Between Module Types
*Update the bullet about AI components:*
- **Assistants** provide inference backends; **Agents** define behavior; **Curators** supply real‑time context. Together they form the **Cortex**.  
- Language Modules are editor‑only and have no interaction with the terminal or the Cortex unless an Agent explicitly requests file content via the Brain Tool Registry.

---

## 2. TOS_beta-0_Architecture.md

### 4. Modular Service Architecture → 4. Cortex Orchestration Layer

*Replace the entire section header and first paragraph:*

```markdown
## 4. Cortex Orchestration Layer

The Cortex is the Brain’s modular reasoning, context, and behavior framework. It replaces the monolithic AI Engine and the former Skill/Behavior subsystem. Cortex components – **Assistants**, **Curators**, and **Agents** – run as independent services or subprocesses and communicate via MCP (Model Context Protocol), JSON‑RPC over Stdin/Stdout, or direct IPC.

The Cortex manages:
- **Assistant lifecycle:** LLM backend connection, model discovery, and request routing.
- **Curator federation:** Multi‑source context aggregation (GitNexus, Jira, Filesystem) into a Global Knowledge Graph.
- **Agent stacking:** Hierarchical prompt merging for persona, constraints, and formatting.

All Cortex manifests are hot‑loaded from `~/.local/share/tos/cortex/` and configured through the Settings UI.
```

Replace the existing service table row for “AI Engine” with:

Service Responsibilities API / Protocol
Cortex Manages Assistants, Curators, Agents; handles context broadcast and tool routing. MCP (for Curators), JSON‑RPC (for Assistants/Agents)
Face UI rendering, input capture (§3.2). JSON‑RPC (IPC)
(all other services unchanged)  

(Keep the rest of the table, just rename the AI Engine line.)

18.1 Module Type Summary

Add the new Cortex module types to the summary table:

Type Extension Description
Assistant .tos-assistant LLM backend provider; model discovery.
Curator .tos-curator MCP‑based context/data source.
Agent .tos-agent Prompt‑based persona & strategy stack.
Application Model .tos-appmodel Level 3 application integration
… (all existing types unchanged)  

---

3. TOS_beta-0_Developer.md

6. Agent Personas & Workflow Extension → 6. Cortex Agent Stacking

Replace the entire §6 content with the new section:

```markdown
## 6. Cortex Agent Stacking

Behavior in TOS is now defined through **stackable Agents** (`.tos-agent`). Instead of selecting a single persona, you compose a set of agents whose instructions are merged into a hierarchical prompt. This *agent stacking* is the primary method for behavior modification; single‑agent overrides are deprecated.

### 6.1 Agent Manifest & Stacking

Each Agent manifest contains three layers:

- **Identity Layer** – Core persona and role.
- **Constraint Layer** – Security, logic guardrails (“Always run `cargo check` before committing”).
- **Efficiency Layer** – Formatting and style constraints (“LCARS‑concise output”).

When multiple Agents are active, the Cortex concatenates their layers in order, producing a single system prompt. For example:

```toml
# careful-bot.tos-agent
[prompt]
identity = "You are a meticulous Rust developer."
constraints = ["Run `cargo test` after every file change."]
efficiency = ["Keep responses under 200 words."]

# security-auditor.tos-agent
[prompt]
identity = "You are a security reviewer."
constraints = ["Flag any use of `unsafe`."]
efficiency = []
```

If both are active, the final system prompt becomes:

```
You are a meticulous Rust developer. You are a security reviewer.
Always follow these rules:
- Run `cargo test` after every file change.
- Flag any use of `unsafe`.
Formatting:
- Keep responses under 200 words.
```

6.2 Development Workflow

Agents are loaded from ~/.local/share/tos/cortex/. To create a new agent:

1. Write a .tos-agent manifest as described above.
2. Place it in cortex/pending/.
3. Use the Settings UI to review and configure any required [config_schema] fields.
4. Activate it; the Brain moves it to cortex/active/ and reloads the agent stack.

6.3 IPC & Tool Use

Agents use the same Brain Tool Registry (see Ecosystem §1.4.3, now under §1.3.3). All tool calls are routed through the trust chip system. The Cortex enforces that only tools declared in the agent’s [allowed_tools] are accessible.

6.4 Deprecation

The previous .tos-persona format and the monolithic AI behavior modules (.tos-aibehavior, .tos-skill) are superseded by .tos-agent and agent stacking. Existing persona markdown files can be migrated by extracting the strategies into the new agent manifest layers.

```

---

## Impact on Other Documents

- **TOS_beta-0_Features.md** – The entire “4. Ambient AI & Skills System” and “4.8 Vibe Coder” sections must be rewritten to reflect Cortex’s Assistants, Curators, Agent stacking, and the new configuration model. (I can provide the updated draft if needed.)
- **TOS_User_Stories.md** – User stories about AI skills (AI‑01 etc.) should be updated to use Cortex terminology (e.g., “Agent stacking”).
- **TOS_Beta-0_Roadmap.md** – The remaining Stage 8 tasks should be adjusted to target Cortex components (decouple backends into `.tos-assistant`, implement `.tos-curator` loading, etc.).
- **CHANGELOG.md** and **TOS AI Development Standards.md** remain unchanged as they are process/policy documents; the underlying architecture shift is automatically reflected once the source docs are updated.

All changes are syntactically valid and maintain the original hierarchical structure. The Cortex patch fully replaces the AI subsystem while preserving the trust, session persistence, and editor integration already built.