let my_value = literal 2812
let output = with my_value $my_value run node --print "process.env.my_value"
let output = trim $output
echo $output