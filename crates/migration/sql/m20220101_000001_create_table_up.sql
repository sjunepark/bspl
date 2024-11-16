CREATE SCHEMA smes;

CREATE TABLE smes.company
(
    smes_id                      TEXT PRIMARY KEY CHECK (smes_id ~ '^[0-9]{7}$'),
    representative_name          TEXT      NOT NULL,
    headquarters_address         TEXT      NOT NULL,
    business_registration_number TEXT CHECK (
        business_registration_number ~ '^[0-9]{10}$'
        ),
    company_name                 TEXT      NOT NULL,
    industry_code                TEXT      NOT NULL CHECK (industry_code ~ '^[0-9]{5}$'),
    industry_name                TEXT      NOT NULL,
    created_at                   TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at                   TIMESTAMP NOT NULL DEFAULT current_timestamp
);

CREATE TABLE smes.html
(
    smes_id      TEXT PRIMARY KEY CHECK (smes_id ~ '^[0-9]{7}$'),
    html_content TEXT      NOT NULL,
    created_at   TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at   TIMESTAMP NOT NULL DEFAULT current_timestamp,
    FOREIGN KEY (smes_id) REFERENCES smes.company (smes_id) ON DELETE RESTRICT ON UPDATE CASCADE
);


CREATE SCHEMA dart;

CREATE TABLE dart.filing
(
    dart_id        TEXT PRIMARY KEY CHECK (dart_id ~ '^[0-9]{8}$'),
    report_name    TEXT      NOT NULL,
    receipt_number TEXT      NOT NULL CHECK (receipt_number ~ '^[0-9]{14}$'),
    filer_name     TEXT      NOT NULL,
    receipt_date   DATE      NOT NULL,
    remark         TEXT      NOT NULL,
    created_at     TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at     TIMESTAMP NOT NULL DEFAULT current_timestamp
);

CREATE TABLE dart.company_id
(
    dart_id        TEXT PRIMARY KEY CHECK (dart_id ~ '^[0-9]{8}$'),
    company_name   TEXT NOT NULL,
    stock_code     TEXT NOT NULL CHECK (stock_code ~ '^[0-9]{6}$'),
    id_modify_date DATE NOT NULL
);