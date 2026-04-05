use crate::error::BtcResult;

use super::types::VisualScore;

/// Checks cross-page consistency of visual scores.
pub struct ConsistencyChecker;

impl ConsistencyChecker {
    /// Compute the mean score and check whether any page deviates more than
    /// 15 percentage points from the mean. Returns the mean score.
    pub fn check(scores: &[VisualScore]) -> BtcResult<f64> {
        if scores.is_empty() {
            return Ok(0.0);
        }

        let mean = Self::mean(scores);
        let max_deviation = 15.0;

        for score in scores {
            let deviation = (score.score.value() - mean).abs();
            if deviation > max_deviation {
                tracing::warn!(
                    page = %score.page,
                    score = score.score.value(),
                    mean = mean,
                    deviation = deviation,
                    "Page score deviates >15% from mean"
                );
            }
        }

        Ok(mean)
    }

    /// Returns `true` if all scores meet the threshold and no page deviates
    /// more than 15 points from the mean.
    pub fn is_consistent(scores: &[VisualScore], threshold: f64) -> bool {
        if scores.is_empty() {
            return true;
        }

        let mean = Self::mean(scores);

        scores.iter().all(|s| {
            s.score.value() >= threshold && (s.score.value() - mean).abs() <= 15.0
        })
    }

    fn mean(scores: &[VisualScore]) -> f64 {
        let sum: f64 = scores.iter().map(|s| s.score.value()).sum();
        sum / scores.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Score;
    use chrono::Utc;

    fn make_score(page: &str, value: f64) -> VisualScore {
        VisualScore {
            page: page.to_string(),
            score: Score::new(value),
            feedback: String::new(),
            timestamp: Utc::now(),
        }
    }

    #[test]
    fn test_check_empty() {
        let result = ConsistencyChecker::check(&[]).unwrap();
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_check_uniform_scores() {
        let scores = vec![
            make_score("home", 90.0),
            make_score("about", 92.0),
            make_score("contact", 88.0),
        ];
        let mean = ConsistencyChecker::check(&scores).unwrap();
        assert!((mean - 90.0).abs() < 1.0);
    }

    #[test]
    fn test_check_with_outlier() {
        let scores = vec![
            make_score("home", 95.0),
            make_score("about", 94.0),
            make_score("broken", 60.0), // large deviation
        ];
        let mean = ConsistencyChecker::check(&scores).unwrap();
        // Mean ~ 83.0; broken page deviates > 15 from mean (logged as warning)
        assert!(mean > 80.0 && mean < 85.0);
    }

    #[test]
    fn test_is_consistent_all_passing() {
        let scores = vec![
            make_score("home", 92.0),
            make_score("about", 90.0),
            make_score("contact", 91.0),
        ];
        assert!(ConsistencyChecker::is_consistent(&scores, 85.0));
    }

    #[test]
    fn test_is_consistent_below_threshold() {
        let scores = vec![
            make_score("home", 92.0),
            make_score("about", 80.0), // below 85 threshold
        ];
        assert!(!ConsistencyChecker::is_consistent(&scores, 85.0));
    }

    #[test]
    fn test_is_consistent_large_deviation() {
        let scores = vec![
            make_score("home", 100.0),
            make_score("about", 100.0),
            make_score("bad", 85.0), // deviation from mean ~95 is 10, within 15
        ];
        // mean = 95, max deviation = 10 <= 15, all >= 85
        assert!(ConsistencyChecker::is_consistent(&scores, 85.0));

        let scores2 = vec![
            make_score("home", 100.0),
            make_score("about", 100.0),
            make_score("bad", 86.0), // all above threshold but deviation ~14/3 from mean ~95.3
        ];
        // mean = 95.33, deviation for bad = 9.33 <= 15
        assert!(ConsistencyChecker::is_consistent(&scores2, 85.0));
    }

    #[test]
    fn test_is_consistent_empty() {
        assert!(ConsistencyChecker::is_consistent(&[], 90.0));
    }

    #[test]
    fn test_is_consistent_with_big_spread() {
        // Force a deviation > 15
        let scores = vec![
            make_score("high", 100.0),
            make_score("low", 68.0), // mean=84, deviation from mean: 16 > 15
        ];
        assert!(!ConsistencyChecker::is_consistent(&scores, 60.0));
    }
}
