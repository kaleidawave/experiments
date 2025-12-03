### Learnings

- There are a number of commands
- Labels can denote positions into the commands
- The system is designed to run in one pass
	- Variables can only be referenced after they are defined
- Shapes can have a number of text commands
- Commands are on one line including continuations from trailing backslash
	- Can parse using `LinesWithContinuations`
- Commands start with one of a given keyword. Arguments specified afterards are in quotes and whitespace seperated?