_qit()
{
    args=${COMP_WORDS[@]:0:$((COMP_CWORD+1))}
    cur="${COMP_WORDS[COMP_CWORD]}"
    defaults=()

    if [[ -z "$cur" ]]; then
        custom=($( echo bash $args "''" | xargs PLACEHOLDER_FOR_BIN_PATH))
    else
        custom=($( echo bash $args | xargs PLACEHOLDER_FOR_BIN_PATH))
    fi

    if [ "$?" != "0" ]; then
        defaults=( $(compgen -f -- "$cur") )
    fi

    COMPREPLY=( "${custom[@]}" "${defaults[@]}" )
} &&
    complete -F _qit qit

# ex: filetype=sh
