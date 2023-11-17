# Minecrab Cryptocurrency

Crypto challenges spawn on the map occasionally. Mine them to get cryptocurrency! Cryptocurrencies allow you to call certain system calls, and are converted to bonus score at the end of the game.

## Mining

"Mining" refers to the action of solving a crypto challenge. When a process moves into a square containing a crypto challenge, it can use the `FetchChallenge` system call to read the challenge, solve the challenge, and then call `SolveChallenge` to submit a solution.

After a successful solve, the challenge on the square disappears, and the player is awarded with the corresponding mining reward.

When a process does `FetchChallenge`, it receives some metadata of the challenge along with the actual challenge data.

When a process does `SolveChallenge`, it submits a tuple `nonce` of 4 16-bit integers. Then the challenge verifies whether

## Cryptocurrency Types

### DogeCoin

The currency used to do game-related tasks like moving, mining, attacking, etc.

### StarSleepShortage

You should avoid getting too much of this currency. Each StarSleepShortage decreases the number of cycles a process can run in a tick by 1, therefore decreasing your score. You get this currency from calling certain system calls and solving challenges.

### Ethereum

The currency used to do process-related tasks like forking, killing, renicing, etc.

## Cryptocurrency Challenges

### DogChallenge

> Name: `dog`
> Numeric ID: `0x420`

`FetchChallenge`'s `data` field has 8 bytes: 4 16-bit integers in little-endian. Simply copy and paste them into the nonce.

It rewards DogeCoin and StarSleepShortage.

### BedChallenge

> Name: `bed`
> Numeric ID: `0xbed`

`FetchChallenge`'s `data` field has 0 bytes. Submit `(0, 1, 2, 3)` as the nonce.

It rewards a negative amount of StarSleepShortage.

### EtherChallenge

> Name: `ether`
> Numeric ID: `0x1337`

`FetchChallenge`'s `data` field has 2 bytes: a 16-bit integer `x`, which is always odd. Submit `(y, 0, 0, 0)` such that `x * y mod 2^16 == 1`.

It rewards Ethereum and StarSleepShortage.
