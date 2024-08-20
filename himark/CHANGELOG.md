# Change Log

## [Unreleased] - 2024-06-15

## [0.2.0] - 2024-06-15 
 
Quality of life features that extend the range of items compatible with `#[mark]`

### Added
- Tests which can serve as reference point for users.
- Support `#[himark::mark]` for `enum` and `union` items.
- Support for generic parameters.
- Better module level docs.
- CHANGELOG.md

### Changed
- README.md has been updated.
 
## [0.1.0] - 2024-06-15
  
Initial release, support only for non-generic `struct` items.
 
### Added
- Attribute `#[mark(Marker1, Marker2, ...)]` which generates empty `impl`s for given trait names.
- Attribute `#[marker]` which can be applied to traits in to safeguard against using non empty trait as marker.
