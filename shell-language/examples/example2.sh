# Auto merging of stderr and stdout (unless ends with something)
let output = run ./ezno x
let diagnostics = rbefore $output '---'
let statistics = rafter $output '---'

run curl ...