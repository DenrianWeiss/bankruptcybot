use std::env;

use tokio;
use telegram_bot::*;
use web3::types::{Address, H160};
use web3::futures::StreamExt;
use rustc_hex::FromHexError;

fn answer_inline_by_article(q: InlineQuery, id: &str, title: &str, content: &str) {
    let article = InlineQueryResultArticle::new(
        id,
        title,
        InputTextMessageContent{
            message_text: content.parse().unwrap(),
            parse_mode: Option::from(ParseMode::Markdown),
            disable_web_page_preview: false
        },
    );
    let r = vec![
        InlineQueryResult::InlineQueryResultArticle(article)
    ];
    q.answer(r);
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN not set");
    let web3_endpoint = env::var("WEB3_ENDPOINT")
        .unwrap_or("https://cloudflare-eth.com".parse().unwrap());
    let web3 = web3::Web3::new(web3::transports::Http::new(&*web3_endpoint).unwrap());
    let api = Api::new(token);

    // Fetch new updates via long poll method
    let mut stream = api.stream();
    while let Some(update) = stream.next().await {
        // If the received update contains a new message...
        let update = update?;
        match update.kind {
            UpdateKind::InlineQuery(q) => {
                // Split query data.
                let query_str = q.query.clone();
                let query_parse: Vec<&str> = query_str.split(' ').collect();
                match query_parse.len() {
                    0 => {
                        answer_inline_by_article(
                            q,
                            "usage",
                            "Command usage",
                            "Use `gas` or `balance <eth address>`",
                        )
                    }
                    1 => {
                        match query_parse[0] {
                            "balance" => {
                                answer_inline_by_article(
                                    q,
                                    "Account needed",
                                    "Enter your account to query",
                                    "Usage: `balance <account>`")
                            }
                            "gas" => {
                                let gas = web3.eth().gas_price().await;
                                match gas {
                                    Ok(v) => {
                                        answer_inline_by_article(
                                            q,
                                            "gas price",
                                            &*format!("Gas Price is {}", v.to_string()),
                                            &*format!("Current gas price of ethereum is {}", v),
                                        )
                                    }
                                    Err(_) => {
                                        answer_inline_by_article(
                                            q,
                                            "fail",
                                            "Failed to fetch gas price",
                                            "Failed to fetch gas price",
                                        )
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    2 => {
                        match query_parse[0] {
                            "balance" => {
                                let addr: Result<H160, FromHexError> = query_parse[1].parse();
                                match addr {
                                    Ok(r) => {
                                        let balance = web3.eth().balance(
                                            Address::from(r), None,
                                        ).await;
                                        match balance {
                                            Ok(r) => {
                                                answer_inline_by_article(
                                                    q,
                                                    &*format!("balance {}", query_parse[1]),
                                                    &*format!("Balance of account {} is {}", query_parse[1], r),
                                                    &*format!("Balance of account {} is {}", query_parse[1], r),
                                                )
                                            }
                                            Err(_) => {}
                                        }
                                    }
                                    Err(_) => {}
                                }
                            }
                            _ => {
                                answer_inline_by_article(
                                    q,
                                    "fail cmd",
                                    "Failed to run your command",
                                    "Cannot run your command",
                                )
                            }
                        }
                    }
                    _ => {}
                }
            }
            UpdateKind::Message(m) => {
            }
            _ => {}
        }
    }
    Ok(())
}
