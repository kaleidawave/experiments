let acc = constant ""
for git log --format=%H each
  let branch = git name-rev --name-only --refs="refs/heads/*" $ref
  let branch = if_contains $branch "~" "" $branch
  let acc = concatenate $acc $branch

for literal $acc each
  echo $iter
