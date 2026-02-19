#compdef qit

_qit() {

    local -a expl=()

    local -a candidates
    local cur=${words[-1]}

    if [[ -z "$cur" ]]; then
        candidates=("${(@f)$( echo zsh ${words[1,CURRENT]} "''" | xargs PLACEHOLDER_FOR_BIN_PATH )}")
    else
        candidates=("${(@f)$( echo zsh ${words[1,CURRENT]} | xargs PLACEHOLDER_FOR_BIN_PATH )}")
    fi

    local group=''
    local -a expl=()
    local -a values
    local -a descs
    for line in $candidates; do
        echo $line >> /tmp/sup.log
        if [[ $line == $'\t'* ]]; then
            parts=(${(@ps:\t:)line})
            values+=("${parts[1]}")
            descs+=("${parts[2]}")
        else
            if [[ ! -z "$group" ]]; then
                _wanted $group expl $group compadd -d descs -- ${values}
            fi

            group=$line
            values=()
            descs=()
        fi
    done
}

_qit "$@"
