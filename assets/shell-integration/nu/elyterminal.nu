# ELYTerminal: native prompt marks, percent-safe folders and control-free titles.
let v = (version)
if $v.major == 0 and ($v.minor < 116 or ($v.minor == 116 and $v.patch < 1)) {
    error make {msg: 'Nushell integration requires Nushell 0.116.1 or later'}
}

$env.config.shell_integration.osc2 = false
$env.config.shell_integration.osc133 = true
$env.config.shell_integration.osc633 = false
$env.config.shell_integration.osc7 = false
$env.config.shell_integration.osc9_9 = false

def _ely_file_uri [path: string, windows: bool] {
    let path = if $windows { $path | str replace --all '\' '/' } else { $path }
    let encoded = ($path | url encode --all | str replace --all '%2F' '/')
    let slash = if ($path | str starts-with '/') { '' } else { '/' }
    $'file://localhost($slash)($encoded)'
}

$env.config.hooks.pre_prompt = ($env.config.hooks.pre_prompt | append {||
    let uri = (_ely_file_uri $env.PWD ($nu.os-info.name == 'windows'))
    let title = ($env.PWD | path basename | default --empty '/' | str replace --all --regex '[[:cntrl:]]' '')
    print -n $"(char -i 27)]7;($uri)(char -i 7)(char -i 27)]2;($title)(char -i 7)"
})

$env.config.hooks.pre_execution = ($env.config.hooks.pre_execution | append {||
    let title = (commandline | str replace --all --regex '[[:cntrl:]]' '')
    print -n $"(char -i 27)]2;($title)(char -i 7)"
})
