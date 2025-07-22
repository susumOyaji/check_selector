use actix_web::{get, post, web, HttpResponse, Responder};
use futures::future;
use scraper::{Element, Html, Selector};
use serde::{Deserialize, Serialize};

// フロントエンドから受け取るJSONの構造体
#[derive(Deserialize)]
pub struct QuoteRequest {
    code: String,
}

// フロントエンドに返すJSONの構造体
#[derive(Serialize, Default, Clone)]
pub struct FinancialData {
    pub code: Option<String>,
    pub name: Option<String>,
    pub current_value: Option<f64>,
    pub previous_day_change: Option<f64>,
    pub change_rate: Option<f64>,
    pub bid_value: Option<f64>,
    pub update_time: Option<String>,
}

// POST /api/quote エンドポイントのハンドラ (個別検索)
#[post("/api/quote")]
pub async fn get_quote_by_code(req: web::Json<QuoteRequest>) -> impl Responder {
    let url = format!("https://finance.yahoo.co.jp/quote/{}", req.code);
    match fetch_financial_data(url).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(e) => HttpResponse::InternalServerError().body(format!("データの取得に失敗しました: {}", e)),
    }
}

// GET /api/default エンドポイントのハンドラ (デフォルト指標検索)
#[get("/api/default")]
pub async fn get_default_quotes() -> impl Responder {
    let default_codes = vec!["^DJI", "998407.O", "USDJPY=FX"];

    let futures = default_codes.into_iter().map(|code| {
        let url = format!("https://finance.yahoo.co.jp/quote/{}", code);
        fetch_financial_data(url)
    });

    let results: Vec<Result<FinancialData, _>> = future::join_all(futures).await;
    let successful_results: Vec<FinancialData> = results.into_iter().filter_map(Result::ok).collect();
    HttpResponse::Ok().json(successful_results)
}

// エラーを統一的に扱うための型エイリアス
type FetchResult<T> = Result<T, Box<dyn std::error::Error>>;

// テキストを抽出して返すヘルパー関数
fn get_text(doc: &Html, selector_str: &str) -> FetchResult<Option<String>> {
    let selector = Selector::parse(selector_str).map_err(|e| e.to_string())?;
    Ok(doc.select(&selector).next().map(|el| el.text().collect::<String>()))
}

// 数値に変換するヘルパー関数
fn parse_f64(text: &str) -> f64 {
    text.replace(',', "").parse().unwrap_or(0.0)
}

// URLを所有権で受け取るように変更
pub async fn fetch_financial_data(url: String) -> FetchResult<FinancialData> {
    let client = reqwest::Client::new();
    let html_content = client.get(&url).send().await?.text().await?;
    let document = Html::parse_document(&html_content);

    let mut data = FinancialData::default();

    data.name = get_text(&document, "h2[class*=\"PriceBoard__name\"]")?;
    data.update_time = get_text(&document, "time")?;

    if !url.contains("USDJPY=FX") {
        data.code = get_text(&document, "span[class*=\"PriceBoard__code\"]")?;

        if let Some(price_text) = get_text(&document, "span[class*=\"PriceBoard__price\"] span[class*=\"StyledNumber__value\"]")? {
            data.current_value = Some(parse_f64(&price_text));
        }

        let change_selector = Selector::parse("div[class*=\"PriceChangeLabel\"] span[class*=\"StyledNumber__value\"]").map_err(|e| e.to_string())?;
        let changes: Vec<String> = document.select(&change_selector).map(|el| el.text().collect()).collect();

        if let Some(change_text) = changes.get(0) {
            data.previous_day_change = Some(parse_f64(change_text));
        }
        if let Some(rate_text) = changes.get(1) {
            data.change_rate = Some(parse_f64(rate_text));
        }
    } else {
        let bid_term_selector = Selector::parse("dt[class*=\"_FxPriceBoard__term\"]").map_err(|e| e.to_string())?;
        for dt_element in document.select(&bid_term_selector) {
            if dt_element.text().collect::<String>().trim() == "Bid（売値）" {
                if let Some(dd_element) = dt_element.next_sibling_element() {
                    let price_span_selector = Selector::parse("span[class*=\"_FxPriceBoard__price\"]").map_err(|e| e.to_string())?;
                    if let Some(price_span) = dd_element.select(&price_span_selector).next() {
                        let bid_price_str = price_span.text().collect::<String>();
                        data.bid_value = Some(parse_f64(&bid_price_str));
                        break;
                    }
                }
            }
        }
    }

    Ok(data)
}
