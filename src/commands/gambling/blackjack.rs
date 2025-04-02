use crate::common::{Context, Error};
use std::{cmp::Ordering, fmt::Display, time::Duration};
use poise::serenity_prelude::{self as serenity, CreateInteractionResponseMessage};
use rand::seq::SliceRandom;

#[derive(Clone)]
enum Suite {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
}

impl Display for Suite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::Hearts => "\u{2665}",
            Self::Diamonds => "\u{2666}",
            Self::Clubs => "\u{2663}",
            Self::Spades => "\u{2660}",
        })
    }
}

impl Suite {
    fn suites() -> impl Iterator<Item = Self> {
        [Self::Hearts, Self::Diamonds, Self::Clubs, Self::Spades].iter().cloned()
    }
}

#[derive(Clone)]
enum Rank {
    Pip(u8),
    Jack,
    King,
    Queen,
    Ace,
}

impl Display for Rank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pip(n) => write!(f, "{}", n),
            Self::Jack => write!(f, "J"),
            Self::King => write!(f, "K"),
            Self::Queen => write!(f, "Q"),
            Self::Ace => write!(f, "A"),
        }
    }
}

impl Rank {
    fn ranks() -> impl Iterator<Item = Self> {
        (2..=10).map(|n| Self::Pip(n))
            .chain(vec![Self::Jack, Self::King, Self::Queen, Self::Ace])
    }

    fn value(&self, ace_would_bust: bool) -> u8 {
        match self {
            Self::Ace if ace_would_bust => 1,
            Self::Pip(n) => *n,
            Self::Jack | Self::King | Self::Queen => 10,
            Self::Ace => 11,
        }
    }
}

#[derive(Clone)]
struct Card {
    suite: Suite,
    rank: Rank,
}

impl Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.suite, self.rank)
    }
}

impl Card {
    fn new(suite: Suite, rank: Rank) -> Self {
        Self { suite, rank }
    }

    fn deck() -> impl Iterator<Item = Card> {
        let mut deck = vec![];

        for rank in Rank::ranks() {
            for suite in Suite::suites() {
                deck.push(Card::new(suite, rank.clone()));
            }
        }

        deck.into_iter()
    }

    fn value(&self, ace_would_bust: bool) -> u8 {
        self.rank.value(ace_would_bust)
    }
}

/// Blackjack!
#[poise::command(slash_command, prefix_command, aliases("jackblack", "bj", "21"))]
pub async fn blackjack(ctx: Context<'_>, amount: String) -> Result<(), Error>
{
    let mut tx = ctx.data().database.begin().await?;
    let mut balance = super::get_balance(ctx.author().id, &mut *tx).await?;

    let amount = match amount.to_lowercase().as_str() {
        "all" => balance,
        "half" => balance / 2,
        input => {
            if input.ends_with('%') {
                let percent: f64 = match input[..input.len() - 1].parse::<f64>() {
                    Ok(x) => x,
                    Err(_) => {
                        ctx.reply(format!("{input} is not a valid percent.")).await?;
                        return Ok(());
                    }
                } / 100f64;

                (balance as f64 * percent) as i32
            } else {
                match input.parse() {
                    Ok(n) => n,
                    Err(_) => {
                        ctx.reply("Any one of a number, all, half, or a percent are allowed as arguments.").await?;
                        return Ok(());
                    }
                }
            }
        }
    };

    if amount < 1 {
        ctx.reply("You cannot wager less than 1 token.").await?;
        return Ok(());
    }

    if balance < amount {
        ctx.reply(format!("You do not have enough tokens (**{balance}**) to wager this amount.")).await?;
        return Ok(());
    }

    let mut deck: Vec<_> = Card::deck()
        .filter(|card| !matches!(card, Card { rank: Rank::Jack, .. }))
        .collect();

    deck.shuffle(&mut rand::thread_rng());

    let dealers_hand: Vec<Card> = vec![deck.pop().unwrap(), deck.pop().unwrap()];
    let mut players_hand: Vec<Card> = vec![deck.pop().unwrap(), deck.pop().unwrap()];

    let msg = ctx.reply("Just a second...").await?;

    loop {
        let dealers_count = dealers_hand.iter().fold(0, |acc, card| acc + card.value(acc + 11 > 21));
        let players_count = players_hand.iter().fold(0, |acc, card| acc + card.value(acc + 11 > 21));

        if players_count > 21 {
            msg.edit(ctx, poise::CreateReply::default()
                .components(vec![])
                .content(format!(
                    concat!(
                        "**Dealer's hand**: {} ({})\n",
                        "**Your hand**: {} ({})\n\n",
                        "**Bet**: {}\n",
                        "Bust! You've lost **{}** tokens."
                    ),
                    dealers_hand.iter().map(|card| format!("`{card}`")).collect::<Vec<String>>().join(", "),
                    dealers_count,
                    players_hand.iter().map(|card| format!("`{card}`")).collect::<Vec<String>>().join(", "),
                    players_count,
                    amount, amount
            ))).await?;

            balance -= amount;
            break;
        }

        let reply = {
            let components = vec![serenity::CreateActionRow::Buttons(vec![
                serenity::CreateButton::new("blackjack_hit")
                    .label("Hit")
                    .style(poise::serenity_prelude::ButtonStyle::Primary),
                serenity::CreateButton::new("blackjack_hold")
                    .label("Hold")
                    .style(poise::serenity_prelude::ButtonStyle::Primary),
            ])];

            poise::CreateReply::default()
                .content(
                    format!(
                        concat!(
                            "**Dealer's hand**: {} ({})\n",
                            "**Your hand**: {} ({})\n\n",
                            "**Bet**: {}"
                        ),
                        format!("`{}`, `XX`", dealers_hand[0]),
                        dealers_hand[0].value(matches!(dealers_hand[0], Card { rank: Rank::Ace, .. })),
                        players_hand.iter().map(|card| format!("`{card}`")).collect::<Vec<String>>().join(", "),
                        players_count,
                        amount,
                    )
                )
                .components(components)
        };

        msg.edit(ctx, reply).await?;

        let Some(mci) = serenity::ComponentInteractionCollector::new(ctx.serenity_context())
            .timeout(Duration::from_secs(120))
            .filter(move |mci| mci.data.custom_id.starts_with("blackjack")).await else {
                ctx.reply("failed interaction!").await?;
                return Ok(());
        };

        if mci.member.clone().unwrap().user.id == ctx.author().id {
            mci.create_response(ctx,
                serenity::CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .ephemeral(true)
                        .content("You cannot interact with this message."))).await?;

            continue;
        }

        mci.create_response(ctx, serenity::CreateInteractionResponse::Acknowledge).await?;

        match &mci.data.custom_id[..] {
            "blackjack_hit" => {
                players_hand.push(deck.pop().unwrap());
            }
            "blackjack_hold" => {
                let dealers_hand = dealers_hand.into_iter()
                    .chain(deck.into_iter())
                    .scan(0u8, |acc, card| {
                        if *acc >= 17 {
                            None
                        } else {
                            *acc += card.value(*acc + 11 > 21);
                            Some(card)
                        }
                    }).collect::<Vec<_>>();

                let dealers_count = dealers_hand.iter()
                    .fold(0, |acc, card| acc + card.value(acc + 11 > 21));

                let s = match dealers_count.cmp(&players_count) {
                    Ordering::Less => {
                        if players_count == 21 && players_hand.len() == 2 {
                            let amount = amount * 3 / 2;
                            balance += amount;
                            format!("You've won with a Blackjack! You've gained **{amount}** tokens.")
                        } else {
                            balance += amount;
                            format!("You've won! **{amount}** tokens have been added to your account.")
                        }
                    }
                    Ordering::Greater if dealers_count > 21 => {
                        if players_count == 21 && players_hand.len() == 2 {
                            let amount = amount * 3 / 2;
                            balance += amount;
                            format!("You've won with a Blackjack! You've gained **{amount}** tokens.")
                        } else {
                            balance += amount;
                            format!("You've won! **{amount}** tokens have been added to your account.")
                        }
                    }
                    Ordering::Equal => {
                        format!("A draw!")
                    }
                    Ordering::Greater => {
                        balance -= amount;
                        format!("You've lost. **{amount}** tokens to the dealer.")
                    }
                };

                super::change_balance(ctx.author().id, balance, &mut *tx).await?;
                tx.commit().await?;

                msg.edit(ctx, poise::CreateReply::default()
                    .components(vec![])
                    .content(
                        format!(
                            concat!(
                                "**Dealer's hand**: {} ({})\n",
                                "**Your hand**: {} ({})\n\n",
                                "**Bet**: {}\n",
                                "{}"
                            ),
                            dealers_hand.iter().map(|card| format!("`{card}`")).collect::<Vec<String>>().join(", "),
                            dealers_count,
                            players_hand.iter().map(|card| format!("`{card}`")).collect::<Vec<String>>().join(", "),
                            players_count,
                            amount, s
                        )
                    )).await?;

                return Ok(());
            }
            _ => {
                ctx.reply("Invalid interaction response.").await?;
                return Ok(());
            }
        }
    }

    super::change_balance(ctx.author().id, balance, &mut *tx).await?;
    tx.commit().await?;

    Ok(())
}
