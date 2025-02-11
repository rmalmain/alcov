// alcov programmatic specification.
// this file is a concrete representation of alcov, and should be used as reference for other implementations.
// 
// # Compiler compatibility
//
// For now, alcov supports any C compiler that supports the `scalar_storage_order` pragma.
// For GCC, GCC version should be >= 6.
//
// # Ording
//
// On-file ordering is done following the C struct order.
//
// # Padding
//
// Although we are using C as a reference implementation, no padding is enforced on-file.
// In other words, on-file data is not padded according to the C specification and everything is packed in the file.
// Thus, this file is not suited for in-memory efficient representation.

#ifndef ALCOV_H
#define ALCOV_H

// Every integer should be considered as little endian, for every architecture.
#pragma scalar_storage_order little-endian

#include <stdint.h>

const uint64_t ALCOV_MAGIC = 0xdda28f766f636c61;

const uint64_t ALCOV_VERSION_MAJOR = 0;
const uint64_t ALCOV_VERSION_MINOR = 1;

// if this flag is set, alcov_blocks.edge_offset and alcov_blocks.nb_edges are defined according to their definition.
// otherwise, their value is undefined.
#define ALCOV_FLAG_EDGES 		(1 << 0) // edge coverage is available.
#define ALCOV_FLAG_COMPRESS		(1 << 1) // block section (and edge section if enabled) are compressed using LZMA2.
#define ALCOV_FLAG_INPUT_PATH	(1 << 2) // the first path in the path chunk is the path to the input for which we are measuring coverage.

// header of alcov.
struct __attribute__((packed)) alcov_hdr {
	uint64_t magic; 					// equals ALCOV_MAGIC, always the same across every version.
	uint64_t version_major; 			// equals ALCOV_VERSION_MAJOR, increases when the specification changes in a significant way.
	uint64_t version_minor; 			// equals ALCOV_VERSION_MINOR, increases when the specification changes include minor breaking changes.
	uint16_t nb_modules;				// number of blocks used during coverage.
	uint64_t nb_blocks; 				// number of modules.
	uint64_t nb_edges;					// number of edges.
	uint64_t modules_start;				// offset of modules chunk in file.
	uint64_t paths_start;				// offset of paths chunk in file.
	uint64_t blocks_start;				// offset of blocks chunk in file.
	uint64_t edges_start;				// offset of edges chunk in file.
	uint16_t flags;						// optional flags.
};

struct __attribute__((packed)) alcov_segment {
	uint64_t module_offset;				// Offset from module's base address.
	uint64_t size;						// Size of the segment in bytes.
};

struct __attribute__((packed)) alcov_module {
	uint64_t base_address;				// base address of the module.
	int64_t path_offset;				// offset (in bytes) of the path from paths_start. < 0 if no path is provided.
	uint16_t nb_segments;				// number of segments in next array. must be at least 1.
	struct alcov_segment segments[];	// Segments in the modules.
};

struct __attribute__((packed)) alcov_block {
	uint64_t segment_offset;			// the block offset in its segment.
	uint32_t size;						// the size of the block.
	uint16_t module_id;					// the module ID in which the block lives.
	uint16_t segment_id;				// the segment ID in which the block lives.
	uint64_t nb_out_edges;				// number of outgoing edges. in the block's edge table. 0 if no outgoing edges.
	uint64_t out_edges_offset; 			// the offset (in bytes) in the outgoing edge table. only defined when the EDGES flag is set and nb_out_edges > 0.
	uint64_t nb_taken;					// the number of times the block has been traversed. 0 means it was not measured and this number is unknown.
};

struct __attribute__((packed)) alcov_out_edge {
	uint64_t dst_block_id; 				// the id of the outgoing block. dst block is implicitly determined by parsing alcov_blocks.
	uint64_t nb_taken; 					// the number of times the edge has been taken. 0 means it was not measured and this number is unknown.
};

#endif
