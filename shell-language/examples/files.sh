for files "examples/*" each
	let content = read $file
	echo "---"
	echo $file
	echo $content
	echo "---"
	echo