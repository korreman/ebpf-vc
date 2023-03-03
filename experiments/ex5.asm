; I'd like another branching non-cycle program.
; (* Two Way Sort
;
;    The following program sorts an array of Boolean values, with False<True.
;
;    Questions:
;
;    1. Prove safety i.e. the absence of array access out of bounds.
;
;    2. Prove termination.
;
;    3. Prove that array a is sorted after execution of function two_way_sort
;       (using the predicate sorted that is provided).
;
;    4. Show that after execution the array contents is a permutation of its
;       initial contents. Use the library predicate "permut_all" to do so
;       (the corresponding module ArrayPermut is already imported).
;
;       You can refer to the contents of array a at the beginning of the
;       function with notation "a at Init".
; *)
;
; module TwoWaySort
;
;   use int.Int
;   use bool.Bool
;   use ref.Refint
;   use array.Array
;   use array.ArraySwap
;   use array.ArrayPermut
;
;   predicate (<<) (x y: bool) = x = False \/ y = True
;
;   predicate sorted (a: array bool) =
;     forall i1 i2: int. 0 <= i1 <= i2 < a.length -> a[i1] << a[i2]
;
;   let two_way_sort (a: array bool) : unit
;     ensures { true }
;   =
;     label Init in
;     let ref i = 0 in
;     let ref j = length a - 1 in
;     while i < j do
;       invariant { j < length a}
;       invariant { 0 <= i }
;       variant { j - i }
;       if not a[i] then
;         incr i
;       else if a[j] then
;         decr j
;       else begin
;         swap a i j;
;         incr i;
;         decr j
;       end
;     done
;
; end
exit
