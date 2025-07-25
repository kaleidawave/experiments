### Operation

```
(x (a b) (c d))
```

```
Expression {
    on: "x",
    arguments: [
        Expression {
            on: "a",
            arguments: [
                Expression {
                    on: "b",
                    arguments: [],
                },
            ],
        },
        Expression {
            on: "c",
            arguments: [
                Expression {
                    on: "d",
                    arguments: [],
                },
            ],
        },
    ],
}
```

### Operations

```
_*_ 4
_+_ 3
---
x * y + 2
```

```
Expression {
    on: "+",
    arguments: [
        Expression {
            on: "*",
            arguments: [
                Expression {
                    on: "x",
                    arguments: [],
                },
                Expression {
                    on: "y",
                    arguments: [],
                },
            ],
        },
        Expression {
            on: "2",
            arguments: [],
        },
    ],
}
```

### Adjacency

```
_*_ 4 (adjacent)
---
3xy
```

```
Expression {
    on: "*",
    arguments: [
        Expression {
            on: "*",
            arguments: [
                Expression {
                    on: "3",
                    arguments: [],
                },
                Expression {
                    on: "x",
                    arguments: [],
                },
            ],
        },
        Expression {
            on: "y",
            arguments: [],
        },
    ],
}
```

### Adjacency 2

```
_*_ 4 (adjacent)
_+_ 3
---
x(y + 2)
```

```
Expression {
    on: "*",
    arguments: [
        Expression {
            on: "x",
            arguments: [],
        },
        Expression {
            on: "+",
            arguments: [
                Expression {
                    on: "y",
                    arguments: [],
                },
                Expression {
                    on: "2",
                    arguments: [],
                },
            ],
        },
    ],
}
```