-- Write your up sql migration here
CREATE TABLE company
(
    id                           TEXT PRIMARY KEY NOT NULL CHECK ( length(id) = 10 ),
    representative_name          TEXT             NOT NULL,
    headquarters_address         TEXT             NOT NULL,
    business_registration_number TEXT             NOT NULL CHECK ( length(business_registration_number) = 10 ),
    company_name                 TEXT             NOT NULL,
    industry_code                TEXT             NOT NULL CHECK ( length(industry_code) = 5 ),
    industry_name                TEXT             NOT NULL
);

-- Represents a company with its details.
--
-- let company = Company {
--   id: String::from("1071180"),
--   representative_name: String::from("김성국"),
--   headquarters_address: String::from("경기도 김포시"),
--   business_registration_number: String::from("5632000760"),
--   company_name: String::from("루키게임즈"),
--   industry_code: String::from("63999"),
--   industry_name: String::from("그 외 기타 정보 서비스업")
-- };