# TOS Architectural Shell Hook for Fish (Alpha-2.1)
# Implements OSC 1337 and 9002/9003 emission for Tactical Operating System

function tos_preexec --on-event fish_preexec
    set -g TOS_LAST_COMMAND $argv[1]
    printf "\e]1337;RemoteHost=%s@%s\a" $USER (hostname)
end

function tos_precmd --on-event fish_postexec
    set -l status_code $status
    
    if test -n "$TOS_LAST_COMMAND"
        and test "$TOS_LAST_COMMAND" != "cd"
        and test "$TOS_LAST_COMMAND" != "EXEC"
        
        # Replace newlines/semicolons with underscores
        set -l safe_cmd (string replace -a ";" "_" $TOS_LAST_COMMAND | string replace -a "\r" "" | string sub -l 100)
        
        printf "\e]9002;%s;%s\a" "$safe_cmd" "$status_code"
    end
    
    # Emit cwd via OSC 9003
    printf "\e]9003;%s\a" "$PWD"
    
    set -e TOS_LAST_COMMAND
end
