Rustetris backlog

++ Main tasks
[Done] 1. Destroy a full line and move all other dead pieces downwards
[Done] 2. Create other types of pieces randomly
[Done] 3. Recognize when game is over, meaning when a spawned piece cannot move even once (or is already overlapping)
[Done] 4. Drop with space or downkey
[Done] 5. Show where piece is gonna fall to
[Done] 6. Show what piece comes next
[    ] 7. Show score in game window 



++ Known bugs
[Possibly fixed by refreshing old_tiles and loading piece to grid also if drop key has moved brick]
[1]. Sometimes when pressing downkey, block stopped around middle of screen even though nothing was blocking it
   * Doesn't seem to stop in middle of screen anymore after addition of continous press to drop.
     However, stops one tile before it should. Possibly related to case where the piece moves twice within one
     loop run: First when pressing down and then with tick-move. Maybe caused by old tiles getting refreshed only
     once during the loop, and there is a case of brick overlapping with itself
     ...but this doesn't explain why it always happens just one tile before it would overlap with something
  * Found new occurrence of this after fixing. The block stopped 1 tile before, possibly when rotating it
    * Happened also when moving it to right, not rotating
