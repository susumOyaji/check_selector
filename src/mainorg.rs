use scraper::{Html, Selector};
use reqwest;
use std::io;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut targets = HashMap::new();
    targets.insert("1", ("Dow Jones Industrial Average", "https://finance.yahoo.co.jp/quote/^DJI"));
    targets.insert("2", ("Nikkei 225", "https://finance.yahoo.co.jp/quote/998407.O"));
    targets.insert("3", ("Sony Group Corp.", "https://finance.yahoo.co.jp/quote/6758"));
    targets.insert("4", ("USD/JPY", "https://finance.yahoo.co.jp/quote/USDJPY=FX"));

    loop {
        println!("
どの金融商品の情報を取得しますか？番号を入力してください (終了するには'q'を入力):");
        for (key, (name, _)) in &targets {
            println!("{}: {}", key, name);
        }

        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;
        let choice = choice.trim();

        if choice == "q" {
            break;
        }

        if let Some((name, url)) = targets.get(choice) {
            println!("
--- {} の情報を取得中... ---", name);
            fetch_and_display_data(url)?;
        } else {
            println!("無効な選択です。もう一度入力してください。");
        }
    }

    Ok(())
}

fn fetch_and_display_data(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let html_content = reqwest::blocking::get(url)?.text()?;
    let document = Html::parse_document(&html_content);

    // 会社名・指数名、企業コード、更新時間のセレクターを定義
    let name_selector = Selector::parse("h2[class*=\"PriceBoard__name\"]").unwrap();
    let code_selector = Selector::parse("span[class*=\"PriceBoard__code\"]").unwrap();
    let time_selector = Selector::parse("time").unwrap();

    // 会社名・指数名の表示
    if let Some(element) = document.select(&name_selector).next() {
        let name = element.text().collect::<String>();
        println!("名称: {}", name);
    } else {
        println!("名称が見つかりませんでした。");
    }

    // 企業コードの表示
    if !url.contains("USDJPY=FX") {
        if let Some(element) = document.select(&code_selector).next() {
            let code = element.text().collect::<String>();
            println!("code: {}", code);
        } else {
            println!("code: 見つかりませんでした。");
        }
    }

    // 更新時間の表示
    if let Some(element) = document.select(&time_selector).next() {
        let update_time = element.text().collect::<String>();
        println!("更新時間: {}", update_time);
    } else {
        println!("更新時間が見つかりませんでした。");
    }

    // USD/JPYの場合の特別な処理
    if url.contains("USDJPY=FX") {
        let bid_term_selector = Selector::parse("dt[class*=\"_FxPriceBoard__term\"]").unwrap();
        let mut bid_price_found = false;
        for dt_element in document.select(&bid_term_selector) {
            if dt_element.text().collect::<String>().trim() == "Bid（売値）" {
                if let Some(parent_dl_node) = dt_element.parent() {
                    if let Some(parent_dl_element) = scraper::ElementRef::wrap(parent_dl_node) {
                        let dd_selector = Selector::parse("dd[class*=\"_FxPriceBoard__description\"]").unwrap();
                        if let Some(dd_element) = parent_dl_element.select(&dd_selector).next() {
                            let price_span_selector = Selector::parse("span[class*=\"_FxPriceBoard__price\"]").unwrap();
                            if let Some(price_span_element) = dd_element.select(&price_span_selector).next() {
                                let bid_price_str = price_span_element.text().collect::<String>();
                                let bid_price: f64 = bid_price_str.replace(",", "").parse().unwrap_or(0.0);
                                println!("Bid（売値）: {}", bid_price);
                                bid_price_found = true;
                                break;
                            }
                        }
                    }
                }
            }
        }
        if !bid_price_found {
            println!("Bid（売値）が見つかりませんでした。");
        }
    } else { // USD/JPY以外の場合の処理
        // 価格のセレクター
        let price_selector = Selector::parse("span[class*=\"PriceBoard__price\"] span[class*=\"StyledNumber__value\"]").unwrap();
        if let Some(element) = document.select(&price_selector).next() {
            let price_str = element.text().collect::<String>();
            let price: f64 = price_str.replace(",", "").parse().unwrap_or(0.0);
            println!("現在値: {}", price);
        } else {
            println!("現在値が見つかりませんでした。");
        }

        // 前日比と騰落率のセレクター
        let change_selector = Selector::parse("div[class*=\"PriceChangeLabel\"] span[class*=\"StyledNumber__value\"]").unwrap();
        let changes: Vec<String> = document
            .select(&change_selector)
            .map(|el| el.text().collect::<String>())
            .collect();

        if changes.len() >= 2 {
            let previ_str = &changes[0];
            let rate_str = &changes[1];

            let previ: f64 = previ_str.parse().unwrap_or(0.0);
            let rate: f64 = rate_str.parse().unwrap_or(0.0);

            println!("前日比: {}", previ);
            println!("騰落率: {}%", rate);
        } else {
            println!("前日比・騰落率が見つかりませんでした。");
        }
    }

    Ok(())
}