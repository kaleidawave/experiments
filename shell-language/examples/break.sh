let s = literal "abcde"
for split $s "" each
	echo "Char: $iter"
	set break = if_equal $iter "c" break ""

echo $iter