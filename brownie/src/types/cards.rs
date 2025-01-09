use std::{cmp::Ordering, collections::HashMap};

use rand::seq::SliceRandom;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PokerCard {
    One(Suit),
    Two(Suit),
    Three(Suit),
    Four(Suit),
    Five(Suit),
    Six(Suit),
    Seven(Suit),
    Eight(Suit),
    Nine(Suit),
    Ten(Suit),
    Jack(Suit),
    Queen(Suit),
    King(Suit),
    Ace(Suit),
}

#[derive(Clone, Debug)]
pub enum NimCard {
    Zero,
    One,
    Two,
    Three,
}

impl NimCard {
    pub fn std_deck() -> Vec<Self> {
        let mut deck = Vec::new();

        for _ in 0..10 {
            deck.extend([Self::Zero, Self::One, Self::Two, Self::Three]);
        }

        deck.shuffle(&mut rand::thread_rng());
        deck
    }

    pub fn value(&self) -> u8 {
        match *self {
            Self::Zero => 0,
            Self::One => 1,
            Self::Two => 2,
            Self::Three => 3,
        }
    }
}

impl PokerCard {
    pub fn std_deck() -> Vec<Self> {
        let mut deck = Vec::new();

        for pale in [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades] {
            deck.extend([
                Self::Two(pale),
                Self::Three(pale),
                Self::Four(pale),
                Self::Five(pale),
                Self::Six(pale),
                Self::Seven(pale),
                Self::Eight(pale),
                Self::Nine(pale),
                Self::Ten(pale),
                Self::Queen(pale),
                Self::King(pale),
                Self::Ace(pale),
                Self::Jack(pale),
            ]);
        }

        deck.shuffle(&mut rand::thread_rng());
        deck
    }

    pub fn value(&self) -> u8 {
        match *self {
            Self::One(_) => 1,
            Self::Two(_) => 2,
            Self::Three(_) => 3,
            Self::Four(_) => 4,
            Self::Five(_) => 5,
            Self::Six(_) => 6,
            Self::Seven(_) => 7,
            Self::Eight(_) => 8,
            Self::Nine(_) => 9,
            Self::Ten(_) | Self::Queen(_) | Self::King(_) | Self::Jack(_) => 10,
            Self::Ace(_) => 11,
        }
    }

    pub fn suit(&self) -> Suit {
        match *self {
            Self::Two(s)
            | Self::Three(s)
            | Self::Four(s)
            | Self::Five(s)
            | Self::Six(s)
            | Self::Seven(s)
            | Self::Eight(s)
            | Self::Nine(s)
            | Self::Ten(s)
            | Self::Jack(s)
            | Self::Queen(s)
            | Self::King(s)
            | Self::Ace(s)
            | Self::One(s) => s,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum PokerHandRank {
    HighCard(Vec<PokerCard>),
    OnePair(PokerCard, Vec<PokerCard>),
    TwoPair(PokerCard, PokerCard, PokerCard),
    ThreeOfAKind(PokerCard, Vec<PokerCard>),
    Straight(PokerCard),
    Flush(Vec<PokerCard>),
    FullHouse(PokerCard, PokerCard),
    FourOfAKind(PokerCard, PokerCard),
    StraightFlush(PokerCard),
    RoyalFlush,
}

impl PokerHandRank {
    pub fn rank_value(&self) -> u8 {
        match *self {
            Self::HighCard(_) => 1,
            Self::OnePair(_, _) => 2,
            Self::TwoPair(_, _, _) => 3,
            Self::ThreeOfAKind(_, _) => 4,
            Self::Straight(_) => 5,
            Self::Flush(_) => 6,
            Self::FullHouse(_, _) => 7,
            Self::FourOfAKind(_, _) => 8,
            Self::StraightFlush(_) => 9,
            Self::RoyalFlush => 10,
        }
    }
}

impl PartialOrd for PokerHandRank {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PokerHandRank {
    fn cmp(&self, other: &Self) -> Ordering {
        // Primero comparamos los rangos
        let rank_comparison = self.rank_value().cmp(&other.rank_value());
        if rank_comparison != Ordering::Equal {
            return rank_comparison;
        }

        // Si los rangos son iguales, comparamos los valores específicos
        match (self, other) {
            (PokerHandRank::HighCard(cards1), PokerHandRank::HighCard(cards2)) => cards1
                .iter()
                .rev()
                .zip(cards2.iter().rev())
                .map(|(c1, c2)| c1.value().cmp(&c2.value()))
                .find(|&ord| ord != Ordering::Equal)
                .unwrap_or(Ordering::Equal),
            (PokerHandRank::OnePair(pair1, kickers1), PokerHandRank::OnePair(pair2, kickers2)) => {
                match pair1.value().cmp(&pair2.value()) {
                    Ordering::Equal => kickers1
                        .iter()
                        .rev()
                        .zip(kickers2.iter().rev())
                        .map(|(c1, c2)| c1.value().cmp(&c2.value()))
                        .find(|&ord| ord != Ordering::Equal)
                        .unwrap_or(Ordering::Equal),
                    ord => ord,
                }
            }
            // Implementa comparaciones similares para otros tipos de manos
            _ => Ordering::Equal,
        }
    }
}

fn is_straight(hand: &[PokerCard]) -> Option<PokerCard> {
    // Ordenar las cartas por valor
    let mut sorted_values: Vec<u8> = hand.iter().map(|card| card.value()).collect();
    sorted_values.sort_unstable();

    // Caso especial para elAs que puede ser bajo o alto
    let has_ace = sorted_values.contains(&14);

    // Verificar escalera normal
    for window in sorted_values.windows(5) {
        if window.windows(2).all(|pair| pair[1] - pair[0] == 1) {
            return Some(
                hand.iter()
                    .find(|&card| card.value() == window[4])
                    .unwrap()
                    .clone(),
            );
        }
    }

    // Verificar escalera especial con As (A-2-3-4-5)
    if has_ace && sorted_values.starts_with(&[2, 3, 4, 5]) && sorted_values.contains(&14) {
        return Some(hand.iter().find(|&card| card.value() == 5).unwrap().clone());
    }

    None
}

fn is_flush(hand: &[PokerCard]) -> Option<Vec<PokerCard>> {
    let first_suit = hand[0].suit();
    if hand.iter().all(|card| card.suit() == first_suit) {
        let mut flush_cards = hand.to_vec();
        flush_cards.sort_by(|a, b| b.value().cmp(&a.value()));
        Some(flush_cards)
    } else {
        None
    }
}

fn count_card_values(hand: &[PokerCard]) -> HashMap<u8, Vec<PokerCard>> {
    let mut value_counts: HashMap<u8, Vec<PokerCard>> = HashMap::new();

    for card in hand {
        value_counts
            .entry(card.value())
            .or_insert_with(Vec::new)
            .push(card.clone());
    }

    value_counts
}

fn find_pairs(hand: &[PokerCard]) -> Option<PokerHandRank> {
    let value_counts = count_card_values(hand);

    // Encontrar pares, tercias y cuatro de un tipo
    let mut pairs = Vec::new();
    let mut three_of_a_kind = None;
    let mut four_of_a_kind = None;

    for (value, cards) in value_counts.iter() {
        match cards.len() {
            4 => {
                four_of_a_kind = Some(cards.clone());
            }
            3 => {
                three_of_a_kind = Some(cards.clone());
            }
            2 => {
                pairs.push(cards.clone());
            }
            _ => {}
        }
    }

    // Clasificar pares por valor
    pairs.sort_by(|a, b| b[0].value().cmp(&a[0].value()));

    // Manejar diferentes combinaciones
    match (four_of_a_kind, three_of_a_kind, pairs.len()) {
        // Cuatro de un tipo (Poker)
        (Some(four), _, _) => {
            let kicker = value_counts
                .iter()
                .find(|&(k, _)| *k != four[0].value())
                .map(|(_, cards)| cards[0].clone())
                .unwrap();
            Some(PokerHandRank::FourOfAKind(four[0].clone(), kicker))
        }
        // Full House
        (_, Some(three), 1) => Some(PokerHandRank::FullHouse(
            three[0].clone(),
            pairs[0][0].clone(),
        )),
        // Tercia
        (_, Some(three), _) => {
            let kickers: Vec<PokerCard> = value_counts
                .iter()
                .filter(|&(k, _)| *k != three[0].value())
                .flat_map(|(_, cards)| cards.clone())
                .collect();

            Some(PokerHandRank::ThreeOfAKind(three[0].clone(), kickers))
        }
        // Dos pares
        (_, _, 2) => {
            let kicker = value_counts
                .iter()
                .find(|&(k, _)| !pairs.iter().any(|pair| pair[0].value() == *k))
                .map(|(_, cards)| cards[0].clone())
                .unwrap();
            Some(PokerHandRank::TwoPair(
                pairs[0][0].clone(),
                pairs[1][0].clone(),
                kicker,
            ))
        }
        // Un par
        (_, _, 1) => {
            let pair = pairs[0].clone();
            let kickers: Vec<PokerCard> = value_counts
                .iter()
                .filter(|&(k, _)| *k != pair[0].value())
                .flat_map(|(_, cards)| cards.clone())
                .collect();

            Some(PokerHandRank::OnePair(pair[0].clone(), kickers))
        }
        _ => None,
    }
}

pub fn evaluate_hand(hand: &[PokerCard]) -> PokerHandRank {
    // Implementación de la lógica de evaluación de manos
    // Esta es una implementación simplificada y necesitaría más trabajo

    // Verificar si es Royal Flush
    let is_same_suit = hand.iter().all(|card| card.suit() == hand[0].suit());
    let card_values: Vec<u8> = hand.iter().map(|card| card.value()).collect();

    if is_same_suit
        && card_values.contains(&10)
        && card_values.contains(&11)
        && card_values.contains(&12)
        && card_values.contains(&13)
        && card_values.contains(&14)
    {
        return PokerHandRank::RoyalFlush;
    }

    // Verificar si es Straight Flush
    match (is_flush(hand), is_straight(hand)) {
        (Some(flush_cards), Some(straight_card))
            if flush_cards
                .iter()
                .any(|c| c.value() == straight_card.value()) =>
        {
            return PokerHandRank::StraightFlush(straight_card);
        }
        _ => {}
    }

    // Verificar Straight
    if let Some(straight_card) = is_straight(hand) {
        return PokerHandRank::Straight(straight_card);
    }

    // Verificar Flush
    if let Some(flush_cards) = is_flush(hand) {
        return PokerHandRank::Flush(flush_cards);
    }

    if let Some(pair_hand) = find_pairs(hand) {
        return pair_hand;
    }

    // Resto de la lógica de evaluación...
    // Esta es una implementación básica que necesitaría ser expandida

    // Por defecto, devolver la carta más alta
    let mut sorted_hand = hand.to_vec();
    sorted_hand.sort_by(|a, b| b.value().cmp(&a.value()));
    PokerHandRank::HighCard(sorted_hand)
}

pub fn compare_hands(hand1: &[PokerCard], hand2: &[PokerCard]) -> Ordering {
    let rank1 = evaluate_hand(hand1);
    let rank2 = evaluate_hand(hand2);

    rank1.cmp(&rank2)
}
