#!/bin/sh
# TOS Default Ash Configuration

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

alias grh='git reset --hard'
alias grm='git reset --mixed'
alias grs='git reset --soft'
