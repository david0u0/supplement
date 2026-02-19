_qit()
{
    args=${COMP_WORDS[@]:0:$((COMP_CWORD+1))}
    cur="${COMP_WORDS[COMP_CWORD]}"

    if [[ -z "$cur" ]]; then
        COMPREPLY=($( echo bash $args "''" | xargs PLACEHOLDER_FOR_BIN_PATH))
    else
        COMPREPLY=($( echo bash $args | xargs PLACEHOLDER_FOR_BIN_PATH))
    fi
} &&
    complete -F _qit qit

# ex: filetype=sh
