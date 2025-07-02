let x = constant "hi"
if literal $x
	echo "found hi"

set x = constant ""
if literal $x
	echo "found hello"