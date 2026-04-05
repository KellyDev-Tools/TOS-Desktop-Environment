#!/bin/bash
# TOS Default Bash Configuration

# Git Aliases
alias gap='git add --patch'
alias gar='git apply --reject'
alias gbv='git branch -vv'
alias gca='git commit --amend -C HEAD'
alias gclean='git clean -Xdf'
alias gco='git checkout'
alias gdh='git diff HEAD'
alias gds='git diff --staged'
alias gdu='git diff'
alias gfa='git fetch --all'
alias gp='git pull --stat'
alias gpr='git pull --stat --rebase'
alias gr='git remote -v'
alias gra='git rebase --abort'
alias grc='git rebase --continue'
alias gri='git rebase --interactive'
alias gs='git status --short'
alias gsi='git status --short --ignored'
alias gsl='git status'
alias gsli='git status --ignored'

alias gcm='git commit -m "__CURSOR__"'
alias gcs='git __CURSOR__ --compact-summary'

alias grh='git reset --hard'
alias grm='git reset --mixed'
alias grs='git reset --soft'

# Bash completions
export BASH_COMPLETION_USER_DIR="${HOME}/.bash_completions"
mkdir -p "${BASH_COMPLETION_USER_DIR}"

if command -v kubectl >/dev/null 2>&1 && [ ! -e "${BASH_COMPLETION_USER_DIR}/kubectl.bash" ]; then
	kubectl completion bash > "${BASH_COMPLETION_USER_DIR}/kubectl.bash"
fi
if command -v rustup >/dev/null 2>&1; then
	if [ ! -e "${BASH_COMPLETION_USER_DIR}/rustup.bash" ]; then
		rustup completions bash rustup > "${BASH_COMPLETION_USER_DIR}/rustup.bash"
	fi
	if [ ! -e "${BASH_COMPLETION_USER_DIR}/cargo.bash" ]; then
		rustup completions bash cargo > "${BASH_COMPLETION_USER_DIR}/cargo.bash"
	fi
fi
if command -v just >/dev/null 2>&1 && [ ! -e "${BASH_COMPLETION_USER_DIR}/just.bash" ]; then
	just --completions bash > "${BASH_COMPLETION_USER_DIR}/just.bash"
fi
if command -v delta >/dev/null 2>&1 && [ ! -e "${BASH_COMPLETION_USER_DIR}/delta.bash" ]; then
	delta --generate-completion bash > "${BASH_COMPLETION_USER_DIR}/delta.bash"
fi

for completion_script in "${BASH_COMPLETION_USER_DIR}"/*.bash; do
    if [ -f "$completion_script" ]; then
        . "$completion_script"
    fi
done
