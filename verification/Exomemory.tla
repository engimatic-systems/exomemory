------------------------------ MODULE Exomemory ------------------------------
EXTENDS Naturals, FiniteSets

(***************************************************************************
This is a deliberately small, executable model of the stateful parts of the
Exomemory design spec.  It does not model LLM quality.  It models the places
where concurrency, lag, and authority can violate crisp safety properties.

The Boolean constants are design choices.  The Current_*.cfg files assign
choices that the prose spec currently permits.  Hardened.cfg assigns one
coherent set of stricter choices.
***************************************************************************)

CONSTANTS
    Memories, Sessions, Writers, NoMemory,
    MaxVersion, CooldownLen, ContextBudget,
    AtomicCreate, AtomicOverwrite, CompareAndSwap,
    ExactContextDedup, EnforceBudget, SafeAuthority,
    TrustedMemories

ASSUME /\ Memories # {}
       /\ Sessions # {}
       /\ Writers # {}
       /\ NoMemory \notin Memories
       /\ MaxVersion \in Nat \ {0}
       /\ CooldownLen \in Nat
       /\ ContextBudget \in Nat \ {0}
       /\ TrustedMemories \subseteq Memories

Authorities == {"None", "Data", "Developer"}

VARIABLES
    bodyVersion,       \* durable full-data version; 0 means absent
    ghostVersion,      \* metadata/preview version; 0 means absent
    triggerPublished,  \* a trigger visible to hook processing
    pendingMemory,     \* writer's staged target
    pendingBase,       \* version the writer read before doing work
    contextCopies,     \* copies of each ghost in each live context window
    historyView,       \* keys inferred from the asynchronously observed log
    cooldown,
    contextAuthority,  \* authority used for the currently live copy
    staleSuppressionObserved,
    staleCommitObserved

vars == <<bodyVersion, ghostVersion, triggerPublished,
          pendingMemory, pendingBase,
          contextCopies, historyView, cooldown, contextAuthority,
          staleSuppressionObserved, staleCommitObserved>>

TotalCopies(s) ==
    Cardinality(UNION {
        {<<m, n>> : n \in 1..contextCopies[s][m]} : m \in Memories})

CurrentlySeen(s, m) ==
    IF ExactContextDedup
    THEN contextCopies[s][m] > 0
    ELSE m \in historyView[s]

Init ==
    /\ bodyVersion = [m \in Memories |-> 0]
    /\ ghostVersion = [m \in Memories |-> 0]
    /\ triggerPublished = [m \in Memories |-> FALSE]
    /\ pendingMemory = [w \in Writers |-> NoMemory]
    /\ pendingBase = [w \in Writers |-> 0]
    /\ contextCopies =
        [s \in Sessions |-> [m \in Memories |-> 0]]
    /\ historyView = [s \in Sessions |-> {}]
    /\ cooldown =
        [s \in Sessions |-> [m \in Memories |-> 0]]
    /\ contextAuthority =
        [s \in Sessions |-> [m \in Memories |-> "None"]]
    /\ staleSuppressionObserved = FALSE
    /\ staleCommitObserved = FALSE

AtomicCreateRecord(m) ==
    /\ AtomicCreate
    /\ bodyVersion[m] = 0
    /\ ~triggerPublished[m]
    /\ bodyVersion' = [bodyVersion EXCEPT ![m] = 1]
    /\ ghostVersion' = [ghostVersion EXCEPT ![m] = 1]
    /\ triggerPublished' = [triggerPublished EXCEPT ![m] = TRUE]
    /\ UNCHANGED <<pendingMemory, pendingBase,
                    contextCopies, historyView, cooldown, contextAuthority,
                    staleSuppressionObserved, staleCommitObserved>>

(***************************************************************************
When creation is not required to be atomic, a backend write and trigger
publication may occur in either order.  Publishing first is the dangerous
interleaving: hooks can advertise a key whose body does not exist.
***************************************************************************)
WriteInitialBody(m) ==
    /\ ~AtomicCreate
    /\ bodyVersion[m] = 0
    /\ bodyVersion' = [bodyVersion EXCEPT ![m] = 1]
    /\ ghostVersion' = [ghostVersion EXCEPT ![m] = 1]
    /\ UNCHANGED <<triggerPublished, pendingMemory, pendingBase,
                    contextCopies, historyView, cooldown, contextAuthority,
                    staleSuppressionObserved, staleCommitObserved>>

PublishTrigger(m) ==
    /\ ~AtomicCreate
    /\ ~triggerPublished[m]
    /\ triggerPublished' = [triggerPublished EXCEPT ![m] = TRUE]
    /\ UNCHANGED <<bodyVersion, ghostVersion, pendingMemory, pendingBase,
                    contextCopies, historyView, cooldown, contextAuthority,
                    staleSuppressionObserved, staleCommitObserved>>

BeginOverwrite(w, m) ==
    /\ bodyVersion[m] > 0
    /\ pendingMemory[w] = NoMemory
    /\ pendingMemory' = [pendingMemory EXCEPT ![w] = m]
    /\ pendingBase' = [pendingBase EXCEPT ![w] = bodyVersion[m]]
    /\ UNCHANGED <<bodyVersion, ghostVersion, triggerPublished,
                    contextCopies, historyView, cooldown, contextAuthority,
                    staleSuppressionObserved, staleCommitObserved>>

CommitOverwrite(w) ==
    LET m == pendingMemory[w] IN
    /\ m \in Memories
    /\ bodyVersion[m] < MaxVersion
    /\ (~CompareAndSwap \/ pendingBase[w] = bodyVersion[m])
    /\ bodyVersion' = [bodyVersion EXCEPT ![m] = @ + 1]
    /\ ghostVersion' =
        IF AtomicOverwrite
        THEN [ghostVersion EXCEPT ![m] = bodyVersion'[m]]
        ELSE ghostVersion
    /\ pendingMemory' = [pendingMemory EXCEPT ![w] = NoMemory]
    /\ staleCommitObserved' =
        (staleCommitObserved \/ (pendingBase[w] # bodyVersion[m]))
    /\ UNCHANGED <<triggerPublished, pendingBase,
                    contextCopies, historyView, cooldown, contextAuthority,
                    staleSuppressionObserved>>

AbortStaleOverwrite(w) ==
    LET m == pendingMemory[w] IN
    /\ CompareAndSwap
    /\ m \in Memories
    /\ pendingBase[w] # bodyVersion[m]
    /\ pendingMemory' = [pendingMemory EXCEPT ![w] = NoMemory]
    /\ UNCHANGED <<bodyVersion, ghostVersion, triggerPublished, pendingBase,
                    contextCopies, historyView, cooldown, contextAuthority,
                    staleSuppressionObserved, staleCommitObserved>>

(***************************************************************************
A non-atomic overwrite eventually refreshes the ghost.  Until it does, the
old preview and new body form a visible mixed version.
***************************************************************************)
RefreshGhost(m) ==
    /\ ~AtomicOverwrite
    /\ ghostVersion[m] < bodyVersion[m]
    /\ ghostVersion' = [ghostVersion EXCEPT ![m] = bodyVersion[m]]
    /\ UNCHANGED <<bodyVersion, triggerPublished, pendingMemory, pendingBase,
                    contextCopies, historyView, cooldown, contextAuthority,
                    staleSuppressionObserved, staleCommitObserved>>

InjectGhost(s, m) ==
    /\ triggerPublished[m]
    /\ bodyVersion[m] > 0
    /\ cooldown[s][m] = 0
    /\ ~CurrentlySeen(s, m)
    /\ IF EnforceBudget
       THEN TotalCopies(s) < ContextBudget
       ELSE TRUE
    /\ contextCopies' =
        [contextCopies EXCEPT ![s][m] = @ + 1]
    /\ cooldown' =
        [cooldown EXCEPT ![s][m] = CooldownLen]
    /\ contextAuthority' =
        [contextAuthority EXCEPT ![s][m] =
            IF SafeAuthority /\ m \notin TrustedMemories
            THEN "Data"
            ELSE "Developer"]
    /\ UNCHANGED <<bodyVersion, ghostVersion, triggerPublished,
                    pendingMemory, pendingBase, historyView,
                    staleSuppressionObserved, staleCommitObserved>>

(***************************************************************************
The prose design uses a lagging session-history file.  SyncHistory represents
one watcher update.  It is intentionally not part of InjectGhost, so TLC can
schedule more hook events before the watcher catches up.
***************************************************************************)
SyncHistory(s) ==
    /\ historyView' = [historyView EXCEPT ![s] =
        @ \cup {m \in Memories : contextCopies[s][m] > 0}]
    /\ UNCHANGED <<bodyVersion, ghostVersion, triggerPublished,
                    pendingMemory, pendingBase, contextCopies, cooldown,
                    contextAuthority, staleSuppressionObserved,
                    staleCommitObserved>>

AdvanceHook(s) ==
    /\ \E m \in Memories : cooldown[s][m] > 0
    /\ cooldown' = [cooldown EXCEPT ![s] =
        [m \in Memories |-> IF @[m] > 0 THEN @[m] - 1 ELSE 0]]
    /\ UNCHANGED <<bodyVersion, ghostVersion, triggerPublished,
                    pendingMemory, pendingBase, contextCopies, historyView,
                    contextAuthority, staleSuppressionObserved,
                    staleCommitObserved>>

CompactAway(s, m) ==
    /\ contextCopies[s][m] > 0
    /\ contextCopies' = [contextCopies EXCEPT ![s][m] = 0]
    /\ contextAuthority' =
        [contextAuthority EXCEPT ![s][m] = "None"]
    /\ UNCHANGED <<bodyVersion, ghostVersion, triggerPublished,
                    pendingMemory, pendingBase, historyView, cooldown,
                    staleSuppressionObserved, staleCommitObserved>>

(***************************************************************************
If an append-only history says a key is present after compaction removed it
from the actual window, the system suppresses a useful resurfacing.  This
action records that externally observable false-negative decision.
***************************************************************************)
ObserveStaleSuppression(s, m) ==
    /\ triggerPublished[m]
    /\ bodyVersion[m] > 0
    /\ contextCopies[s][m] = 0
    /\ cooldown[s][m] = 0
    /\ CurrentlySeen(s, m)
    /\ staleSuppressionObserved' = TRUE
    /\ UNCHANGED <<bodyVersion, ghostVersion, triggerPublished,
                    pendingMemory, pendingBase, contextCopies, historyView,
                    cooldown, contextAuthority, staleCommitObserved>>

Next ==
    \/ \E m \in Memories : AtomicCreateRecord(m)
    \/ \E m \in Memories : WriteInitialBody(m)
    \/ \E m \in Memories : PublishTrigger(m)
    \/ \E w \in Writers, m \in Memories : BeginOverwrite(w, m)
    \/ \E w \in Writers : CommitOverwrite(w)
    \/ \E w \in Writers : AbortStaleOverwrite(w)
    \/ \E m \in Memories : RefreshGhost(m)
    \/ \E s \in Sessions, m \in Memories : InjectGhost(s, m)
    \/ \E s \in Sessions : SyncHistory(s)
    \/ \E s \in Sessions : AdvanceHook(s)
    \/ \E s \in Sessions, m \in Memories : CompactAway(s, m)
    \/ \E s \in Sessions, m \in Memories : ObserveStaleSuppression(s, m)

Spec == Init /\ [][Next]_vars

TypeOK ==
    /\ bodyVersion \in [Memories -> 0..MaxVersion]
    /\ ghostVersion \in [Memories -> 0..MaxVersion]
    /\ triggerPublished \in [Memories -> BOOLEAN]
    /\ pendingMemory \in [Writers -> Memories \cup {NoMemory}]
    /\ pendingBase \in [Writers -> 0..MaxVersion]
    /\ contextCopies \in
        [Sessions -> [Memories -> 0..(ContextBudget + 1)]]
    /\ historyView \in [Sessions -> SUBSET Memories]
    /\ cooldown \in [Sessions -> [Memories -> 0..CooldownLen]]
    /\ contextAuthority \in
        [Sessions -> [Memories -> Authorities]]
    /\ staleSuppressionObserved \in BOOLEAN
    /\ staleCommitObserved \in BOOLEAN

(***************************************************************************
Safety properties.  Each name states a contract that the prose spec needs in
order to rule out the corresponding TLC counterexample.
***************************************************************************)
NoDanglingGhost ==
    \A m \in Memories : triggerPublished[m] => bodyVersion[m] > 0

GhostBodySameVersion ==
    \A m \in Memories :
        triggerPublished[m] /\ bodyVersion[m] > 0
        => ghostVersion[m] = bodyVersion[m]

AtMostOneGhostPerWindow ==
    \A s \in Sessions, m \in Memories : contextCopies[s][m] <= 1

WithinGhostBudget ==
    \A s \in Sessions : TotalCopies(s) <= ContextBudget

NoUntrustedDeveloperContext ==
    \A s \in Sessions, m \in Memories :
        contextCopies[s][m] > 0 /\ m \notin TrustedMemories
        => contextAuthority[s][m] = "Data"

NoStaleHistorySuppression == ~staleSuppressionObserved

NoStaleWriterCommit == ~staleCommitObserved

=============================================================================
