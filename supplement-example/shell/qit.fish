function __qit_do_completion
    set cmd (commandline -c)
    set cmd_arr (string split ' ' $cmd)
    set cur "$cmd_arr[-1]"

    if [ -z "$cur" ]
        # preserve the last white space
        echo fish $cmd "''" | xargs PLACEHOLDER_FOR_BIN_PATH
    else
        echo fish $cmd | xargs PLACEHOLDER_FOR_BIN_PATH
    end

    if [ "$status" != "0" ]
        complete -C "'' $cur"
    end
end

complete -k -c qit -x -a "(__qit_do_completion)"
