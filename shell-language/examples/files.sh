for files "examples/*.sh" each
	let content = read $file
	echo "---"
	echo $file
	echo $content
	echo "---"
	echo