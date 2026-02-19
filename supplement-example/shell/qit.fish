function __do_completion
    set cmd (commandline -j)
    set cmd_arr (string split ' ' $cmd)
    if [ -z "$cmd_arr[-1]" ]
        # preserve the last white space
        echo fish $cmd "''" | xargs PLACEHOLDER_FOR_BIN_PATH
    else
        echo fish $cmd | xargs PLACEHOLDER_FOR_BIN_PATH
    end
end

complete -k -c qit -x -a "(__do_completion)"
