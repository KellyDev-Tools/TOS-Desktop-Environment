#!/bin/zsh
# TOS Default Zsh Configuration

# Zsh specific config options
setopt AUTO_CD
setopt AUTO_NAME_DIRS
setopt HIST_IGNORE_SPACE
setopt HIST_NO_STORE
setopt INC_APPEND_HISTORY
setopt INTERACTIVE_COMMENTS
setopt LIST_ROWS_FIRST
setopt PROMPT_SUBST
setopt PUSHD_IGNORE_DUPS
setopt PUSHD_SILENT

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

alias gl='git log --oneline --decorate -n 10'

alias gcm='git commit -m "__CURSOR__"'
alias gcs='git __CURSOR__ --compact-summary'

alias grh='git reset --hard'
alias grm='git reset --mixed'
alias grs='git reset --soft'

# Zsh completions
typeset -g env_completion_dir="${HOME}/.zsh/completions"
mkdir -p "${env_completion_dir}"

if [[ -n "$(whence kubectl)" && ! -e "${env_completion_dir}/_kubectl" ]]; then
	kubectl completion zsh > "${env_completion_dir}/_kubectl"
fi
if [[ -n "$(whence rustup)" ]]; then
	if [[ ! -e "${env_completion_dir}/_rustup" ]]; then
		rustup completions zsh rustup > "${env_completion_dir}/_rustup"
	fi
	if [[ ! -e "${env_completion_dir}/_cargo" ]]; then
		rustup completions zsh cargo > "${env_completion_dir}/_cargo"
	fi
fi
if [[ -n "$(whence just)" && ! -e "${env_completion_dir}/_just" ]]; then
	just --completions zsh > "${env_completion_dir}/_just"
fi
if [[ -n "$(whence delta)" && ! -e "${env_completion_dir}/_delta" ]]; then
	delta --generate-completion zsh > "${env_completion_dir}/_delta"
fi

typeset -Uga fpath
fpath=("${env_completion_dir}" ${fpath})

# Reinitialize completion with the new fpath
autoload -U compinit
compinit
