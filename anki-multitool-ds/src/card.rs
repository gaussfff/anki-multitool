use anyhow::{Error, Result, anyhow};
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq, Eq)]
pub struct Card {
    pub front: String,
    pub back: String,
}

impl FromStr for Card {
    type Err = Error;

    /// Parses a string in the format "front - back" into a Card.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.contains('-') || s.trim().starts_with('-') || s.trim().ends_with('-') {
            return Err(anyhow!(
                "invalid card format, expected 'front - back', got: {s}"
            ));
        }

        let parts: Vec<&str> = s.splitn(2, "-").collect();

        let [front, back] = parts.as_slice() else {
            return Err(anyhow!(
                "invalid card format, expected 'front - back', got: {s}"
            ));
        };

        Ok(Self {
            front: front.trim().to_string(),
            back: back.trim().to_string(),
        })
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} - {}", self.front, self.back)
    }
}

#[cfg(test)]
mod card_tests {
    use super::Card;
    use std::str::FromStr;

    #[test]
    pub fn test_card_from_str() {
        let mut card = Card::from_str("Front-Back ").expect("wrong format of str");

        assert_eq!(card.front, "Front");
        assert_eq!(card.back, "Back");

        card = Card::from_str("Front - Back ").expect("wrong format of str");

        assert_eq!(card.front, "Front");
        assert_eq!(card.back, "Back");

        card = Card::from_str("Front Text - Back Text").expect("wrong format of str");

        assert_eq!(card.front, "Front Text");
        assert_eq!(card.back, "Back Text");

        card = Card::from_str("Front - Back - Text").expect("wrong format of str");

        assert_eq!(card.front, "Front");
        assert_eq!(card.back, "Back - Text");

        card = Card::from_str("Front - - Back Text").expect("wrong format of str");

        assert_eq!(card.front, "Front");
        assert_eq!(card.back, "- Back Text");
    }

    #[test]
    pub fn test_wrong_format() {
        let mut card = Card::from_str("Front Back");
        assert!(card.is_err());

        card = Card::from_str("Front -");
        assert!(card.is_err())
    }
}
