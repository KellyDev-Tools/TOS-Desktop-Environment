# TOS Architectural Shell Hook for Zsh (Alpha-2.1)
# Implements OSC 1337 and 9002/9003 emission for Tactical Operating System

typeset -g TOS_LAST_COMMAND=""

function tos_preexec() {
    TOS_LAST_COMMAND=$1
    # Standard terminal identification
    printf "\e]1337;RemoteHost=%s@%s\a" "$USER" "$HOST"
}

function tos_precmd() {
    local status_code=$?

    if [[ -n "$TOS_LAST_COMMAND" && "$TOS_LAST_COMMAND" != "cd" && "$TOS_LAST_COMMAND" != "EXEC" ]]; then
        # Safely remove control characters and truncate
        local safe_cmd="${TOS_LAST_COMMAND[1,100]}"
        safe_cmd=${safe_cmd//[;]/_}
        safe_cmd=${safe_cmd//[$'\r\n']/}

        # Emit completion OSC 9002
        printf "\e]9002;%s;%s\a" "$safe_cmd" "$status_code"
    fi

    # Emit cwd via OSC 9003
    printf "\e]9003;%s\a" "$PWD"
    
    TOS_LAST_COMMAND=""
}

# Attach to Zsh preexec and precmd hooks
autoload -Uz add-zsh-hook
add-zsh-hook preexec tos_preexec
add-zsh-hook precmd tos_precmd
