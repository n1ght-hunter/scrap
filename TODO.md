# TODO

## Enums

- [ ] Niche optimization: for two-variant enums where one variant is a unit and the other carries a value, exploit invalid bit patterns ("niches") instead of storing a separate discriminant. E.g. `Option<&T>` uses null since references are never null, `Option<bool>` uses 2 since bool only occupies 0/1. Matches Rust's niche layout optimization. This should be deterministic and not require a heuristic, since the niche is always the same for a given type. This optimization should be applied to enums with more than two variants if it can be determined that some variants are never used, but this is not required for MVP.
