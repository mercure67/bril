## 'resolver'

goal:
- simplify searching for symbols, labels, and functions
- it should be clear:
  - what args an operation uses (lvars, rvars)
  - where jumps, branches, calls point
  - where labels are (i.e. what code do they refer to)
- 

ex: when forming cfg, constantly have to remap labels, functions to blockno


track definition / reference (?):
- simplifies CFG formation for function calls
- requires a second iteration


defer simplification to later:
- ex: removal of unused labels / variables, dead code elimination, lvn.

however, design struct for useability in optimisations

misc implementation details:
- create a queue of 'unresolved' symbols:
  - if the symbol is not resolved yet, add it to the symbol table with an empty entry, put it in a queue
- error if the queue is non-empty
- groupings based on function?
- table design:
  - searchable based on name *and* blockno. however, name searching can be discarded later
  - table contains a mapping between name/blockno to "location", data particular to each thing
    - symbols (values): reads, writes
    - labels: definition location, associated block(s), references
    - functions: location, associated block(s)

jumps may only be to labels within the same function


labels, varnames should always be local to each function
- args need special handling
- however, this means that control flow is generally within a function, with only calls interrupting this

function data:
- callers (vector of String)
- lines (vector)
  - lines should by of type Code
- blocks (vector of ranges)
- labels (hashmap of string to lineno)
- returns (vector of lineno)

"file" data:
- list of functions
- hashmap of function name to function data

TODO:
- strip all 'global' functionality out
- change as much as possible to use numerical ranges
- 


AAAA changes are not working well
