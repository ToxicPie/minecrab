# Minecrab

Minecrab is a [game](./docs/tldr.md) in which players write bytecode to fight each other on a 2D map.

Each player submits two files, bytecode and memory, which are used to created the init process of the user. After all init processes are created, they are on their own and the players have no control over them. Processes can create new processes, mine resources, attack other players' processes, etc. to win the game. Try to make the best AI fighter!

Processes run on a specially crafted [instruction set](./docs/instructions.md) and can interact with various game components with [system calls](./docs/syscall.md).

A game consists of several ticks. In each tick, every process is run for a certain number of cycles. Every process has a finite number of ticks that it can live for, and a player leaves the game as soon as their init process dies. The game continues until every player has left.

When the game ends, players are ranked according to their scores. A player gains score when one of their processes  survives for a tick, or when the game ends and they have leftover cryptocurrency.

For more details, read the files in the [docs](./docs/) folder. Note that documentation may contain inaccuracies. When in doubt, check the code.

## How To Play

Write a good AI program and upload the bytecode file to the system. You also need to upload an initial memory file for the program. Both files must be exactly 65536 bytes, otherwise it is undefined behavior.

Then just sit back and watch all the processes fight!
