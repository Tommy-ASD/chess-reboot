Okay, how should we handle Skibidi brain rot?
A radius around the skibidi must become brainrotted when the skibidi changes phase
It must increase in size when the skibidi increases further
The areas of brainrot of a given Skibidi must be removed once the Skibidi moves or is taken
Brainrotted squares will have the "Brainrot" condition applied to them.
Pieces on these squares will not be able to move.

How and where do we apply brainrot?
In the global make_move function?
In Skibidi's make move function?
In the global post-move function?
In the Skibidi's post-move function?
Somewhere else?

Maybe we make a `board.recalculate_all_brainrot_zones()`
Which iterates through all squares and removes brainrot, before re-applying based on Skibidis
