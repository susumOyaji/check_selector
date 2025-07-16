use scraper::{Html, Selector};

fn main() {
    let html_content = r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <title>Test Page</title>
        </head>
        <body>
            <div id="container">
                <p class="item">First item</p>
                <p class="item">Second item</p>
                

                class="PriceBoard__main__1liM"><header class="PriceBoard__header__2Wi4"><div class="PriceBoard__nameBlock__3rFf"><h2 class="PriceBoard__name__166W">ソニーグループ(株)</h2>
                </div></header><div id="industry" class="PriceBoard__mainHeader__3MRw target_modules"><span class="PriceBoard__code__SnMF">6758</span><a href="https://finance.yahoo.co.jp/search/qi/?ids=3650" class="PriceBoard__industryName__3vYM PriceBoard__industryName--link__Ahtz" data-cl-params="_cl_link:_;_cl_position:0">電気機器</a>
                </div><div class="PriceBoard__priceInformation__78Tl"><div class="PriceBoard__priceBlock__1PmX"><span class="StyledNumber__1fof StyledNumber--vertical__2aoh PriceBoard__price__1V0k"><span class="StyledNumber__item__1-yu"><span class="StyledNumber__value__3rXW">3,533</span></span></span>
                <div class="PriceChangeLabel__2Kf0 PriceChangeLabel--red__2zs-"><dl class="PriceChangeLabel__definition__3Jdj"><dt class="PriceChangeLabel__term__3H4k">前日比</dt><dd class="PriceChangeLabel__description__a5Lp"><span class="StyledNumber__1fof StyledNumber--horizontal__HwH8 PriceChangeLabel__prices__30Ey">
                <span class="StyledNumber__item__1-yu PriceChangeLabel__primary__Y_ut"><span class="StyledNumber__value__3rXW">-33</span></span><span class="StyledNumber__item__1-yu StyledNumber__item--secondary__RTJc StyledNumber__item--small__2hJE PriceChangeLabel__secondary__3BXI">
                <span class="StyledNumber__punctuation__3pWV">(</span><span class="StyledNumber__value__3rXW">-0.93</span><span class="StyledNumber__suffix__2SD5">%</span><span class="StyledNumber__punctuation__3pWV">)</span></span></span></dd></dl></div></div><div class="PriceBoard__mainFooter__16pO"><ul class="PriceBoard__times__3vyU">
                <li class="PriceBoard__time__3ixW">リアルタイム株価</li><li class="PriceBoard__time__3ixW"><time>15:01</time></li></ul></div><div class="PriceBoard__rightContents__3abw">
                 
               
            </div>
                <a href="https://example.com">Link</a>
            </div>
            <div class="another-container">
                <p>Another paragraph</p>
            </div>
        </body>
        </html>
    "#;

    let document = Html::parse_document(html_content);

    // 例1: IDで要素を選択
    let selector_id = Selector::parse("#container").unwrap();
    println!("--- Selecting by ID: #container ---");
    for element in document.select(&selector_id) {
        println!("Found element: {}", element.html());
    }

    // 例2: クラスで要素を選択
    let selector_class = Selector::parse(".item").unwrap();
    println!("\n--- Selecting by Class: .item ---");
    for element in document.select(&selector_class) {
        println!("Found element text: {}", element.text().collect::<String>());
    }

    // 例3: タグ名と属性で要素を選択
    let selector_a = Selector::parse("a[href]").unwrap();
    println!("\n--- Selecting by Tag and Attribute: a[href] ---");
    for element in document.select(&selector_a) {
        if let Some(href) = element.value().attr("href") {
            println!("Found link with href: {}", href);
        }
    }

    // 例4: 存在しないセレクター
    let selector_non_existent = Selector::parse(".non-existent").unwrap();
    println!("\n--- Selecting non-existent class: .non-existent ---");
    if document.select(&selector_non_existent).next().is_none() {
        println!("No elements found for .non-existent");
    } else {
        println!("Elements found for .non-existent (unexpected)");
    }

    // 例5: 特定のクラス名で数値要素を抽出し、変数に代入
    let selector_number = Selector::parse(".StyledNumber__value__3rXW").unwrap();
    println!("\n--- Extracting numbers and assigning to variables ---");
    let numbers: Vec<String> = document
        .select(&selector_number)
        .map(|el| el.text().collect::<String>())
        .collect();

    if numbers.len() >= 3 {
        // 1番目の数値をpriceに代入
        let price: f64 = numbers[0].replace(',', "").parse().unwrap_or(0.0);

        // 2番目の数値をpreviに代入
        let previ: f64 = numbers[1].parse().unwrap_or(0.0);

        // 3番目の数値をrateに代入
        let rate: f64 = numbers[2].parse().unwrap_or(0.0);

        println!("株価 (price): {}", price);
        println!("前日比 (previ): {}", previ);
        println!("騰落率 (rate): {}", rate);
    } else {
        println!("Could not extract all three values.");
    }

    // 例6: ソニーグループ(株)を抜き出すセレクター
    let selector_sony_group = Selector::parse(".PriceBoard__name__166W").unwrap();
    println!("\n--- Extracting 'ソニーグループ(株)' ---");
    for element in document.select(&selector_sony_group) {
        let sony_text = element.text().collect::<String>();
        println!("Extracted: {}", sony_text);
    }
}