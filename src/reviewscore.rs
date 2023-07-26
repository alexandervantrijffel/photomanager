use async_graphql::Enum;

#[derive(Debug, Enum, Copy, Clone, Eq, PartialEq)]
pub enum ReviewScore {
    Best,
    Nah,
    Worst,
    AlreadyReviewed,
}
impl ReviewScore {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReviewScore::Best => "best",
            ReviewScore::Nah => "nah",
            ReviewScore::Worst => "worst",
            ReviewScore::AlreadyReviewed => "already_reviewed",
        }
    }
}
pub fn get_review_scores() -> Vec<ReviewScore> {
    vec![ReviewScore::Best, ReviewScore::Nah, ReviewScore::Worst]
}

pub fn get_review_scores_as_str() -> Vec<&'static str> {
    get_review_scores()
        .iter()
        .map(|score| score.as_str())
        .collect()
}
