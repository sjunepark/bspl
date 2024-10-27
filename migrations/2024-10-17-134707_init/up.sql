-- Your SQL goes here
CREATE SCHEMA smes;

CREATE TABLE smes.company
(
    company_id                   TEXT PRIMARY KEY CHECK (company_id ~ '^[0-9]{7}$'),
    representative_name          TEXT      NOT NULL,
    headquarters_address         TEXT      NOT NULL,
    business_registration_number TEXT      NOT NULL CHECK (
        business_registration_number = '' OR
        business_registration_number ~ '^[0-9]{10}$'
        ),
    company_name                 TEXT      NOT NULL,
    industry_code                TEXT      NOT NULL CHECK (industry_code ~ '^[0-9]{5}$'),
    industry_name                TEXT      NOT NULL,
    created_at                   TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at                   TIMESTAMP NOT NULL DEFAULT current_timestamp
);
SELECT diesel_manage_updated_at('smes.company');

CREATE TABLE smes.html
(
    company_id   TEXT PRIMARY KEY CHECK (company_id ~ '^[0-9]{7}$'),
    html_content TEXT      NOT NULL,
    created_at   TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at   TIMESTAMP NOT NULL DEFAULT current_timestamp,
    FOREIGN KEY (company_id) REFERENCES smes.company (company_id) ON DELETE RESTRICT ON UPDATE CASCADE
);
SELECT diesel_manage_updated_at('smes.html');


CREATE SCHEMA dart;

CREATE TABLE dart.filing
(
    corp_code  TEXT PRIMARY KEY CHECK (corp_code ~ '^[0-9]{8}$'),
    corp_name  TEXT      NOT NULL,
    stock_code TEXT      NOT NULL CHECK (stock_code ~ '^[0-9]{6}$'),
    corp_cls   TEXT      NOT NULL CHECK (corp_cls IN ('Y', 'K', 'N', 'E')),
    report_nm  TEXT      NOT NULL,
    rcept_no   TEXT      NOT NULL CHECK (rcept_no ~ '^[0-9]{14}$'),
    flr_nm     TEXT      NOT NULL,
    rcept_dt   DATE      NOT NULL,
    rm         TEXT      NOT NULL CHECK (rm ~ '^[유코채넥공연정철]+$'),
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);
SELECT diesel_manage_updated_at('dart.filing');