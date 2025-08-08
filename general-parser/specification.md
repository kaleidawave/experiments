### Calling

```
(x (a b) (c d))
```

```
(x (a b) (c d))
```

### Binary operation with precedence

```
_ * _ #4
_ + _ #3
---
x * y + 2
```

```
(+ (* x y) 2)
```

### Alphabetic operation

```
_ and _ #4
_ or _ #3
---
x and y or 2
```

```
(or (and x y) 2)
```

### Postfix unary operator

```
fact: _! #3
---
3!
```

```
(! 3)
```

### Prefix unary operator

```
not1: !_ #3
not2: not _ #3
---
and (not a) (!b)
```

```
(and (not a) (! b))
```

### Prefix ternary operation

```
if: if _ then _ else _
_ and _ #4
---
if x and y then 2 else 4
```

```
(if (and x y) 2 4)
```

### Postfix ternary operation

```
mod_eq: _ = _ (mod _)
---
x = y (mod z)
```

```
(mod_eq x y z)
```

### Ternary operations binary fallback

```
mod_eq: _ = _ (mod _)
eq: _ = _
---
x = y
```

```
(= x y)
```

### Adjacency

```
_*_ #4 (adjacent)
---
23xyz
```

```
(* 23 (* z (* y x)))
```

### Adjacency with parenthesised/grouped expression

```
_*_ #4 (adjacent)
_+_ #3
---
x(y + 2) + (6 + 7)z
```

```
(+ (* x (+ y 2)) (* (+ 6 7) z))
```

### Adjacency with function list

```
_*_ #4 (adjacent)
sin (function)
---
5sin(6)
```

```
(* 5 (sin 6))
```
