use scraper::element_ref::Select;
use scraper::{ElementRef, Selector};

struct Table<'a, 'b> {
    root: ElementRef<'a>,
    cells: Select<'a, 'b>,
    _selectors: Vec<Selector>,
}

impl<'a, 'b> Table<'a, 'b> {
    fn new(table: ElementRef<'a>) -> Self {
        let selector = Selector::parse("tbody>tr>td").unwrap();
        let cells = table.select(&selector.clone());
        let mut selectors = Vec::new();
        selectors.push(selector);

        Self {
            root: table,
            cells,
            _selectors: selectors,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scraper::Html;

    fn table(document: &Html) -> ElementRef {
        let selector = Selector::parse("table").unwrap();
        let mut selected = document.select(&selector);

        let table = selected.next().expect("No table found");
        if selected.next().is_some() {
            panic!("More than one table found")
        }

        table
    }

    #[test]
    fn new_cursor_should_point_to_first_row() {
        let document = Html::parse_document(TABLE);
        let table = table(&document);

        let table = Table::new(table);

        assert_eq!(table.cursor.html(), r#"<td>543,858,000</td>"#);
    }

    const TABLE: &str = r#"<table class="board_write sofp">
    <caption>년도에 따른 유동자산[당좌자산(현금 및 현금 등가물, 단기금융상품, 단기투자증권, 매출채권, (받을어음), 단기대여금, 미수금, 미수수익, 선급금, 선급비용, 선급법인세, 기타),
        재고자산(상품,제품,원재료,기타)], 비유동자산[투자자산(장기금융상품,장기투자증권,장기대여금,기타), 유형자산(토지,건물,기계장치,차량운반구,건설중인자산,기타), 무형자산(영업권,개발비,기타),
        기타비유동자산(보증금,장기성매출채권,이연법인세자산,부도어음,기타)], 자산총계를 확인할 수 있는 폼
    </caption>
    <colgroup>
        <col style="width: auto;">
        <col style="width: 19%;">
        <col style="width: 19%;">
        <col style="width: 19%;">
    </colgroup>
    <thead>
    <tr>
        <th scope="col">년도</th>
        <th scope="col">2023</th>
        <th scope="col">2022</th>
        <th scope="col">2021</th>
    </tr>
    </thead>
    <tbody>
    <tr class="dep1">
        <th scope="row">Ⅰ. 유동자산</th>
        <td>543,858,000</td>
        <td>504,274,000</td>
        <td>217,228,000</td>
    </tr>
    <tr class="dep2">
        <th scope="row">1. 당좌자산</th>
        <td>472,608,000</td>
        <td>386,033,000</td>
        <td>217,228,000</td>
    </tr>
    <tr>
        <th scope="row">(1) 현금 및 현금성자산</th>
        <td>308,131,000</td>
        <td>330,783,000</td>
        <td>163,202,000</td>
    </tr>
    <tr>
        <th scope="row">(2) 단기금융상품</th>
        <td>0</td>
        <td>0</td>
        <td>0</td>
    </tr>
    </tbody>
</table>"#;
}
