use async_graphql::Enum;

#[derive(Debug, Enum, Copy, Clone, Eq, PartialEq)]
pub enum ReviewScore {
    Best,
    Good,
    Worst,
    AlreadyReviewed,
}
impl ReviewScore {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Best => "001-best",
            Self::Good => "002-good",
            Self::Worst => "003-worst",
            Self::AlreadyReviewed => "already_reviewed",
        }
    }
}

#[must_use]
pub fn get_review_scores() -> Vec<ReviewScore> {
    vec![ReviewScore::Best, ReviewScore::Good, ReviewScore::Worst]
}

#[must_use]
pub fn get_review_scores_as_str() -> Vec<&'static str> {
    get_review_scores()
        .iter()
        .map(ReviewScore::as_str)
        .collect()
}
