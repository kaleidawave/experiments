### Operation

```
(x (a b) (c d))
```

```
(x (a b) (c d))
```

### Operations

```
_*_ 4
_+_ 3
---
x * y + 2
```

```
(+ (* x y) 2)
```

### Adjacency

```
_*_ 4 (adjacent)
---
3xy
```

```
(* (* 3 x) y)
```

### Adjacency 2

```
_*_ 4 (adjacent)
_+_ 3
---
x(y + 2)
```

```
(* x (+ y 2))
```