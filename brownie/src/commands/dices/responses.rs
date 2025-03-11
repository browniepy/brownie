use poise::{CreateReply, ReplyHandle};
use types::dices::{DiceResult, Dices, Player, Selection};

use crate::{charge_single_bet, translate, Context, Error};

pub async fn initial<'b>(ctx: Context<'b>, player: &Player) -> Result<ReplyHandle<'b>, Error> {
    let content = if let Some(choice) = &player.selection {
        let choice_tr = match choice {
            Selection::Unpair => translate!(ctx, "unpairs"),
            Selection::Pair => translate!(ctx, "pairs"),
        };

        translate!(ctx, "dices-choice-start", choice: choice_tr)
    } else {
        let number = player.number.unwrap();
        translate!(ctx, "dices-number-start", number: number)
    };

    let res = ctx.reply(content).await?;

    Ok(res)
}

pub async fn result(
    ctx: Context<'_>,
    message: ReplyHandle<'_>,
    dices: &Dices,
) -> Result<(), Error> {
    let num = dices.dices.iter().sum::<i32>();

    let choice = match dices.check_pair(num) {
        Selection::Unpair => translate!(ctx, "unpairs"),
        Selection::Pair => translate!(ctx, "pairs"),
    };

    let mut content = translate!(ctx, "dices-result", choice: choice, dice1: dices.dices[0], dice2: dices.dices[1]);

    match dices.player_result() {
        DiceResult::ChoiceWin => {
            charge_single_bet(ctx, ctx.author().id, dices.player.bet, true).await?;
            let cont = translate!(ctx, "dices-win", bet: dices.player.bet);
            content.push_str(&format!("\n{}", cont));
        }
        DiceResult::NumWin => {
            charge_single_bet(ctx, ctx.author().id, dices.player.bet * 2, true).await?;
            let cont = translate!(ctx, "dices-number-win", bet: dices.player.bet * 2);
            content.push_str(&format!("\n{}", cont));
        }
        DiceResult::NumberLose => {
            charge_single_bet(ctx, ctx.author().id, dices.player.bet, false).await?;
            let cont = translate!(ctx, "dices-number-lose", bet: dices.player.bet);
            content.push_str(&format!(", {}", cont));
        }
        _ => {
            charge_single_bet(ctx, ctx.author().id, dices.player.bet, false).await?;
            let cont = translate!(ctx, "dices-lose", bet: dices.player.bet);
            content.push_str(&format!(", {}", cont));
        }
    }

    message
        .edit(ctx, CreateReply::default().content(content))
        .await?;

    Ok(())
}
