let output = literal "hi" then concatenate_separator "" $piped " Ben"
echo $output

let output = literal "hi" then repeat 10
echo $output