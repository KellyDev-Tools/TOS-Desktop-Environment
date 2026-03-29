# TOS Heuristic Service (`tos-heuristicd`)

The predictive intelligence layer for smart command completion and intuitive interaction.

## Design Drivers
- **Architecture Spec §31**: "Predictive Fillers & Intuitive Interaction" define the autocomplete-to-chip logic.
- **Architecture Spec §4**: Specifies the service boundary for real-time suggestions.

## Responsibilities
- Levenshtein-based typo correction for shell commands.
- Context-aware path completion suggestions.
- Heuristic sector labeling and suggestion ranking.
