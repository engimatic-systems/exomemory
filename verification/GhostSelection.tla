--------------------------- MODULE GhostSelection ---------------------------
EXTENDS Naturals, Sequences, FiniteSets

(***************************************************************************
This module models the finalized candidate-selection rule separately from the
storage/lifecycle model in Exomemory.tla.

Key design choices represented here:
  * a hard maximum number of ghosts, not a runtime token budget;
  * a hard maximum size for the final rendered form of each ghost;
  * deterministic ordering with a stable key tie-break;
  * `annoy` adds a deterministic rank penalty instead of random sampling.

BaseRank abstracts the earlier lexicographic fields (relevance band, scope,
and backend-local rank) into one deterministic integer for this small model.
Lower values rank first. LastShownEvent and StableOrder finish the ordering.
***************************************************************************)

CONSTANTS
    Candidates, PreferredCandidate, SecondCandidate,
    Sources, PrimarySource,
    MaxGhosts, MaxGhostBytes, WrapperBytes, MaxInjectionBytes,
    MaxPerSource, MaxPenalty, PenaltyStep, MaxEvent,
    DeterministicSelection

ASSUME /\ Cardinality(Candidates) = 3
       /\ PreferredCandidate \in Candidates
       /\ SecondCandidate \in Candidates \ {PreferredCandidate}
       /\ Cardinality(Sources) = 2
       /\ PrimarySource \in Sources
       /\ MaxGhosts \in Nat \ {0}
       /\ MaxGhostBytes \in Nat \ {0}
       /\ WrapperBytes \in Nat
       /\ MaxInjectionBytes \in Nat
       /\ WrapperBytes + (MaxGhosts * MaxGhostBytes) <= MaxInjectionBytes
       /\ MaxPerSource \in Nat \ {0}
       /\ MaxPenalty \in Nat \ {0}
       /\ PenaltyStep \in Nat \ {0}
       /\ MaxEvent \in Nat \ {0}

ThirdCandidate ==
    CHOOSE c \in Candidates :
        c # PreferredCandidate /\ c # SecondCandidate

OtherSource == CHOOSE source \in Sources : source # PrimarySource

(***************************************************************************
The two candidates intentionally begin in different relevance bands. Stable
order is still explicit so equal effective ranks always have one winner.
***************************************************************************)
BaseRank(c) ==
    CASE c = PreferredCandidate -> 0
      [] c = SecondCandidate -> 1
      [] OTHER -> 2

StableOrder(c) == BaseRank(c)

(***************************************************************************
A source is the pair (backend instance, trigger id) after duplicate ghost keys
have been collapsed. Two high-ranked candidates intentionally share a source;
the third comes from a different source so TLC can exercise the diversity cap.
***************************************************************************)
SourceOf(c) ==
    IF c = PreferredCandidate \/ c = SecondCandidate
    THEN PrimarySource
    ELSE OtherSource

VARIABLES
    selected,            \* ordered ghosts selected for this hook response
    annoyPenalty,        \* durable, deterministic negative feedback
    lastShownEvent,      \* event sequence, not wall-clock time
    eventNumber,
    nonTopChoiceObserved

vars == <<selected, annoyPenalty, lastShownEvent,
          eventNumber, nonTopChoiceObserved>>

SelectedSet == {selected[i] : i \in 1..Len(selected)}

SourceCount(source) ==
    Cardinality({c \in SelectedSet : SourceOf(c) = source})

EffectiveRank(c) == BaseRank(c) + (PenaltyStep * annoyPenalty[c])

RanksBefore(a, b) ==
    \/ EffectiveRank(a) < EffectiveRank(b)
    \/ /\ EffectiveRank(a) = EffectiveRank(b)
       /\ lastShownEvent[a] < lastShownEvent[b]
    \/ /\ EffectiveRank(a) = EffectiveRank(b)
       /\ lastShownEvent[a] = lastShownEvent[b]
       /\ StableOrder(a) < StableOrder(b)

Eligible(c) ==
    /\ c \in Candidates \ SelectedSet
    /\ Len(selected) < MaxGhosts
    /\ eventNumber < MaxEvent
    /\ SourceCount(SourceOf(c)) < MaxPerSource

TopEligible(c) ==
    /\ Eligible(c)
    /\ ~\E other \in Candidates :
        Eligible(other) /\ RanksBefore(other, c)

Init ==
    /\ selected = <<>>
    /\ annoyPenalty = [c \in Candidates |-> 0]
    /\ lastShownEvent = [c \in Candidates |-> 0]
    /\ eventNumber = 0
    /\ nonTopChoiceObserved = FALSE

Annoy(c) ==
    /\ c \in Candidates
    /\ annoyPenalty[c] < MaxPenalty
    /\ annoyPenalty' = [annoyPenalty EXCEPT ![c] = @ + 1]
    /\ UNCHANGED <<selected, lastShownEvent,
                    eventNumber, nonTopChoiceObserved>>

SelectGhost(c) ==
    /\ Eligible(c)
    /\ IF DeterministicSelection THEN TopEligible(c) ELSE TRUE
    /\ selected' = Append(selected, c)
    /\ eventNumber' = eventNumber + 1
    /\ lastShownEvent' =
        [lastShownEvent EXCEPT ![c] = eventNumber']
    /\ nonTopChoiceObserved' =
        (nonTopChoiceObserved \/ ~TopEligible(c))
    /\ UNCHANGED annoyPenalty

SelectionComplete ==
    /\ (Len(selected) = MaxGhosts
        \/ ~\E c \in Candidates : Eligible(c))
    /\ UNCHANGED vars

Next ==
    \/ \E c \in Candidates : Annoy(c)
    \/ \E c \in Candidates : SelectGhost(c)
    \/ SelectionComplete

Spec == Init /\ [][Next]_vars

TypeOK ==
    /\ selected \in Seq(Candidates)
    /\ Len(selected) <= MaxGhosts
    /\ annoyPenalty \in [Candidates -> 0..MaxPenalty]
    /\ lastShownEvent \in [Candidates -> 0..MaxEvent]
    /\ eventNumber \in 0..MaxEvent
    /\ nonTopChoiceObserved \in BOOLEAN

WithinCountCap == Len(selected) <= MaxGhosts

(***************************************************************************
Because the limit applies to the final rendered ghost, not merely its blurb,
the count cap implies a conservative byte bound without runtime tokenization.
***************************************************************************)
WithinRenderedByteBound ==
    WrapperBytes + (Len(selected) * MaxGhostBytes) <= MaxInjectionBytes

AlwaysSelectedTopRanked == ~nonTopChoiceObserved

NoDuplicateSelection == Len(selected) = Cardinality(SelectedSet)

WithinPerSourceLimit ==
    \A source \in Sources : SourceCount(source) <= MaxPerSource

AtMostOneTopCandidate ==
    Cardinality({c \in Candidates : TopEligible(c)}) <= 1

(***************************************************************************
With this model's rank spacing, one `annoy` action moves the formerly preferred
candidate behind the other candidate. No random draw is involved.
***************************************************************************)
PenaltyDemotesPreferred ==
    /\ selected = <<>>
    /\ annoyPenalty[PreferredCandidate] > 0
    /\ annoyPenalty[SecondCandidate] = 0
    /\ annoyPenalty[ThirdCandidate] = 0
    => TopEligible(SecondCandidate)

(***************************************************************************
Even though SecondCandidate outranks ThirdCandidate globally, it becomes
ineligible after PreferredCandidate consumes their shared source's quota.
***************************************************************************)
DiversityChangesSecondChoice ==
    selected = <<PreferredCandidate>> => TopEligible(ThirdCandidate)

=============================================================================
