# alcov: a file format for binary-only coverage

alcov has been designed as a simple file format to get coverage information from
binaries, while using minimum space.
The main goal of alcov is to provide a clear, simple and usable specification for block & edge coverage, usable across different projects.
It does not aim at replacing other formats, but instead to provide a simple and easily implementable format.
It has been hightly inspired by [drcov](https://dynamorio.org/page_drcov.html).

## Alternative coverage file formats

Other coverage file format, among others:
- [drcov](https://dynamorio.org/page_drcov.html). An older version of drcov [has been described in a blogpost](https://www.ayrx.me/drcov-file-format/).
- [profraw](https://leodido.dev/demystifying-profraw/), used by the LLVM project for storing coverage information.
- [gcov](https://github.com/gcc-mirror/gcc/blob/master/gcc/gcov-io.h). GCC's coverage format.

## Versions

The latest version of alcov is `v0.1`.

- [alcov v0 specification](v0)

## Library

A library (developed in Rust) is available to manipulate alcov.
It can be found [in the alcov subdirectory](alcov).
The crate has features describing the version to compile, following the version of the alcov specification.

## CLI

A user-friendly CLI for alcov is available [in the alcov-cli subdirectory](alcov-cli).
Once again, the version of alcov specification can be set through a feature.

### Compatibility between versions

Nearly strong no compatibility guarantee is enforced between versions.
The corresponding specification and header file should be used for a given version.
The only guarantee shared between versions are the 3 first fields of the trace file:

- The magic (64 bits) is always equal to `ALCOV_MAGIC` (`0xdda28f766f636c61`).
- The major version (unsigned 64 bits integer) is incremented by one for each new important release.
- The minor version (unsigned 64 bits integer) is incremented by one for each minor revision.

During v0, minor version changes can be significant and break completely from one version to another.