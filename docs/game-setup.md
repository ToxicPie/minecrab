# Minecrab Setup

How to actually run the game.

## Usage

`minecrab --config-path <path_to_configuration_file>`

## Configuration File

The configuration file contains the following JSON:

```python
{
    "user_configs": [
        {
            "initd_bytecode": "path to init bytecode of user 1",
            "initd_memory": "path to init memory of user 1",
            "uid": int,
            "spawn_point": [x, y]
        },
        ...
    ],
    "default_nice": int,
    "initd_lifetime": int,
    "max_processes": int,
    "mapdata_path": "path to map data",
    "crypto_spawn": {
        "name": [[difficulty (int), probability (float)], [difficulty, probability], ...],
        ...
    }
}
```

### Map File

The map file must consist of exactly 256*256 bytes. Each byte represents a tile on the map:
- 0: Land
- 1: Wall

The tiles are in x-major order, i.e. the order of coordinates is `(0, 0), (0, 1), (0, 2), ...`.

### Player Files

Each user must have two files `initd_bytecode` and `initd_memory` describing their init process. Each of the files must contain exactly 65536 bytes.

### Some Other Fields

- `uid`: The user id of each user. Must be between 0 and 65535.
- `spawn_point`: The initd location of each user. It's recommended to make them unique and not inside walls.
- `default_nice`: The default nice value of processes.
- `max_processes`: The maximum number of processes a user can have at a time.
- `crypto_spawn`: Describes the spawn rates of each crypto challenge type. It should be a map of `str: [[int, float], ...]`. For example, `"dog": [[1, 0.5], [2, 0.3], [3, 0.1]]` means in each tick: a dog challenge of difficulty 1 spawns with probability 0.5, a dog challenge of difficulty 2 spawns with probability 0.3, and a dog challenge of difficulty 3 spawns with probability 0.1.

## Replay

The executable prints game events to stdout. They should be pretty self-explanatory.
