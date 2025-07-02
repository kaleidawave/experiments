let size = constant "123,4567"
let size = replace $size "," ""
let size = format_number $size
echo $size

let size = constant "1234.56789"
let size = format_number $size
echo $size