# Plan for experiments

Another thing to consider is that it would be good to have similar negative examples
that fail to verify.

## Straightline programs

These exercise the weakest precondition calculus
without any branches.

- A program demonstrating integer arithmetic through division and modulo.
- A program demonstrating memory access.
- A program demonstrating helper functions.

Nice-to-have:
- Checking for integer initialization.

## Jumping programs

Same as straightline,
but show that a branchless program can jump all over the place.

## Branching programs

Show that programs for which the CFG is a DAG don't require annotations
(aside from module-level requirements).

Limitation: Show that too many branches results in exponential(?) growth of the VC.

## Cyclic programs

