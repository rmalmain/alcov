# alcov: a file format for binary-only coverage

<p align="center"><b>alcov version</b>: 0.1</p>

## Overview

**alcov** is a file format used to store coverage information. It is able to store both block-coverage and edge-coverage information, without assuming the source code is available.

Here is a global view of the memory layout of the alcov file format:

```
+---------------------------------------+   ----+
|                                       |       |
|   struct alcov_hdr header             |       |   Header chunk
|                                       |       |
+---------------------------------------+   ----+   <--- modules_start
|                                       |       |
|   struct alcov_module mod0            |       |
|       struct alcov_segment seg0       |       |
|       struct alcov_segment seg1       |       |
|                                       |       |
|   struct alcov_module mod1            |       |
|       struct alcov_segment seg0       |       |   Modules chunk
|                                       |       |
|   struct alcov_module mod2            |       |
|       struct alcov_segment seg0       |       |
|       struct alcov_segment seg1       |       |
|       struct alcov_segment seg2       |       |
|                                       |       |
+---------------------------------------+   ----+   <---  paths_start
|                                       |       |
| "path_mod0" "path_mod1" "path_mod2"   |       |   Paths chunk
|                                       |       |
+---------------------------------------+   ----+   <---  blocks_start
|                                       |       |
|   struct alcov_blocks b0              |       |
|                                       |       |
|   struct alcov_blocks b1              |       |   Blocks chunk
|                                       |       |
|   struct alcov_blocks b2              |       |
|                                       |       |
+---------------------------------------+   ----+   <---  edges_start (if ALCOV_FLAG_EDGES is set)
|                                       |       |
|   struct alcov_out_edge out_edge_b0   |       |
|                                       |       |
|   struct alcov_out_edge out_edge_b1   |       |   Out edges chunk (if ALCOV_FLAG_EDGES is set)
|                                       |       |
|   struct alcov_out_edge out_edge_b2   |       |
|                                       |       |
+---------------------------------------+   ----+   <---  EOF
```

## Chunks

The alcov file format is split in multiple **chunks** (in order):
- The **header chunk** (one `alcov_hdr`).
- The **modules chunk** (array of `alcov_module`).
- The **paths chunk** (array of C (ASCII) strings).
- The **blocks chunk** (array of `alcov_block`)
- The **edges chunk** (*optional*) (array of `alcov_array`)

### Header

The header contains global information about what is stored in the trace file.
The fields available are:
- `magic`: A 64bits magic value that should always be the same (`0xdda28f766f636c61`).
- `version_major`: The alcov major version being used.
- `version_minor`: The alcov minor version being used.
- `nb_modules`: The number of modules in the module section.
- `nb_blocks`: The number of blocks in the module section.
- `nb_edges`: The total number of edges being traversed.
- `paths_start`: the offset in bytes in the file to the start of the path chunk.
- `blocks_start`: the offset in bytes in the file to the start of the block chunk.
- `edges_start`: the offset in bytes in the file to the start of the edges chunk.
- `flags`: the flags enabled or disabled for the current trace.

### Flags

alcov has three main flags that can be either set or unset independently, by checking if the bit at the given position is 0 (unsed) or 1 (set). They are given by the `alcov_hdr.flags` field:
- `ALCOV_FLAG_EDGES` (position 0): if set, it indicates edges have been tracked. If unset, the edge chunk is absent from the file and the following fields are ignored: `alcov_block.nb_out_edges`, `alcov_block.out_edges_offset`.
- `ALCOV_FLAG_COMPRESS` (position 1): if set, it indicates the block (and edge if enabled) chunks have been compressed with LZMA2.
- `ALCOV_FLAG_INPUT_PATH` (position 2): if set, it indicates the trace was run while executing the program with a particular input file. The file path can be found in the paths chunk as the very first string in the chunk.

### Modules

Modules are a binary unit (often, but not necessarily) backed by a file.
It typically represents an ELF, a PE, a raw binary, but can be any file.
It is the implementation's responsibility to detect and handle or not some files or not.
If the module has a path, it must be unique. In other words, there cannot be two or more modules with the same path.
These modules are split in segments, useful to describe files mapped at different locations.
It is implementation-specific whether the backing file will be parsed, or used to get a certain interpretation of the module.

### Paths

Paths in alcov follow POSIX's pathname specification.
The paths chunk contains a list of C strings (NULL-terminated sequence of bytes).
Each path should be unique, according to the previous section.
Paths should be encoded with a NULL-terminated ASCII encoding.

### Blocks

A block is analogous to a basic block in compilation terms.
It represents an indivisible sequence of instructions (only the last instruction can alter the control flow, there is no incoming edge in the block except for the first instruction).

### Edges

An edge is a directed transition between two blocks. It represents the execution going from one block to another.
In alcov, it is represented similarly as an adjacency list: for each block, only its outgoing edges are registered (represented by the ID of the block to which the control flow has been at some point).
This is done to save as much space as possible.

## Versioning

alcov, in v0, differs a bit in the way breaking changes are handled: during v0.1, there is no restriction on what can break, and a which frequency.
