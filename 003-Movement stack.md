Okay how do we do movement
Because there's SO MANY THINGS that can interere or change how a move is made, what's allowed, etc
A square can be frozen. A square can be slippery. A square can be blocked, can be a turret, etc, etc, etc
Other pieces can change movement. 
There are so many factors to consider
SO
I'M PROPOSING
A movement stack
Inspired by Binding of Isaac: Four Souls, we have a system where each potential modifier is stacked on top of each other
Each step down the stack modifies what happened before
Implementing the logic behind this may be challenging, but if I do it, I'll have one hell of a powerful system
So the pipeline would take in the proposed set of allowed moves and change it accordingly for each new "event"

How do we figure out what modifies the movement set?
How do different things modify it?
What's the order of operations?
There are so many things that can affect this
Frozen square behavior
Slippery square behavior
Turret that blocks or shoots
Aura of a piece that limits range
Board-wide weather pattern that affects diagonal moves
Whatever your mind can think of, the system needs to make it possible

What types do we need?
`MovementEvent`, `MovementStack`, `MovementModifier`, `MovementEffect`, etc