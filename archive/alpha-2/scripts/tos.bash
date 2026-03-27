# TOS Architectural Shell Hook for Bash (Alpha-2.1)
# Implements OSC 1337 and 9002/9003 emission for Tactical Operating System

# Track the last executed command
TOS_LAST_COMMAND=""

function _tos_preexec() {
    # Store the command before execution
    TOS_LAST_COMMAND="$1"
    
    # Indicate command start to TOS (optional, standard 1337 hooks)
    printf "\e]1337;RemoteHost=%s@%s\a" "$USER" "$HOSTNAME"
}

function _tos_precmd() {
    local status_code=$?
    
    # Do not emit for empty commands or internal TOS commands
    if [[ -z "$TOS_LAST_COMMAND" || "$TOS_LAST_COMMAND" == "cd" || "$TOS_LAST_COMMAND" == "EXEC" ]]; then
        TOS_LAST_COMMAND=""
        return
    fi
    
    # Extract only the command name, prevent control chars
    local safe_cmd=$(echo "$TOS_LAST_COMMAND" | tr -d '\n\r;' | head -c 100)
    
    # Emit OSC 9002 for command completion: <command>;<exit_status>
    printf "\e]9002;%s;%s\a" "$safe_cmd" "$status_code"
    
    # Always emit directory context via OSC 9003
    printf "\e]9003;%s\a" "$PWD"
    
    # Reset for next prompt
    TOS_LAST_COMMAND=""
}

# Attach to Bash's DEBUG trap and PROMPT_COMMAND
# We use a trap to simulate preexec in bash
trap '_tos_preexec "$BASH_COMMAND"' DEBUG
PROMPT_COMMAND="_tos_precmd; $PROMPT_COMMAND"
