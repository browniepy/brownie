use std::collections::HashMap;

use itertools::Itertools;

use super::cards::poker::{Card, PokerValue};

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum HandType {
    HighCard(u8),
    Pair(u8),
    TwoPair(u8, u8),
    ThreeOfKind(u8),
    Straight(u8),
    Flush(u8),
    FullHouse(u8, u8),
    FourOfKind(u8),
    StraightFlush(u8),
    RoyalFlush,
}

pub trait EvaluatePoker {
    fn evaluate_hand(hand: &[Card]) -> HandType;
}

impl EvaluatePoker for Card {
    fn evaluate_hand(hand: &[Card]) -> HandType {
        let mut counts = HashMap::new();
        let mut suits = HashMap::new();

        for card in hand {
            *counts.entry(card.value()).or_insert(0) += 1;
            *suits
                .entry(match card {
                    Card::Joker(s)
                    | Card::Two(s)
                    | Card::Three(s)
                    | Card::Four(s)
                    | Card::Five(s)
                    | Card::Six(s)
                    | Card::Seven(s)
                    | Card::Eight(s)
                    | Card::Nine(s)
                    | Card::Ten(s)
                    | Card::Jack(s)
                    | Card::Queen(s)
                    | Card::King(s)
                    | Card::Ace(s) => s,
                })
                .or_insert(0) += 1;
        }

        let mut values: Vec<u8> = counts.keys().copied().collect();
        values.sort();

        let value_counts: Vec<u8> = counts.values().copied().collect();
        let suit_counts: Vec<u8> = suits.values().copied().collect();
        let max_count = value_counts.iter().max().unwrap_or(&0);

        let is_flush = suit_counts.contains(&5);
        let is_straight = is_consecutive(&values);

        if is_flush && is_straight {
            if values.last() == Some(&14) && values.first() == Some(&10) {
                return HandType::RoyalFlush;
            }

            return HandType::StraightFlush(*values.last().unwrap_or(&0));
        }

        if max_count == &4 {
            return HandType::FourOfKind(*values.last().unwrap_or(&0));
        }

        if max_count == &3 && value_counts.contains(&2) {
            let three_value = *counts
                .iter()
                .find(|(_, &count)| count == 3)
                .map(|(v, _)| v)
                .unwrap_or(&0);
            let two_value = *counts
                .iter()
                .find(|(_, &count)| count == 2)
                .map(|(v, _)| v)
                .unwrap_or(&0);
            return HandType::FullHouse(three_value, two_value);
        }

        if is_flush {
            return HandType::Flush(*values.last().unwrap_or(&0));
        }

        if is_straight {
            return HandType::Straight(*values.last().unwrap_or(&0));
        }

        if max_count == &3 {
            return HandType::ThreeOfKind(*values.last().unwrap_or(&0));
        }

        if max_count == &2 {
            let pairs_count = count_pairs(hand);

            if pairs_count == 2 {
                let mut pairs: Vec<u8> = counts
                    .iter()
                    .filter(|(_, &count)| count == 2)
                    .map(|(v, _)| *v)
                    .collect();
                pairs.sort();
                return HandType::TwoPair(pairs[1], pairs[0]);
            }

            let pair_value = *counts
                .iter()
                .find(|(_, &count)| count == 2)
                .map(|(v, _)| v)
                .unwrap_or(&0);
            return HandType::Pair(pair_value);
        }

        HandType::HighCard(*values.last().unwrap_or(&0))
    }
}

fn is_consecutive(values: &[u8]) -> bool {
    if values.len() != 5 {
        return false;
    }

    for i in 0..4 {
        if values[i] + 1 != values[i + 1] {
            return false;
        }
    }
    true
}

fn count_pairs(hand: &[Card]) -> usize {
    let hand = hand.iter().map(|c| c.key_name()).collect::<Vec<_>>();

    let pairs = hand.iter().unique().collect::<Vec<_>>();

    match pairs.len() {
        4 => 1,
        _ => 2,
    }
}

pub fn compare_hands(hand1: &[Card], hand2: &[Card]) -> std::cmp::Ordering {
    let hand_type1 = Card::evaluate_hand(hand1);
    let hand_type2 = Card::evaluate_hand(hand2);

    hand_type1.cmp(&hand_type2)
}
