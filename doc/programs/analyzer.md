# analyzer

Program `analyzer` (Rust): calculates the score of each shot, the session score and the grouping score. Request: [interfaces.md](../interfaces.md#analyzer).

## Responsibilities
- Ring score of each shot, using the edge rule with the bullet diameter.
- Session score: sum of all shot scores.
- Grouping score: the highest session score that can be reached by shifting the rings, i.e. the score the group would have had with perfectly adjusted sights.
- Mean point of impact.

The analyzer is stateless: everything it needs is in the request. It doesn't read the database; `session_manager` (live, after every shot change), `exporter` and `web_server` send it the shots, the target type and the bullet diameter. The analyzer always generates the rings from the anchors with the shared function `generate_rings`; a `rings` field in the request is ignored.

## Technology
`targetscope_common` (bus, model, geometry, ring generation), `tokio`. The calculation itself is plain Rust without dependencies, in its own module (`scoring.rs`), so it can be tested without Redis.

## Definitions
- Shots are converted to Cartesian millimeters `p_j = (x_j, y_j)` (see [interfaces.md](../interfaces.md#shot)).
- Ring radii `r_1 < r_2 < … < r_n` (`radius_mm` of the generated rings, see [ring generation](../interfaces.md#targettype)) with scores `s_1 > s_2 > … > s_n`. `r_1` is the inner white ring, `r_n` the outermost ring.
- Bullet radius `b = bullet_diameter_mm / 2`.

### Shot score (edge rule)
A shot touches a ring if its edge reaches the ring line: `distance - b <= r_i`. The shot scores the ring with the smallest radius it touches:
```
score(d) = s_i for the smallest i with d - b <= r_i,  0 if there is none
```
where `d` is the distance of the shot center from the ring center. A shot exactly on the line (`d - b == r_i`) counts as touching. Comparisons use a tolerance of 1e-9 mm.

### Session score
```
score = Σ_j score(|p_j|)
```

### Grouping score
The rings are shifted by `c = (c_x, c_y)` so that the total score is as high as possible:
```
grouping_score = max over c of  S(c) = Σ_j score(|p_j - c|)
grouping_shift = the c that reaches it
```
`c = (0, 0)` is always a candidate, so `grouping_score >= score`.

### Mean point of impact
`mean_point = (1/N) Σ_j p_j`; `null` without shots.

## Grouping algorithm
`S(c)` is a step function (it only changes when a ring edge crosses a shot), so gradient methods don't work. A two-stage grid search is used:

1. **Search area:** the bounding box of all shots, extended on each side by `r_n + b`. Outside this area no shot touches any ring, so `S(c) = 0` there.
2. **Coarse grid:** step `h = min(0.5 mm, (r_1 + b) / 5)`. Evaluate `S(c)` at every grid point, plus `(0, 0)` and the mean point. Keep every point with the best coarse score (at least the 10 best points, at most 1 000, ordered by the tie rule of step 4). Many points usually share the best score; a higher zone narrower than `h` lies next to one of them, so refining only 10 of them would miss it in about 6 % of sessions.
3. **Fine grid:** around each kept point, a grid of ±`h` with step `h / 10`. Evaluate `S(c)`.
4. **Result:** the highest `S(c)`. Ties are resolved by the smallest shift `|c|`, then by smallest `c_x`, then `c_y`, so the result is deterministic.

With a score lookup by binary search over the ring radii, a 60-shot session on a 155 mm target needs a few tens of millions of shot evaluations in the worst case, which takes well under a second. The calculation runs in `spawn_blocking`.

Known limit: the grid search is not guaranteed to find the exact maximum. A scoring zone (the area where several shots reach a high ring at the same time) smaller than the coarse step can be missed if no coarse point lands near it. Refining the 10 best points instead of only the best reduces this risk. The brute-force comparison test (below) measures how often it happens.

## Interface
```rust
pub fn analyze(shots: &[Shot], target: &TargetType, bullet_diameter_mm: f64) -> AnalysisResult;
pub fn shot_score(distance_mm: f64, rings: &[Ring], bullet_radius_mm: f64) -> i64;
```
`analyze_shots` requests call `analyze` and return the result.

Validation: target type as in [interfaces.md](../interfaces.md#targettype) (anchor radii increasing, scores decreasing); `bullet_diameter_mm > 0`; every Shot within its ranges. Otherwise `validation_failed`.

Because `session_manager` analyzes after every shot, a typical call (up to 60 shots) must answer within 200 ms; the worst case stays well under a second (see above).

## Testing
All test cases with their expected outcomes: [testing/analyzer.md](../testing/analyzer.md).
