use crate::error::{BuildError, HtmlParseError};
use crate::parser::utils::{join_text_nodes, parse_comma_sep_digit};
use crate::SmesError;
use derive_builder::Builder;
use scraper::{ElementRef, Selector};

pub struct Table<'a> {
    root: ElementRef<'a>,
}

#[derive(Debug, PartialEq, Clone)]
enum Level {
    Actual,
    Group,
}

#[derive(Builder, PartialEq, Debug, Clone)]
#[builder(setter(strip_option, into))]
pub struct Cell {
    level: Level,
    #[builder(default)]
    dep1: Option<String>,
    #[builder(default)]
    dep2: Option<String>,
    account: String,
    year: String,
    value: i64,
}

impl<'a> Table<'a> {
    pub fn new(table: ElementRef<'a>) -> Self {
        Self { root: table }
    }

    pub fn parse_body(&self) -> Result<Vec<Cell>, SmesError> {
        // Keep the states during iteration
        let mut dep1: Option<String> = None;
        let mut dep2: Option<String> = None;

        let years = self.years()?;
        let mut years = years.iter().cycle();

        // All the cells parsed from the table
        let mut cells = Vec::<Cell>::new();

        // region: Iteration
        let selector = Selector::parse("tbody>tr")?;
        let rows = self.root.select(&selector);

        for row in rows {
            let mut account: Option<String> = None;

            match row.attr("class") {
                Some("dep1") => {
                    for cell in row.child_elements() {
                        let value = join_text_nodes(cell.text());

                        match cell.value().name.local.as_ref() {
                            "th" => {
                                account = Some(value.clone());
                                dep1 = Some(value.clone());
                                dep2 = None;
                            }
                            "td" => {
                                let year = years.next().expect("Year not found. It should have been set before the current call");
                                let dep1 = dep1.as_ref().expect("dep1 not found. It should have been set before the current call");

                                let parsed = CellBuilder::default()
                                    .level(Level::Group)
                                    .dep1(dep1)
                                    .account(account.clone().expect("Account not found").to_owned())
                                    .year(year)
                                    .value(parse_comma_sep_digit(&value)?)
                                    .build()
                                    .map_err(|e| BuildError {
                                        source: Some(Box::new(e)),
                                        message: "Failed to build cell",
                                    })?;
                                cells.push(parsed);
                            }
                            _ => panic!("Unexpected element"),
                        };
                    }
                }
                Some("dep2") => {
                    for cell in row.child_elements() {
                        let value = join_text_nodes(cell.text());

                        match cell.value().name.local.as_ref() {
                            "th" => {
                                account = Some(value.clone());
                                dep2 = Some(value.clone());
                            }
                            "td" => {
                                let year = years.next().expect("Year not found. It should have been set before the current call");
                                let dep1 = dep1.as_ref().expect("dep1 not found. It should have been set before the current call");
                                let dep2 = dep2.as_ref().expect("dep2 not found. It should have been set before the current call");

                                let parsed = CellBuilder::default()
                                    .level(Level::Group)
                                    .dep1(dep1)
                                    .dep2(dep2)
                                    .account(account.clone().expect("Account not found").to_owned())
                                    .year(year)
                                    .value(parse_comma_sep_digit(&value)?)
                                    .build()
                                    .map_err(|e| BuildError {
                                        source: Some(Box::new(e)),
                                        message: "Failed to build cell",
                                    })?;
                                cells.push(parsed);
                            }
                            _ => panic!("Unexpected element"),
                        };
                    }
                }
                None => {
                    for cell in row.child_elements() {
                        let value = join_text_nodes(cell.text());

                        match cell.value().name.local.as_ref() {
                            "th" => {
                                account = Some(value.clone());
                            }
                            "td" => {
                                let year = years.next().expect("Year not found. It should have been set before the current call");
                                let dep1 = dep1.as_ref().expect("dep1 not found. It should have been set before the current call");
                                let dep2 = dep2.as_ref().expect("dep2 not found. It should have been set before the current call");

                                let parsed = CellBuilder::default()
                                    .level(Level::Actual)
                                    .dep1(dep1)
                                    .dep2(dep2)
                                    .account(account.clone().expect("Account not found").to_owned())
                                    .year(year)
                                    .value(parse_comma_sep_digit(&value)?)
                                    .build()
                                    .map_err(|e| BuildError {
                                        source: Some(Box::new(e)),
                                        message: "Failed to build cell",
                                    })?;
                                cells.push(parsed);
                            }
                            _ => panic!("Unexpected element"),
                        };
                    }
                }
                _ => unimplemented!(),
            }
        }
        // endregion: Iteration

        Ok(cells)
    }

    fn years(&self) -> Result<Vec<String>, SmesError> {
        let selector = Selector::parse("thead>tr>th")?;
        let mut header = self.root.select(&selector);
        if header.next().is_none() {
            Err(HtmlParseError {
                source: None,
                message: "No header found",
            })?;
        };

        Ok(header.map(|year| join_text_nodes(year.text())).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scraper::Html;

    #[test]
    fn dep1_should_parse_as_expected() {
        let document = Html::parse_fragment(&format_table(vec![ROW_DEP1]));
        let table = Table::new(table_element(&document));

        let cells = table.parse_body().unwrap();

        let expected_dep1_and_account = "Ⅰ. 유동자산".to_string();

        assert_eq!(
            cells,
            vec![
                Cell {
                    level: Level::Group,
                    dep1: Some(expected_dep1_and_account.clone()),
                    dep2: None,
                    account: expected_dep1_and_account.clone(),
                    year: "2023".to_string(),
                    value: 543_858_000
                },
                Cell {
                    level: Level::Group,
                    dep1: Some(expected_dep1_and_account.clone()),
                    dep2: None,
                    account: expected_dep1_and_account.clone(),
                    year: "2022".to_string(),
                    value: 504_274_000
                },
                Cell {
                    level: Level::Group,
                    dep1: Some(expected_dep1_and_account.clone()),
                    dep2: None,
                    account: expected_dep1_and_account.clone(),
                    year: "2021".to_string(),
                    value: 217_228_000
                }
            ]
        );
    }

    #[test]
    fn dep2_should_parse_as_expected() {
        let document = Html::parse_fragment(&format_table(vec![ROW_DEP1, ROW_DEP2]));
        let table = Table::new(table_element(&document));

        let cells = table.parse_body().unwrap();
        assert_eq!(cells.len(), 6);

        let dep2_cells = cells
            .into_iter()
            .filter(|cell| cell.dep2.is_some())
            .collect::<Vec<_>>();

        let expected_dep1 = "Ⅰ. 유동자산".to_string();
        let expected_dep2_and_account = "1. 당좌자산".to_string();

        assert_eq!(
            dep2_cells,
            vec![
                Cell {
                    level: Level::Group,
                    dep1: Some(expected_dep1.clone()),
                    dep2: Some(expected_dep2_and_account.clone()),
                    account: expected_dep2_and_account.clone(),
                    year: "2023".to_string(),
                    value: 472_608_000
                },
                Cell {
                    level: Level::Group,
                    dep1: Some(expected_dep1.clone()),
                    dep2: Some(expected_dep2_and_account.clone()),
                    account: expected_dep2_and_account.clone(),
                    year: "2022".to_string(),
                    value: 386_033_000
                },
                Cell {
                    level: Level::Group,
                    dep1: Some(expected_dep1.clone()),
                    dep2: Some(expected_dep2_and_account.clone()),
                    account: expected_dep2_and_account.clone(),
                    year: "2021".to_string(),
                    value: 217_228_000
                }
            ]
        );
    }

    #[test]
    fn no_dep_should_parse_as_expected() {
        let document = Html::parse_fragment(&format_table(vec![ROW_DEP1, ROW_DEP2, ROW]));
        let table = Table::new(table_element(&document));

        let cells = table.parse_body().unwrap();
        assert_eq!(cells.len(), 9);

        let row_cells = cells
            .into_iter()
            .filter(|cell| matches!(cell.level, Level::Actual))
            .collect::<Vec<_>>();

        let expected_dep1 = "Ⅰ. 유동자산".to_string();
        let expected_dep2 = "1. 당좌자산".to_string();
        let expected_account = "(1) 현금 및 현금성자산".to_string();

        assert_eq!(
            row_cells,
            vec![
                Cell {
                    level: Level::Actual,
                    dep1: Some(expected_dep1.clone()),
                    dep2: Some(expected_dep2.clone()),
                    account: expected_account.clone(),
                    year: "2023".to_string(),
                    value: 308_131_000
                },
                Cell {
                    level: Level::Actual,
                    dep1: Some(expected_dep1.clone()),
                    dep2: Some(expected_dep2.clone()),
                    account: expected_account.clone(),
                    year: "2022".to_string(),
                    value: 330_783_000
                },
                Cell {
                    level: Level::Actual,
                    dep1: Some(expected_dep1.clone()),
                    dep2: Some(expected_dep2.clone()),
                    account: expected_account.clone(),
                    year: "2021".to_string(),
                    value: 163_202_000
                }
            ]
        );
    }

    fn format_table(rows: Vec<&str>) -> String {
        let rows: String = rows.into_iter().map(|row| row.to_string()).collect();

        format!(
            r#"<table>
    <thead>
    <tr>
        <th scope="col">년도</th>
        <th scope="col">2023</th>
        <th scope="col">2022</th>
        <th scope="col">2021</th>
    </tr>
    </thead>
    <tbody>
    {}
    </tbody>
</table>"#,
            rows
        )
    }

    const ROW_DEP1: &str = r#"<tr class="dep1">
            <th scope="row">Ⅰ. 유동자산</th>
            <td>543,858,000</td>
            <td>504,274,000</td>
            <td>217,228,000</td>
        </tr>"#;

    const ROW_DEP2: &str = r#"<tr class="dep2">
        <th scope="row">1. 당좌자산</th>
        <td>472,608,000</td>
        <td>386,033,000</td>
        <td>217,228,000</td>
    </tr>"#;

    const ROW: &str = r#"<tr>
        <th scope="row">(1) 현금 및 현금성자산</th>
        <td>308,131,000</td>
        <td>330,783,000</td>
        <td>163,202,000</td>
    </tr>"#;

    const TABLE: &str = r#"<table class="board_write sofp">
    <caption>년도에 따른 유동자산[당좌자산(현금 및 현금 등가물, 단기금융상품, 단기투자증권, 매출채권, (받을어음), 단기대여금, 미수금, 미수수익, 선급금, 선급비용, 선급법인세, 기타),
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

    #[test]
    fn years_should_work_as_expected() {
        let document = Html::parse_fragment(TABLE);
        let table = Table::new(table_element(&document));
        let years = table.years().unwrap();

        assert_eq!(years, vec!["2023", "2022", "2021"]);
    }

    fn table_element(fragment: &Html) -> ElementRef {
        let selector = Selector::parse("table").unwrap();
        let mut selected = fragment.select(&selector);

        let table = selected.next().expect("No table found");
        if selected.next().is_some() {
            panic!("More than one table found")
        }

        table
    }
}
