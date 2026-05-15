function _i
    command -v $argv[1] >/dev/null 2>&1
end

function _l
    set _m (printf "{\"hook\": \"%s\", \"value\": %s}" $argv[1] $argv[2] | od -An -v -tx1 | tr -d " \n")
    printf '\033\120\044\144%s\234' $_m
end

function _e
    _l RemoteWarpificationIsUnavailable $argv[1]
end

function _d
    set -l PK ""

    if _i brew
      set PK "homebrew"
    end

    printf '{"operating_system": "Darwin", "package_manager": "%s", "shell": "fish", "root_access": "no_root_access", "writable_home": %s}' "$PK" $( [ -w ~ ] && echo true || echo false )
end

function _c
    set -g T "$HOME/.warp/tmux/execute_tmux.sh"
    if _i "$T"
        _l SshTmuxInstaller "\"warp\""
    else if _i tmux
        set T "tmux"
        _l SshTmuxInstaller "\"user\""
    end

    if test -n "$T"
        $T -V | awk '{print $2}' | read V;
        if test -z "$V"
            _e "\"TmuxFailed\""
        else if test (printf '%s\n' "$V" "2.9" | sort -V | tail -n1) = "2.9"
            set -l D (_d)
            _e "{\"UnsupportedTmuxVersion\": $D}"
        else;
          return 0
        end
    else;
            set -l D (_d)
        _e "{\"TmuxNotInstalled\": $D}"
    end
    return 1
end

_c; and $T -Lwarp -CC; and exit
