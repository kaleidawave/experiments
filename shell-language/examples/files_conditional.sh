for files "examples/*.sh" each
	let content = read $file
	let content = if_equal $file "examples/files.sh" $content
	echo $content