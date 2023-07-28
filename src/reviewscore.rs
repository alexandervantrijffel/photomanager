use async_graphql::Enum;

#[derive(Debug, Enum, Copy, Clone, Eq, PartialEq)]
pub enum ReviewScore {
    Best,
    Good,
    Worst,
    AlreadyReviewed,
}
impl ReviewScore {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReviewScore::Best => "001-best",
            ReviewScore::Good => "002-good",
            ReviewScore::Worst => "003-worst",
            ReviewScore::AlreadyReviewed => "already_reviewed",
        }
    }
}
pub fn get_review_scores() -> Vec<ReviewScore> {
    vec![ReviewScore::Best, ReviewScore::Good, ReviewScore::Worst]
}

pub fn get_review_scores_as_str() -> Vec<&'static str> {
    get_review_scores()
        .iter()
        .map(|score| score.as_str())
        .collect()
}
