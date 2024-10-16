use smes::Table;

#[tokio::main]
async fn main() {
    let html_file = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/resources/searchVntrCmpDtls.html"
    ))
    .expect("Failed to read html file");

    let document = scraper::Html::parse_document(&html_file);
    let selector =
        scraper::Selector::parse(r#"#real_contents > div > div.board_tab_con_box > div:nth-child(2) > div > div:nth-child(1) > div:nth-child(2) > div.sofp_tbl2_box > div:nth-child(1) > table"#).unwrap();
    let element = document.select(&selector).next().unwrap();

    let table = Table::new(element);

    let cells = table.parse_body().unwrap();

    for cell in cells {
        println!("{:?}", cell);
    }
}
