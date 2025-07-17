use actix_web::{get, web, App, HttpServer, Responder, HttpResponse};
use scraper::{Html, Selector};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct FinancialData {
    name: Option<String>,
    code: Option<String>,
    update_time: Option<String>,
    current_value: Option<f64>,
    bid_value: Option<f64>,
    previous_day_change: Option<f64>,
    change_rate: Option<f64>,
}

#[get("/quote/{symbol}")]
pub async fn get_quote(symbol: web::Path<String>) -> impl Responder {
    let mut targets = HashMap::new();
    targets.insert("^DJI", ("Dow Jones Industrial Average", "https://finance.yahoo.co.jp/quote/^DJI"));
    targets.insert("998407.O", ("Nikkei 225", "https://finance.yahoo.co.jp/quote/998407.O"));
    targets.insert("6758", ("Sony Group Corp.", "https://finance.yahoo.co.jp/quote/6758"));
    targets.insert("USDJPY=FX", ("USD/JPY", "https://finance.yahoo.co.jp/quote/USDJPY=FX"));

    let symbol_str = symbol.as_str();

    if let Some((_, url)) = targets.get(symbol_str) {
        match fetch_financial_data(url).await {
            Ok(data) => HttpResponse::Ok().json(data),
            Err(e) => HttpResponse::InternalServerError().body(format!("Failed to fetch data: {}", e)),
        }
    } else {
        HttpResponse::NotFound().body(format!("Symbol not found: {}", symbol_str))
    }
}

pub async fn fetch_financial_data(url: &str) -> Result<FinancialData, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let html_content = client.get(url).send().await?.text().await?;
    let document = Html::parse_document(&html_content);

    let mut data = FinancialData {
        name: None,
        code: None,
        update_time: None,
        current_value: None,
        bid_value: None,
        previous_day_change: None,
        change_rate: None,
    };

    // 会社名・指数名のセレクター
    let name_selector = Selector::parse("h2[class*=\"PriceBoard__name\"]").unwrap();
    if let Some(element) = document.select(&name_selector).next() {
        data.name = Some(element.text().collect::<String>());
    }

    // 企業コードのセレクター
    let code_selector = Selector::parse("span[class*=\"PriceBoard__code\"]").unwrap();
    if !url.contains("USDJPY=FX") {
        if let Some(element) = document.select(&code_selector).next() {
            data.code = Some(element.text().collect::<String>());
        }
    }

    // 更新時間のセレクター
    let time_selector = Selector::parse("time").unwrap();
    if let Some(element) = document.select(&time_selector).next() {
        data.update_time = Some(element.text().collect::<String>());
    }

    // USD/JPYの場合の特別な処理
    if url.contains("USDJPY=FX") {
        let bid_term_selector = Selector::parse("dt[class*=\"_FxPriceBoard__term\"]").unwrap();
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
                                data.bid_value = Some(bid_price);
                                break;
                            }
                        }
                    }
                }
            }
        }
    } else { // USD/JPY以外の場合の処理
        // 価格のセレクター
        let price_selector = Selector::parse("span[class*=\"PriceBoard__price\"] span[class*=\"StyledNumber__value\"]").unwrap();
        if let Some(element) = document.select(&price_selector).next() {
            let price_str = element.text().collect::<String>();
            let price: f64 = price_str.replace(",", "").parse().unwrap_or(0.0);
            data.current_value = Some(price);
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

            data.previous_day_change = Some(previ);
            data.change_rate = Some(rate);
        }
    }

    Ok(data)
}
