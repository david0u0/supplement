function __do_completion
    set cmd (commandline -j)
    set cmd_arr (string split ' ' $cmd)
    if [ -z "$cmd_arr[-1]" ]
        # preserve the last white space
        eval "PLACEHOLDER_FOR_BIN_PATH $cmd ''"
    else
        eval PLACEHOLDER_FOR_BIN_PATH $cmd
    end
end

complete -k -c qit -x -a "(__do_completion)"
