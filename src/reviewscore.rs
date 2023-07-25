use async_graphql::Enum;

#[derive(Debug, Enum, Copy, Clone, Eq, PartialEq)]
pub enum ReviewScore {
    Best,
    Nah,
    Worst,
}
impl ReviewScore {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReviewScore::Best => "best",
            ReviewScore::Nah => "nah",
            ReviewScore::Worst => "worst",
        }
    }
}
pub fn get_review_scores_as_str() -> Vec<&'static str> {
    vec![
        ReviewScore::Best.as_str(),
        ReviewScore::Nah.as_str(),
        ReviewScore::Worst.as_str(),
    ]
}
