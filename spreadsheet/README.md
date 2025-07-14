Spreadsheet web app

Run with

```shell
npm i
npm run
```

### Features

- Tab and shift keys to move around cells
- Syncs with server via web-sockets
- Expression evaluation (via [mathematics-parser](https://jsr.io/@bengineering/mathematics-parser)). Can reference cells `=2*B1 + B2`
- Resizable columns
- Horizontally drag numeric inputs to change value

### To implement

- Save tables
- Send table information on new connection
- Recursive evaluation (and cyclic reference checking)
- Resizable **rows**
- Message sanitisation. More message checking (bad expressions, invalid positions) and error handling
