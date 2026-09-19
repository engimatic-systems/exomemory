------------------------ MODULE AttemptSuppression ------------------------
EXTENDS Naturals, FiniteSets
CONSTANTS Keys, Contexts, MaxAttempts
VARIABLES attempted, pending, sends, compacting
vars == <<attempted, pending, sends, compacting>>

Init == /\ attempted = [c \in Contexts |-> {}]
        /\ pending = [c \in Contexts |-> {}]
        /\ sends = [c \in Contexts |-> [k \in Keys |-> 0]]
        /\ compacting = {}

\* Marking is atomic; a later send is optional and never retried.
Mark(c,k) == /\ c \notin compacting
             /\ k \notin attempted[c]
             /\ Cardinality(attempted[c]) < MaxAttempts
             /\ attempted' = [attempted EXCEPT ![c] = @ \cup {k}]
             /\ pending' = [pending EXCEPT ![c] = @ \cup {k}]
             /\ UNCHANGED <<sends, compacting>>
Send(c,k) == /\ c \notin compacting
             /\ k \in pending[c]
             /\ pending' = [pending EXCEPT ![c] = @ \ {k}]
             /\ sends' = [sends EXCEPT ![c][k] = @ + 1]
             /\ UNCHANGED <<attempted, compacting>>
Drop(c) == /\ pending' = [pending EXCEPT ![c] = {}]
           /\ UNCHANGED <<attempted, sends, compacting>>
Start(c) == /\ compacting' = compacting \cup {c}
            /\ UNCHANGED <<attempted, pending, sends>>
Abort(c) == /\ c \in compacting
            /\ compacting' = compacting \ {c}
            /\ UNCHANGED <<attempted, pending, sends>>
\* The adapter serializes completion with sends and discards abandoned old batches.
Complete(c) == /\ c \in compacting
               /\ compacting' = compacting \ {c}
               /\ attempted' = [attempted EXCEPT ![c] = {}]
               /\ pending' = [pending EXCEPT ![c] = {}]
               /\ sends' = [sends EXCEPT ![c] = [k \in Keys |-> 0]]
Next == \E c \in Contexts :
          (\E k \in Keys : Mark(c,k) \/ Send(c,k))
          \/ Drop(c) \/ Start(c) \/ Abort(c) \/ Complete(c)
Spec == Init /\ [][Next]_vars
TypeOK == /\ attempted \in [Contexts -> SUBSET Keys]
          /\ pending \in [Contexts -> SUBSET Keys]
          /\ sends \in [Contexts -> [Keys -> Nat]]
          /\ compacting \subseteq Contexts
WithinAttemptCap == \A c \in Contexts : Cardinality(attempted[c]) <= MaxAttempts
AtMostOneSendPerInterval == \A c \in Contexts : \A k \in Keys : sends[c][k] <= 1
SendRequiresMark == \A c \in Contexts :
                     /\ pending[c] \subseteq attempted[c]
                     /\ {k \in Keys : sends[c][k] > 0} \subseteq attempted[c]
=============================================================================
