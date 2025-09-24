# armstrong
## getDigits
```
block: f1.b0
in: [ConstProp { name: "getDigitsone", val: Int { v: 1 } }, ConstProp { name: "getDigitsten", val: Int { v: 10 } }, ConstProp { name: "getDigitszero", val: Int { v: 0 } }]
out: [ConstProp { name: "getDigitsone", val: Int { v: 1 } }, ConstProp { name: "getDigitsten", val: Int { v: 10 } }, ConstProp { name: "getDigitszero", val: Int { v: 0 } }]
```
these consts are const. they might persist with recursive calls.

```
block: f1.b1
in: [ConstProp { name: "getDigitsone", val: Int { v: 1 } }, ConstProp { name: "getDigitsten", val: Int { v: 10 } }, ConstProp { name: "getDigitszero", val: Int { v: 0 } }]
out: [ConstProp { name: "getDigitsone", val: Int { v: 1 } }, ConstProp { name: "getDigitsten", val: Int { v: 10 } }, ConstProp { name: "getDigitszero", val: Int { v: 0 } }]
```
nothing introduced

```
block: f1.b2
in: [ConstProp { name: "getDigitsone", val: Int { v: 1 } }, ConstProp { name: "getDigitsten", val: Int { v: 10 } }, ConstProp { name: "getDigitszero", val: Int { v: 0 } }]
out: [ConstProp { name: "getDigitsone", val: Int { v: 1 } }, ConstProp { name: "getDigitsten", val: Int { v: 10 } }, ConstProp { name: "getDigitszero", val: Int { v: 0 } }]
```
nothing introduced

```
block: f1.b3
in: [ConstProp { name: "getDigitsone", val: Int { v: 1 } }, ConstProp { name: "getDigitsten", val: Int { v: 10 } }, ConstProp { name: "getDigitszero", val: Int { v: 0 } }]
out: [ConstProp { name: "getDigitsone", val: Int { v: 1 } }, ConstProp { name: "getDigitsten", val: Int { v: 10 } }, ConstProp { name: "getDigitszero", val: Int { v: 0 } }]
```
nothing introduced

## mod
```
block: f2.b0
in: [ConstProp { name: "powerone", val: Int { v: 1 } }, ConstProp { name: "powerten", val: Int { v: 10 } }, ConstProp { name: "powerzero", val: Int { v: 0 } }]
out: [ConstProp { name: "powerone", val: Int { v: 1 } }, ConstProp { name: "powerten", val: Int { v: 10 } }, ConstProp { name: "powerzero", val: Int { v: 0 } }]
```
nothing introduced other than implicits from power

## main
```
block: f0.b0
in: [ConstProp { name: "getDigitsone", val: Int { v: 1 } }, ConstProp { name: "getDigitsten", val: Int { v: 10 } }, ConstProp { name: "getDigitszero", val: Int { v: 0 } }]
out: [ConstProp { name: "getDigitsone", val: Int { v: 1 } }, ConstProp { name: "getDigitsten", val: Int { v: 10 } }, ConstProp { name: "getDigitszero", val: Int { v: 0 } }, ConstProp { name: "mainsum", val: Int { v: 0 } }, ConstProp { name: "mainten", val: Int { v: 10 } }, ConstProp { name: "mainzero", val: Int { v: 0 } }]

block: f0.b1
in: [ConstProp { name: "getDigitsone", val: Int { v: 1 } }, ConstProp { name: "getDigitsten", val: Int { v: 10 } }, ConstProp { name: "getDigitszero", val: Int { v: 0 } }, ConstProp { name: "mainsum", val: Int { v: 0 } }, ConstProp { name: "mainten", val: Int { v: 10 } }, ConstProp { name: "mainzero", val: Int { v: 0 } }]
out: [ConstProp { name: "getDigitsone", val: Int { v: 1 } }, ConstProp { name: "getDigitsten", val: Int { v: 10 } }, ConstProp { name: "getDigitszero", val: Int { v: 0 } }, ConstProp { name: "mainsum", val: Int { v: 0 } }, ConstProp { name: "mainten", val: Int { v: 10 } }, ConstProp { name: "mainzero", val: Int { v: 0 } }]
```
the getdigits stuff is a limitation of how we handle function calls.

```
block: f0.b2
in: []
out: []

block: f0.b3
in: [ConstProp { name: "powerone", val: Int { v: 1 } }, ConstProp { name: "powerten", val: Int { v: 10 } }, ConstProp { name: "powerzero", val: Int { v: 0 } }]
out: [ConstProp { name: "powerone", val: Int { v: 1 } }, ConstProp { name: "powerten", val: Int { v: 10 } }, ConstProp { name: "powerzero", val: Int { v: 0 } }]

block: f0.b4
in: [ConstProp { name: "powerone", val: Int { v: 1 } }, ConstProp { name: "powerten", val: Int { v: 10 } }, ConstProp { name: "powerzero", val: Int { v: 0 } }]
out: [ConstProp { name: "powerone", val: Int { v: 1 } }, ConstProp { name: "powerten", val: Int { v: 10 } }, ConstProp { name: "powerzero", val: Int { v: 0 } }]

block: f0.b5
in: [ConstProp { name: "powerone", val: Int { v: 1 } }, ConstProp { name: "powerten", val: Int { v: 10 } }, ConstProp { name: "powerzero", val: Int { v: 0 } }]
out: [ConstProp { name: "powerone", val: Int { v: 1 } }, ConstProp { name: "powerten", val: Int { v: 10 } }, ConstProp { name: "powerzero", val: Int { v: 0 } }]

block: f0.b6
in: []
out: []
```

otherwise, no constants introduced.


## power 
```
block: f3.b0
in: [ConstProp { name: "powerone", val: Int { v: 1 } }, ConstProp { name: "powerten", val: Int { v: 10 } }, ConstProp { name: "powerzero", val: Int { v: 0 } }]
out: [ConstProp { name: "powerone", val: Int { v: 1 } }, ConstProp { name: "powerres", val: Int { v: 1 } }, ConstProp { name: "powerten", val: Int { v: 10 } }, ConstProp { name: "powerzero", val: Int { v: 0 } }]

block: f3.b1
in: [ConstProp { name: "powerone", val: Int { v: 1 } }, ConstProp { name: "powerten", val: Int { v: 10 } }, ConstProp { name: "powerzero", val: Int { v: 0 } }]
out: [ConstProp { name: "powerone", val: Int { v: 1 } }, ConstProp { name: "powerten", val: Int { v: 10 } }, ConstProp { name: "powerzero", val: Int { v: 0 } }]

block: f3.b2
in: [ConstProp { name: "powerone", val: Int { v: 1 } }, ConstProp { name: "powerten", val: Int { v: 10 } }, ConstProp { name: "powerzero", val: Int { v: 0 } }]
out: [ConstProp { name: "powerone", val: Int { v: 1 } }, ConstProp { name: "powerten", val: Int { v: 10 } }, ConstProp { name: "powerzero", val: Int { v: 0 } }]

block: f3.b3
in: [ConstProp { name: "powerone", val: Int { v: 1 } }, ConstProp { name: "powerten", val: Int { v: 10 } }, ConstProp { name: "powerzero", val: Int { v: 0 } }]
out: [ConstProp { name: "powerone", val: Int { v: 1 } }, ConstProp { name: "powerten", val: Int { v: 10 } }, ConstProp { name: "powerzero", val: Int { v: 0 } }]
```
we see that powerres is no longer a constant after block 0!

# gcd
```
block: f0.b0
in: []
out: [ConstProp { name: "mainvc0", val: Int { v: 0 } }]

block: f0.b1
in: [ConstProp { name: "mainvc0", val: Int { v: 0 } }]
out: [ConstProp { name: "mainvc0", val: Int { v: 0 } }]

block: f0.b2
in: [ConstProp { name: "mainvc0", val: Int { v: 0 } }]
out: [ConstProp { name: "mainvc0", val: Int { v: 0 } }]

block: f0.b3
in: [ConstProp { name: "mainvc0", val: Int { v: 0 } }]
out: [ConstProp { name: "mainvc0", val: Int { v: 0 } }]

block: f0.b4
in: [ConstProp { name: "mainvc0", val: Int { v: 0 } }]
out: [ConstProp { name: "mainvc0", val: Int { v: 0 } }]

block: f0.b5
in: [ConstProp { name: "mainvc0", val: Int { v: 0 } }]
out: [ConstProp { name: "mainvc0", val: Int { v: 0 } }]

block: f0.b6
in: [ConstProp { name: "mainvc0", val: Int { v: 0 } }]
out: [ConstProp { name: "mainvc0", val: Int { v: 0 } }]

block: f0.b7
in: [ConstProp { name: "mainvc0", val: Int { v: 0 } }]
out: [ConstProp { name: "mainvc0", val: Int { v: 0 } }]

block: f0.b8
in: [ConstProp { name: "mainvc0", val: Int { v: 0 } }]
out: [ConstProp { name: "mainvc0", val: Int { v: 0 } }]
````

yes! vc0 is the only constant!

# bbs
## mod
```
block: f0.b0
in: [ConstProp { name: "lsbtwo", val: Int { v: 2 } }]
out: [ConstProp { name: "lsbtwo", val: Int { v: 2 } }]

```
mod defines no constants other than those from lsb.

## lsb

```
block: f1.b0
in: [ConstProp { name: "lsbtwo", val: Int { v: 2 } }]
out: [ConstProp { name: "lsbtwo", val: Int { v: 2 } }]

block: f1.b1
in: [ConstProp { name: "lsbtwo", val: Int { v: 2 } }]
out: [ConstProp { name: "lsbtwo", val: Int { v: 2 } }]

```

the two is handled right!


## square
```
block: f2.b0
in: []
out: []
```
square defines no constants.

## main
```
block: f3.b0
in: []
out: [ConstProp { name: "mainstart", val: Int { v: 0 } }]

block: f3.b1
in: []
out: []

block: f3.b2
in: []
out: []

block: f3.b3
in: [ConstProp { name: "lsbtwo", val: Int { v: 2 } }]
out: [ConstProp { name: "lsbtwo", val: Int { v: 2 } }]

block: f3.b4
in: [ConstProp { name: "lsbtwo", val: Int { v: 2 } }]
out: [ConstProp { name: "lsbtwo", val: Int { v: 2 } }]

block: f3.b5
in: [ConstProp { name: "lsbtwo", val: Int { v: 2 } }]
out: [ConstProp { name: "lsbtwo", val: Int { v: 2 } }, ConstProp { name: "mainone", val: Int { v: 1 } }]
```
the constant one is only limited to this block

```
block: f3.b6
in: []
out: []
```
