# Minecrab Gaming - TL;DR

Welcome to the Minecrab, a thrilling competition where participants will showcase their skills in coding Real-Time Strategy Game AI in the Minecrab world. Are you ready to take on the ultimate coding battle and conquer the cool game world?

## Tick

Each game is divided into several ticks. In each game tick, the following things happen in order:
- The map spawns crypto challenges.
- The kernel runs all alive processes in the order of descending nice values.

## Process

Processes run bytecode to interact with the game, from mining resources to attacking enemies.

Each process has a random PID between 1 and 65534.

Each process has a finite lifetime, which is the number of ticks it can live for. For every tick a process lives, its user gains score. When a process's lifetime reaches 0, it dies and all descendant processes die.

### Init

For each user, there's a special process called init, which is created at the start of the game. It is the only process directly controllable by players. When it dies the player leaves the game, so protect it at all costs.

### Other Process

Any process that is not init. They are created from forking.

## VM

### Instruction

Bytecode.

### Emulator

Runs bytecode.

## Map

A 256*256 map. Each tile can be:
- Land, which may contain a process, a crypto challenge, or both.
- Wall, which only contains a wall and nothing else.

The map takes the topology of a torus. In normal human speak, it means `x` and `y` coordinates are modulo 256, so moving by `(1, 0)` from `(255, 123)` sends you to `(0, 123)`, etc.

## Crypto

TO THE MOON
